#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_displays_chinese_labels() {
        assert_eq!(Scope::User.to_string(), "用户");
        assert_eq!(Scope::System.to_string(), "系统");
    }

    #[test]
    fn env_var_serializes_camel_case_fields() {
        let value = EnvVar {
            name: "PATH".into(),
            value: "/bin".into(),
            scope: Scope::User,
            is_expandable: true,
            raw_value: Some("%SystemRoot%".into()),
            source: Some("~/.zshrc".into()),
        };
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["isExpandable"], true);
        assert_eq!(json["source"], "~/.zshrc");
        assert_eq!(json["rawValue"], "%SystemRoot%");
    }
}

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Scope {
    User,
    System,
}

impl fmt::Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::User => write!(formatter, "用户"),
            Self::System => write!(formatter, "系统"),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvVar {
    pub name: String,
    pub value: String,
    pub scope: Scope,
    pub is_expandable: bool,
    #[serde(default)]
    pub raw_value: Option<String>,
    pub source: Option<String>,
}

impl EnvVar {
    pub fn new(name: impl Into<String>, value: impl Into<String>, scope: Scope) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            scope,
            is_expandable: false,
            raw_value: None,
            source: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PathEntry {
    pub path: String,
    pub exists: bool,
    pub is_duplicate: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "type", content = "value")]
pub enum Change {
    Add(EnvVar),
    Modify(EnvVar),
    Remove(EnvVar),
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvDiff {
    pub added: Vec<EnvVar>,
    pub modified: Vec<(EnvVar, EnvVar)>,
    pub removed: Vec<EnvVar>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub path: String,
    pub scope: Scope,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupPayload {
    pub scope: Scope,
    pub variables: Vec<EnvVar>,
}
