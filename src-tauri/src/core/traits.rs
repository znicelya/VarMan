use crate::core::{
    error::EnvError,
    model::{Change, EnvDiff, EnvVar, Scope},
};
use std::path::PathBuf;

pub trait EnvProvider: Send + Sync {
    fn list_vars(&self, scope: Scope) -> Result<Vec<EnvVar>, EnvError>;

    fn get_var(&self, name: &str, scope: Scope) -> Result<Option<EnvVar>, EnvError>;

    fn set_var(&self, var: &EnvVar) -> Result<EnvDiff, EnvError>;

    fn remove_var(&self, name: &str, scope: Scope) -> Result<EnvDiff, EnvError>;

    fn check_permission(&self, scope: Scope) -> Result<bool, EnvError>;

    fn preview(&self, changes: &[Change]) -> Result<EnvDiff, EnvError>;

    fn backup(&self, scope: Scope) -> Result<PathBuf, EnvError>;
}
