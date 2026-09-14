use crate::core::{
    diff::build_diff,
    error::EnvError,
    model::{BackupPayload, Change, EnvDiff, EnvVar, Scope},
    traits::EnvProvider,
};
use chrono::Local;
use std::{env, fs, path::PathBuf};
use windows_registry::{Key, Type, CURRENT_USER, LOCAL_MACHINE};
use windows_sys::Win32::System::{Environment::ExpandEnvironmentStringsW, Registry::KEY_SET_VALUE};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
};

const ENVIRONMENT_PATH: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

pub struct WindowsProvider;

impl WindowsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for WindowsProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvProvider for WindowsProvider {
    fn list_vars(&self, scope: Scope) -> Result<Vec<EnvVar>, EnvError> {
        let key = open_key(scope, false)?;
        let mut variables = Vec::new();

        for (name, value) in key.values().map_err(registry_error)? {
            if name.is_empty() {
                continue;
            }
            let is_expandable = value.ty() == Type::ExpandString;
            let raw_value = String::try_from(value)
                .map_err(|error| EnvError::parse(format!("注册表值无效：{error}")))?;
            let value = if is_expandable {
                expand_environment_strings(&raw_value)?
            } else {
                raw_value.clone()
            };
            variables.push(EnvVar {
                name,
                value,
                scope,
                is_expandable,
                raw_value: if is_expandable {
                    Some(raw_value.clone())
                } else {
                    None
                },
                source: None,
            });
        }

        variables.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(variables)
    }

    fn get_var(&self, name: &str, scope: Scope) -> Result<Option<EnvVar>, EnvError> {
        Ok(self
            .list_vars(scope)?
            .into_iter()
            .find(|variable| variable.name.eq_ignore_ascii_case(name)))
    }

    fn set_var(&self, variable: &EnvVar) -> Result<EnvDiff, EnvError> {
        if variable.name.trim().is_empty() {
            return Err(EnvError::parse("变量名不能为空"));
        }

        let old_variable = self.get_var(&variable.name, variable.scope)?;
        if !self.check_permission(variable.scope)? {
            return Err(EnvError::permission_denied("没有权限写入系统环境变量"));
        }

        let key = open_key(variable.scope, true)?;
        let keep_expandable =
            old_variable.as_ref().is_some_and(|old| old.is_expandable) || variable.is_expandable;
        if keep_expandable {
            let value = raw_value_to_write(variable)?;
            key.set_expand_string(&variable.name, value.as_str())
                .map_err(registry_error)?;
        } else {
            key.set_string(&variable.name, &variable.value)
                .map_err(registry_error)?;
        }
        broadcast_environment_change();

        let mut diff = EnvDiff::default();
        match old_variable {
            Some(old) => diff.modified.push((old, variable.clone())),
            None => diff.added.push(variable.clone()),
        }
        Ok(diff)
    }

    fn remove_var(&self, name: &str, scope: Scope) -> Result<EnvDiff, EnvError> {
        let old_variable = self
            .get_var(name, scope)?
            .ok_or_else(|| EnvError::not_found(format!("变量不存在：{name}")))?;
        if !self.check_permission(scope)? {
            return Err(EnvError::permission_denied("没有权限删除系统环境变量"));
        }

        open_key(scope, true)?
            .remove_value(name)
            .map_err(registry_error)?;
        broadcast_environment_change();

        let mut diff = EnvDiff::default();
        diff.removed.push(old_variable);
        Ok(diff)
    }

    fn check_permission(&self, scope: Scope) -> Result<bool, EnvError> {
        Ok(open_key(scope, true).is_ok())
    }

    fn preview(&self, changes: &[Change]) -> Result<EnvDiff, EnvError> {
        let Some(scope) = changes.first().map(change_scope) else {
            return Ok(EnvDiff::default());
        };
        let current = self.list_vars(scope)?;
        build_diff(&current, changes, false)
    }

    fn backup(&self, scope: Scope) -> Result<PathBuf, EnvError> {
        let directory = backup_directory()?;
        fs::create_dir_all(&directory)?;
        let timestamp = Local::now().format("%Y%m%d-%H%M%S%3f");
        let path = directory.join(format!("{}-{timestamp}.json", scope_name(scope)));
        let payload = BackupPayload {
            scope,
            variables: self.list_vars(scope)?,
        };
        fs::write(
            &path,
            serde_json::to_vec_pretty(&payload).map_err(|error| EnvError::io(error.to_string()))?,
        )?;
        Ok(path)
    }
}

fn open_key(scope: Scope, write: bool) -> Result<Key, EnvError> {
    let root = match scope {
        Scope::User => &CURRENT_USER,
        Scope::System => &LOCAL_MACHINE,
    };
    let result = if write {
        root.options()
            .read()
            .access(KEY_SET_VALUE)
            .open(registry_path(scope))
    } else {
        root.options().read().open(registry_path(scope))
    };

    result.map_err(|error| {
        if write && scope == Scope::System {
            EnvError::permission_denied(format!("没有权限访问系统环境变量：{error}"))
        } else {
            EnvError::io(format!("无法访问注册表：{error}"))
        }
    })
}

fn registry_path(scope: Scope) -> &'static str {
    match scope {
        Scope::User => "Environment",
        Scope::System => ENVIRONMENT_PATH,
    }
}

fn change_scope(change: &Change) -> Scope {
    match change {
        Change::Add(variable) | Change::Modify(variable) | Change::Remove(variable) => {
            variable.scope
        }
    }
}

fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::User => "user",
        Scope::System => "system",
    }
}

fn backup_directory() -> Result<PathBuf, EnvError> {
    let app_data = env::var_os("APPDATA")
        .map(PathBuf::from)
        .ok_or_else(|| EnvError::io("无法确定 APPDATA 目录"))?;
    Ok(app_data.join("VarMan").join("backups"))
}

fn expand_environment_strings(value: &str) -> Result<String, EnvError> {
    let source = value
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut buffer = vec![0_u16; source.len().max(1024)];

    loop {
        let required = unsafe {
            ExpandEnvironmentStringsW(source.as_ptr(), buffer.as_mut_ptr(), buffer.len() as u32)
        };
        if required == 0 {
            return Err(EnvError::io("展开环境变量引用失败"));
        }
        if required as usize <= buffer.len() {
            return Ok(String::from_utf16_lossy(&buffer[..required as usize - 1]));
        }
        buffer.resize(required as usize, 0);
    }
}

fn broadcast_environment_change() {
    let parameter = "Environment"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();
    let mut send_result = 0_usize;
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            parameter.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            1000,
            &mut send_result,
        );
    }
}

fn registry_error(error: windows_result::Error) -> EnvError {
    EnvError::io(format!("注册表操作失败：{error}"))
}

fn raw_value_to_write(variable: &EnvVar) -> Result<String, EnvError> {
    if !variable.is_expandable {
        return Ok(variable.value.clone());
    }
    if let Some(raw_value) = &variable.raw_value {
        if expand_environment_strings(raw_value)? == variable.value {
            return Ok(raw_value.clone());
        }
    }
    Ok(variable.value.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expandable_write_preserves_unchanged_raw_reference() {
        let mut variable = EnvVar::new("PATH", "C:\\Windows", Scope::User);
        variable.is_expandable = true;
        variable.raw_value = Some("%SystemRoot%".into());

        let value = raw_value_to_write(&variable).unwrap();

        assert_eq!(value, "%SystemRoot%");
    }
}
