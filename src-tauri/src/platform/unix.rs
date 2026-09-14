use crate::core::{
    diff::build_diff,
    error::EnvError,
    model::{Change, EnvDiff, EnvVar, Scope},
    traits::EnvProvider,
    unix_config::{atomic_write, backup_files, parse_config, read_config_contents, render_change},
};
use chrono::Local;
use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

pub struct UnixProvider;

impl UnixProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UnixProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvProvider for UnixProvider {
    fn list_vars(&self, scope: Scope) -> Result<Vec<EnvVar>, EnvError> {
        let mut positions = HashMap::new();
        let mut variables = Vec::new();

        for path in candidate_paths(scope) {
            if !path.exists() {
                continue;
            }
            let source = source_label(&path, scope);
            let contents = read_config_contents(&path)?;
            for variable in parse_config(&contents, scope, &source)? {
                if let Some(&index) = positions.get(&variable.name) {
                    variables[index] = variable;
                } else {
                    positions.insert(variable.name.clone(), variables.len());
                    variables.push(variable);
                }
            }
        }

        variables.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(variables)
    }

    fn get_var(&self, name: &str, scope: Scope) -> Result<Option<EnvVar>, EnvError> {
        Ok(self
            .list_vars(scope)?
            .into_iter()
            .find(|variable| variable.name == name))
    }

    fn set_var(&self, variable: &EnvVar) -> Result<EnvDiff, EnvError> {
        if variable.name.trim().is_empty() {
            return Err(EnvError::parse("变量名不能为空"));
        }
        if !self.check_permission(variable.scope)? {
            return Err(EnvError::permission_denied(
                "需要 root 权限修改系统环境变量",
            ));
        }

        let old_variable = self.get_var(&variable.name, variable.scope)?;
        let path = self
            .target_path(&variable.name, variable.scope)?
            .ok_or_else(|| EnvError::not_found("没有找到可用的 shell 配置文件"))?;
        let contents = read_config_contents(&path)?;
        let updated = render_change(&contents, variable, false)?;
        atomic_write(&path, &updated)?;

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
            return Err(EnvError::permission_denied(
                "需要 root 权限删除系统环境变量",
            ));
        }

        let path = self
            .target_path(name, scope)?
            .ok_or_else(|| EnvError::not_found(format!("变量不存在：{name}")))?;
        let contents = read_config_contents(&path)?;
        let updated = render_change(&contents, &EnvVar::new(name, "", scope), true)?;
        atomic_write(&path, &updated)?;

        let mut diff = EnvDiff::default();
        diff.removed.push(old_variable);
        Ok(diff)
    }

    fn check_permission(&self, scope: Scope) -> Result<bool, EnvError> {
        Ok(match scope {
            Scope::User => true,
            Scope::System => unsafe { libc::geteuid() == 0 },
        })
    }

    fn preview(&self, changes: &[Change]) -> Result<EnvDiff, EnvError> {
        let Some(scope) = changes.first().map(change_scope) else {
            return Ok(EnvDiff::default());
        };
        let current = self.list_vars(scope)?;
        build_diff(&current, changes, true)
    }

    fn backup(&self, scope: Scope) -> Result<PathBuf, EnvError> {
        let directory = home_directory()?.join(".VarMan").join("backups");
        let timestamp = Local::now().format("%Y%m%d-%H%M%S%3f").to_string();
        backup_files(&candidate_paths(scope), &directory, &timestamp)?
            .into_iter()
            .next()
            .ok_or_else(|| EnvError::not_found("没有找到可用的 shell 配置文件"))
    }
}

impl UnixProvider {
    fn target_path(&self, name: &str, scope: Scope) -> Result<Option<PathBuf>, EnvError> {
        for path in candidate_paths(scope) {
            if !path.exists() {
                continue;
            }
            let contents = read_config_contents(&path)?;
            if parse_config(&contents, scope, &source_label(&path, scope))?
                .iter()
                .any(|variable| variable.name == name)
            {
                return Ok(Some(path));
            }
        }
        Ok(candidate_paths(scope)
            .into_iter()
            .find(|path| path.exists()))
    }
}

fn candidate_paths(scope: Scope) -> Vec<PathBuf> {
    match scope {
        Scope::User => {
            let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
                return Vec::new();
            };
            let mut paths = Vec::new();
            let zshrc = home.join(".zshrc");
            if zshrc.exists() {
                paths.push(zshrc.clone());
            }

            let defaults = vec![
                home.join(".bashrc"),
                home.join(".bash_profile"),
                home.join(".profile"),
            ];

            for path in defaults {
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
            paths
        }
        Scope::System => vec![
            PathBuf::from("/etc/environment"),
            PathBuf::from("/etc/profile"),
        ],
    }
}

fn home_directory() -> Result<PathBuf, EnvError> {
    env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| EnvError::io("无法确定用户主目录"))
}

fn source_label(path: &Path, scope: Scope) -> String {
    if scope == Scope::System {
        path.display().to_string()
    } else if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
        path.strip_prefix(&home)
            .map(|relative| format!("~/{}", relative.display()))
            .unwrap_or_else(|_| path.display().to_string())
    } else {
        path.display().to_string()
    }
}

fn change_scope(change: &Change) -> Scope {
    match change {
        Change::Add(variable) | Change::Modify(variable) | Change::Remove(variable) => {
            variable.scope
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backup_copies_all_existing_candidate_files() {
        let home = tempfile::tempdir().unwrap();
        let zshrc = home.path().join(".zshrc");
        let profile = home.path().join(".profile");
        std::fs::write(&zshrc, "FIRST=1").unwrap();
        std::fs::write(&profile, "SECOND=2").unwrap();

        let backups = backup_files(
            &[zshrc, profile],
            &home.path().join("backup-directory"),
            "20260914-000000000",
        )
        .unwrap();

        assert_eq!(backups.len(), 2);
    }
}
