#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{
        error::EnvError,
        model::{Change, EnvVar, Scope},
    };

    #[test]
    fn build_diff_classifies_add_modify_and_remove() {
        let old = EnvVar::new("A", "1", Scope::User);
        let modified = EnvVar::new("A", "2", Scope::User);
        let added = EnvVar::new("B", "3", Scope::User);
        let diff = build_diff(
            &[old.clone()],
            &[Change::Modify(modified.clone()), Change::Add(added.clone())],
            true,
        )
        .unwrap();
        assert_eq!(diff.added, vec![added]);
        assert_eq!(diff.modified[0].0, old);
        assert_eq!(diff.modified[0].1, modified);
    }

    #[test]
    fn build_diff_rejects_modifying_missing_variable() {
        let error = build_diff(
            &[],
            &[Change::Modify(EnvVar::new("A", "1", Scope::User))],
            true,
        )
        .unwrap_err();
        assert_eq!(error.code(), "NOT_FOUND");
    }

    #[test]
    fn build_diff_rejects_empty_names_and_scope_mismatch() {
        let error =
            build_diff(&[], &[Change::Add(EnvVar::new("", "1", Scope::User))], true).unwrap_err();
        assert_eq!(error.code(), "PARSE_ERROR");

        let error = build_diff(
            &[],
            &[
                Change::Add(EnvVar::new("A", "1", Scope::User)),
                Change::Add(EnvVar::new("B", "2", Scope::System)),
            ],
            true,
        )
        .unwrap_err();
        assert_eq!(error, EnvError::parse("所有变更必须属于同一个作用域"));
    }

    #[test]
    fn build_diff_can_match_variable_names_case_insensitively() {
        let old = EnvVar::new("PATH", "/bin", Scope::User);
        let modified = EnvVar::new("path", "/usr/bin", Scope::User);
        let diff = build_diff(&[old.clone()], &[Change::Modify(modified.clone())], false).unwrap();

        assert_eq!(diff.modified, vec![(old, modified)]);
    }
}

use crate::core::{
    error::EnvError,
    model::{Change, EnvDiff, EnvVar, Scope},
};
use std::collections::{HashMap, HashSet};

pub fn build_diff(
    current: &[EnvVar],
    changes: &[Change],
    case_sensitive_names: bool,
) -> Result<EnvDiff, EnvError> {
    let Some(scope) = changes.first().map(change_scope) else {
        return Ok(EnvDiff::default());
    };

    if current
        .iter()
        .any(|variable| variable.scope != scope || variable.name.trim().is_empty())
    {
        return Err(EnvError::parse("当前变量列表包含无效的作用域或变量名"));
    }

    let mut old_vars: HashMap<String, &EnvVar> = HashMap::new();
    for variable in current {
        if variable.scope != scope {
            return Err(EnvError::parse("所有变更必须属于同一个作用域"));
        }
        let name_key = variable_name_key(&variable.name, case_sensitive_names);
        if old_vars.insert(name_key, variable).is_some() {
            return Err(EnvError::parse(format!("变量重复：{}", variable.name)));
        }
    }

    let mut changed_names: HashSet<String> = HashSet::new();
    let mut diff = EnvDiff::default();

    for change in changes {
        let variable = change_var(change);
        if variable.name.trim().is_empty() {
            return Err(EnvError::parse("变量名不能为空"));
        }
        if variable.scope != scope {
            return Err(EnvError::parse("所有变更必须属于同一个作用域"));
        }
        let name_key = variable_name_key(&variable.name, case_sensitive_names);
        if !changed_names.insert(name_key) {
            return Err(EnvError::parse(format!(
                "同一变量只能变更一次：{}",
                variable.name
            )));
        }

        match change {
            Change::Add(new_var) => {
                let name_key = variable_name_key(&new_var.name, case_sensitive_names);
                if old_vars.contains_key(&name_key) {
                    return Err(EnvError::parse(format!("变量已存在：{}", new_var.name)));
                }
                diff.added.push(new_var.clone());
            }
            Change::Modify(new_var) => {
                let name_key = variable_name_key(&new_var.name, case_sensitive_names);
                let old_var = old_vars
                    .get(&name_key)
                    .ok_or_else(|| EnvError::not_found(format!("变量不存在：{}", new_var.name)))?;
                diff.modified.push(((*old_var).clone(), new_var.clone()));
            }
            Change::Remove(_) => {
                let name_key = variable_name_key(&variable.name, case_sensitive_names);
                let old_var = old_vars
                    .get(&name_key)
                    .ok_or_else(|| EnvError::not_found(format!("变量不存在：{}", variable.name)))?;
                diff.removed.push((*old_var).clone());
            }
        }
    }

    Ok(diff)
}

fn change_scope(change: &Change) -> Scope {
    change_var(change).scope
}

fn change_var(change: &Change) -> &EnvVar {
    match change {
        Change::Add(variable) | Change::Modify(variable) | Change::Remove(variable) => variable,
    }
}

fn variable_name_key(name: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        name.to_owned()
    } else {
        name.to_lowercase()
    }
}
