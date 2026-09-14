#[cfg(unix)]
use crate::core::unix_config::{atomic_write, parse_config, read_config_contents};
use crate::core::{
    error::EnvError,
    model::{BackupInfo, BackupPayload, Change, EnvDiff, EnvVar, PathEntry, Scope},
    path::{join_path, parse_path},
    traits::EnvProvider,
};
use crate::platform;
#[cfg(unix)]
use chrono::Local;
use chrono::{DateTime, Utc};
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::{Path, PathBuf},
    sync::{Mutex, MutexGuard, OnceLock},
};
#[cfg(windows)]
use windows_sys::Win32::UI::Shell::ShellExecuteW;
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

type SharedProvider = Mutex<Box<dyn EnvProvider + Send + Sync>>;
static PROVIDER: OnceLock<SharedProvider> = OnceLock::new();

fn provider() -> &'static SharedProvider {
    PROVIDER.get_or_init(|| Mutex::new(platform::create_provider()))
}

fn lock_provider() -> Result<MutexGuard<'static, Box<dyn EnvProvider + Send + Sync>>, EnvError> {
    provider()
        .lock()
        .map_err(|_| EnvError::io("环境变量管理器状态已被锁定"))
}

async fn with_provider<T, F>(operation: F) -> Result<T, EnvError>
where
    T: Send + 'static,
    F: FnOnce(&dyn EnvProvider) -> Result<T, EnvError> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let provider = lock_provider()?;
        operation(provider.as_ref())
    })
    .await
    .map_err(|error| EnvError::io(format!("后台任务执行失败：{error}")))?
}

fn apply_changes_impl(
    provider: &dyn EnvProvider,
    changes: Vec<Change>,
) -> Result<EnvDiff, EnvError> {
    provider.preview(&changes)?;
    let Some(scope) = changes.first().map(change_scope) else {
        return Ok(EnvDiff::default());
    };
    if !provider.check_permission(scope)? {
        return Err(permission_message(scope));
    }

    provider.backup(scope)?;
    let mut actual_diff = EnvDiff::default();
    for change in changes {
        let operation_diff = match change {
            Change::Add(variable) | Change::Modify(variable) => provider.set_var(&variable)?,
            Change::Remove(variable) => provider.remove_var(&variable.name, variable.scope)?,
        };
        merge_diff(&mut actual_diff, operation_diff);
    }
    Ok(actual_diff)
}

fn merge_diff(target: &mut EnvDiff, source: EnvDiff) {
    target.added.extend(source.added);
    target.modified.extend(source.modified);
    target.removed.extend(source.removed);
}

fn change_scope(change: &Change) -> Scope {
    match change {
        Change::Add(variable) | Change::Modify(variable) | Change::Remove(variable) => {
            variable.scope
        }
    }
}

fn permission_message(scope: Scope) -> EnvError {
    match scope {
        Scope::User => EnvError::permission_denied("没有权限修改用户环境变量"),
        Scope::System => EnvError::permission_denied("没有权限修改系统环境变量，请提升权限后重试"),
    }
}

fn list_backups_sync() -> Result<Vec<BackupInfo>, EnvError> {
    let directory = backup_directory()?;
    if !directory.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if !is_backup_file(&path) {
            continue;
        }
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let Some(scope) = infer_backup_scope(file_name) else {
            continue;
        };
        let modified = entry.metadata()?.modified()?;
        let created_at: DateTime<Utc> = modified.into();
        backups.push(BackupInfo {
            path: path.to_string_lossy().into_owned(),
            scope,
            created_at: created_at.to_rfc3339(),
        });
    }

    backups.sort_by(|left, right| right.created_at.cmp(&left.created_at));
    Ok(backups)
}

fn restore_backup_with_provider(provider: &dyn EnvProvider, path: String) -> Result<(), EnvError> {
    let path = validate_backup_path(&path)?;

    #[cfg(windows)]
    {
        let file = fs::File::open(&path)?;
        let payload: BackupPayload = serde_json::from_reader(file)
            .map_err(|error| EnvError::parse(format!("备份文件格式错误：{error}")))?;
        if !provider.check_permission(payload.scope)? {
            return Err(permission_message(payload.scope));
        }
        provider.backup(payload.scope)?;

        let current = provider.list_vars(payload.scope)?;
        let backup_names = payload
            .variables
            .iter()
            .map(|variable| variable.name.to_lowercase())
            .collect::<HashSet<_>>();
        for variable in payload.variables {
            provider.set_var(&variable)?;
        }
        for variable in current {
            if !backup_names.contains(&variable.name.to_lowercase()) {
                provider.remove_var(&variable.name, payload.scope)?;
            }
        }
    }

    #[cfg(unix)]
    {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| EnvError::parse("备份文件名无效"))?;
        let scope = infer_backup_scope(file_name)
            .ok_or_else(|| EnvError::parse("无法识别备份所属作用域"))?;
        let target = unix_restore_target(file_name, scope)
            .ok_or_else(|| EnvError::parse("无法确定备份对应的配置文件"))?;
        if !provider.check_permission(scope)? {
            return Err(permission_message(scope));
        }
        if target.exists() {
            let directory = backup_directory()?;
            fs::create_dir_all(&directory)?;
            let timestamp = Local::now().format("%Y%m%d-%H%M%S%3f");
            let target_name = target
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| EnvError::io("配置文件名无效"))?;
            let safety_path = directory.join(format!("{target_name}.{timestamp}.pre-restore.bak"));
            fs::copy(&target, safety_path)?;
        }
        let contents = fs::read_to_string(&path)?;
        atomic_write(&target, &contents)?;
    }

    Ok(())
}

fn preview_backup_restore_with_provider(
    provider: &dyn EnvProvider,
    path: String,
) -> Result<EnvDiff, EnvError> {
    let path = validate_backup_path(&path)?;

    #[cfg(windows)]
    {
        let file = fs::File::open(&path)?;
        let payload: BackupPayload = serde_json::from_reader(file)
            .map_err(|error| EnvError::parse(format!("备份文件格式错误：{error}")))?;
        let current = provider.list_vars(payload.scope)?;
        build_restore_diff(&current, &payload.variables, false)
    }

    #[cfg(unix)]
    {
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| EnvError::parse("备份文件名无效"))?;
        let scope = infer_backup_scope(file_name)
            .ok_or_else(|| EnvError::parse("无法识别备份所属作用域"))?;
        let target = unix_restore_target(file_name, scope)
            .ok_or_else(|| EnvError::parse("无法确定备份对应的配置文件"))?;
        let source = unix_source_label(&target, scope);
        let contents = read_config_contents(&path)?;
        let backup = parse_config(&contents, scope, &source)?;
        let current = provider
            .list_vars(scope)?
            .into_iter()
            .filter(|variable| variable.source.as_deref() == Some(source.as_str()))
            .collect::<Vec<_>>();
        build_restore_diff(&current, &backup, true)
    }
}

fn build_restore_diff(
    current: &[EnvVar],
    backup: &[EnvVar],
    case_sensitive_names: bool,
) -> Result<EnvDiff, EnvError> {
    let mut current_by_name = HashMap::new();
    for variable in current {
        let name = restore_name_key(&variable.name, case_sensitive_names);
        current_by_name.insert(name, variable);
    }

    let mut backup_names = HashSet::new();
    let mut changes = Vec::new();
    for variable in backup {
        let name = restore_name_key(&variable.name, case_sensitive_names);
        backup_names.insert(name.clone());
        match current_by_name.get(&name) {
            Some(current_variable) if restore_variable_equal(current_variable, variable) => {}
            Some(_) => changes.push(Change::Modify(variable.clone())),
            None => changes.push(Change::Add(variable.clone())),
        }
    }

    for variable in current {
        let name = restore_name_key(&variable.name, case_sensitive_names);
        if !backup_names.contains(&name) {
            changes.push(Change::Remove(variable.clone()));
        }
    }

    crate::core::diff::build_diff(current, &changes, case_sensitive_names)
}

fn restore_name_key(name: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        name.to_owned()
    } else {
        name.to_lowercase()
    }
}

fn restore_variable_equal(current: &EnvVar, backup: &EnvVar) -> bool {
    current.value == backup.value
        && current.scope == backup.scope
        && current.is_expandable == backup.is_expandable
        && current.raw_value == backup.raw_value
}

#[cfg(unix)]
fn unix_source_label(path: &Path, scope: Scope) -> String {
    if scope == Scope::System {
        path.display().to_string()
    } else {
        format!(
            "~/{}",
            path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
        )
    }
}

fn validate_backup_path(path: &str) -> Result<PathBuf, EnvError> {
    let path = PathBuf::from(path);
    let directory = backup_directory()?;
    let canonical_path = path
        .canonicalize()
        .map_err(|_| EnvError::not_found("备份文件不存在"))?;
    let canonical_directory = directory
        .canonicalize()
        .map_err(|_| EnvError::io("备份目录不存在"))?;
    if !canonical_path.starts_with(&canonical_directory) {
        return Err(EnvError::parse("只能恢复 VarMan 备份目录中的文件"));
    }
    Ok(canonical_path)
}

fn backup_directory() -> Result<PathBuf, EnvError> {
    #[cfg(windows)]
    {
        let app_data = env::var_os("APPDATA")
            .map(PathBuf::from)
            .ok_or_else(|| EnvError::io("无法确定 APPDATA 目录"))?;
        Ok(app_data.join("VarMan").join("backups"))
    }
    #[cfg(unix)]
    {
        let home = env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| EnvError::io("无法确定用户主目录"))?;
        Ok(home.join(".VarMan").join("backups"))
    }
}

fn is_backup_file(path: &Path) -> bool {
    #[cfg(windows)]
    {
        path.extension()
            .is_some_and(|extension| extension == "json")
    }
    #[cfg(unix)]
    {
        path.extension().is_some_and(|extension| extension == "bak")
    }
}

fn infer_backup_scope(file_name: &str) -> Option<Scope> {
    #[cfg(windows)]
    {
        if file_name.starts_with("user-") {
            Some(Scope::User)
        } else if file_name.starts_with("system-") {
            Some(Scope::System)
        } else {
            None
        }
    }
    #[cfg(unix)]
    {
        if file_name.starts_with("environment.") || file_name.starts_with("profile.") {
            Some(Scope::System)
        } else {
            Some(Scope::User)
        }
    }
}

#[cfg(unix)]
fn unix_restore_target(file_name: &str, scope: Scope) -> Option<PathBuf> {
    let base = file_name.strip_suffix(".bak")?;
    let home = env::var_os("HOME").map(PathBuf::from)?;
    let candidates = match scope {
        Scope::User => vec![
            home.join(".zshrc"),
            home.join(".bashrc"),
            home.join(".bash_profile"),
            home.join(".profile"),
        ],
        Scope::System => vec![
            PathBuf::from("/etc/environment"),
            PathBuf::from("/etc/profile"),
        ],
    };

    candidates.into_iter().find(|candidate| {
        candidate
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| base.starts_with(&format!("{name}.")))
    })
}

fn path_separator() -> char {
    if cfg!(windows) {
        ';'
    } else {
        ':'
    }
}

#[tauri::command]
pub async fn list_env_vars(scope: Scope) -> Result<Vec<EnvVar>, EnvError> {
    with_provider(move |provider| provider.list_vars(scope)).await
}

#[tauri::command]
pub async fn get_env_var(name: String, scope: Scope) -> Result<Option<EnvVar>, EnvError> {
    with_provider(move |provider| provider.get_var(&name, scope)).await
}

#[tauri::command]
pub async fn preview_changes(changes: Vec<Change>) -> Result<EnvDiff, EnvError> {
    with_provider(move |provider| provider.preview(&changes)).await
}

#[tauri::command]
pub async fn apply_changes(changes: Vec<Change>) -> Result<EnvDiff, EnvError> {
    with_provider(move |provider| apply_changes_impl(provider, changes)).await
}

#[tauri::command]
pub async fn remove_env_var(name: String, scope: Scope) -> Result<EnvDiff, EnvError> {
    if name.trim().is_empty() {
        return Err(EnvError::parse("变量名不能为空"));
    }
    let change = Change::Remove(EnvVar::new(name, "", scope));
    with_provider(move |provider| apply_changes_impl(provider, vec![change])).await
}

#[tauri::command]
pub async fn check_permission(scope: Scope) -> Result<bool, EnvError> {
    with_provider(move |provider| provider.check_permission(scope)).await
}

#[tauri::command]
pub async fn create_backup(scope: Scope) -> Result<String, EnvError> {
    with_provider(move |provider| {
        provider
            .backup(scope)
            .map(|path| path.to_string_lossy().into_owned())
    })
    .await
}

#[tauri::command]
pub async fn list_backups() -> Result<Vec<BackupInfo>, EnvError> {
    tauri::async_runtime::spawn_blocking(list_backups_sync)
        .await
        .map_err(|error| EnvError::io(format!("后台任务执行失败：{error}")))?
}

#[tauri::command]
pub async fn restore_backup(path: String) -> Result<(), EnvError> {
    with_provider(move |provider| restore_backup_with_provider(provider, path)).await
}

#[tauri::command]
pub async fn preview_backup_restore(path: String) -> Result<EnvDiff, EnvError> {
    with_provider(move |provider| preview_backup_restore_with_provider(provider, path)).await
}

#[tauri::command]
pub async fn parse_path_var(value: String) -> Result<Vec<PathEntry>, EnvError> {
    Ok(parse_path(&value, path_separator(), !cfg!(windows)))
}

#[tauri::command]
pub async fn join_path_var(entries: Vec<PathEntry>) -> Result<String, EnvError> {
    Ok(join_path(&entries, path_separator()))
}

#[tauri::command]
pub async fn restart_as_admin() -> Result<(), EnvError> {
    tauri::async_runtime::spawn_blocking(|| {
        #[cfg(windows)]
        {
            let executable = env::current_exe()
                .map_err(|error| EnvError::io(format!("无法确定应用路径：{error}")))?;
            let verb = to_wide("runas");
            let file = to_wide(executable.to_string_lossy());
            let directory = to_wide(
                executable
                    .parent()
                    .map(|path| path.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            );
            let result = unsafe {
                ShellExecuteW(
                    std::ptr::null_mut(),
                    verb.as_ptr(),
                    file.as_ptr(),
                    std::ptr::null(),
                    directory.as_ptr(),
                    SW_SHOWNORMAL,
                )
            };
            if result as isize <= 32 {
                return Err(EnvError::permission_denied("无法以管理员身份重启 VarMan"));
            }
            std::process::exit(0);
        }

        #[cfg(unix)]
        Err(EnvError::unsupported(
            "请使用 sudo 或 pkexec 重新启动 VarMan",
        ))
    })
    .await
    .map_err(|error| EnvError::io(format!("后台任务执行失败：{error}")))?
}

#[cfg(windows)]
fn to_wide(value: impl AsRef<str>) -> Vec<u16> {
    value
        .as_ref()
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        error::EnvError,
        model::{Change, EnvDiff, EnvVar, Scope},
        traits::EnvProvider,
    };
    use std::{
        collections::HashMap,
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            RwLock,
        },
    };

    #[derive(Default)]
    struct MemoryProvider {
        variables: RwLock<HashMap<(u8, String), EnvVar>>,
        backups: AtomicUsize,
    }

    fn scope_key(scope: Scope) -> u8 {
        match scope {
            Scope::User => 0,
            Scope::System => 1,
        }
    }

    fn change_scope(change: &Change) -> Scope {
        match change {
            Change::Add(variable) | Change::Modify(variable) | Change::Remove(variable) => {
                variable.scope
            }
        }
    }

    impl EnvProvider for MemoryProvider {
        fn list_vars(&self, scope: Scope) -> Result<Vec<EnvVar>, EnvError> {
            Ok(self
                .variables
                .read()
                .unwrap()
                .values()
                .filter(|variable| variable.scope == scope)
                .cloned()
                .collect())
        }

        fn get_var(&self, name: &str, scope: Scope) -> Result<Option<EnvVar>, EnvError> {
            Ok(self
                .variables
                .read()
                .unwrap()
                .get(&(scope_key(scope), name.to_owned()))
                .cloned())
        }

        fn set_var(&self, variable: &EnvVar) -> Result<EnvDiff, EnvError> {
            let old = self.get_var(&variable.name, variable.scope)?;
            self.variables.write().unwrap().insert(
                (scope_key(variable.scope), variable.name.clone()),
                variable.clone(),
            );
            let mut diff = EnvDiff::default();
            match old {
                Some(old) => diff.modified.push((old, variable.clone())),
                None => diff.added.push(variable.clone()),
            }
            Ok(diff)
        }

        fn remove_var(&self, name: &str, scope: Scope) -> Result<EnvDiff, EnvError> {
            let old = self
                .variables
                .write()
                .unwrap()
                .remove(&(scope_key(scope), name.to_owned()))
                .ok_or_else(|| EnvError::not_found(format!("变量不存在：{name}")))?;
            let mut diff = EnvDiff::default();
            diff.removed.push(old);
            Ok(diff)
        }

        fn check_permission(&self, _scope: Scope) -> Result<bool, EnvError> {
            Ok(true)
        }

        fn preview(&self, changes: &[Change]) -> Result<EnvDiff, EnvError> {
            let Some(scope) = changes.first().map(change_scope) else {
                return Ok(EnvDiff::default());
            };
            crate::core::diff::build_diff(&self.list_vars(scope)?, changes, !cfg!(windows))
        }

        fn backup(&self, _scope: Scope) -> Result<PathBuf, EnvError> {
            self.backups.fetch_add(1, Ordering::SeqCst);
            Ok(PathBuf::from("backup.json"))
        }
    }

    #[test]
    fn apply_changes_rejects_scope_mismatch() {
        let provider = MemoryProvider::default();
        let error = apply_changes_impl(
            &provider,
            vec![
                Change::Add(EnvVar::new("A", "1", Scope::User)),
                Change::Add(EnvVar::new("B", "2", Scope::System)),
            ],
        )
        .unwrap_err();
        assert_eq!(error.code(), "PARSE_ERROR");
    }

    #[test]
    fn apply_changes_previews_backups_and_applies() {
        let provider = MemoryProvider::default();
        provider.variables.write().unwrap().insert(
            (scope_key(Scope::User), "A".into()),
            EnvVar::new("A", "1", Scope::User),
        );
        let diff = apply_changes_impl(
            &provider,
            vec![
                Change::Modify(EnvVar::new("A", "2", Scope::User)),
                Change::Add(EnvVar::new("B", "3", Scope::User)),
            ],
        )
        .unwrap();

        assert_eq!(provider.backups.load(Ordering::SeqCst), 1);
        assert_eq!(diff.added.len(), 1);
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(
            provider.get_var("A", Scope::User).unwrap().unwrap().value,
            "2"
        );
        assert!(provider.get_var("B", Scope::User).unwrap().is_some());
    }

    #[test]
    fn restore_diff_detects_add_modify_remove_and_unchanged() {
        let unchanged = EnvVar::new("A", "1", Scope::User);
        let current_modified = EnvVar::new("B", "old", Scope::User);
        let restored_modified = EnvVar::new("B", "new", Scope::User);
        let restored_added = EnvVar::new("C", "3", Scope::User);
        let current_removed = EnvVar::new("D", "4", Scope::User);
        let current = vec![unchanged.clone(), current_modified, current_removed.clone()];
        let backup = vec![unchanged, restored_modified.clone(), restored_added.clone()];

        let diff = build_restore_diff(&current, &backup, !cfg!(windows)).unwrap();

        assert_eq!(diff.added, vec![restored_added]);
        assert_eq!(diff.modified.len(), 1);
        assert_eq!(diff.modified[0].1.value, "new");
        assert_eq!(diff.removed, vec![current_removed]);
    }
}
