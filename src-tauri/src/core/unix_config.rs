#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::model::{EnvVar, Scope};

    #[cfg(unix)]
    #[test]
    fn atomic_write_preserves_file_mode() {
        use std::os::unix::fs::PermissionsExt;

        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o640)).unwrap();

        atomic_write(&path, "new").unwrap();

        let mode = std::fs::metadata(&path).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o640);
    }

    #[test]
    fn backup_files_copies_every_existing_source() {
        let source_directory = tempfile::tempdir().unwrap();
        let first = source_directory.path().join(".zshrc");
        let second = source_directory.path().join(".profile");
        let missing = source_directory.path().join(".bashrc");
        std::fs::write(&first, "FIRST=1").unwrap();
        std::fs::write(&second, "SECOND=2").unwrap();

        let backup_directory = source_directory.path().join("backups");
        let backups = backup_files(
            &[first.clone(), missing, second.clone()],
            &backup_directory,
            "20260914-000000000",
        )
        .unwrap();

        assert_eq!(backups.len(), 2);
        assert_eq!(std::fs::read_to_string(&backups[0]).unwrap(), "FIRST=1");
        assert_eq!(std::fs::read_to_string(&backups[1]).unwrap(), "SECOND=2");
    }

    #[test]
    fn read_config_contents_rejects_invalid_utf8() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config");
        std::fs::write(&path, [0xff, 0xfe]).unwrap();

        let error = read_config_contents(&path).unwrap_err();

        assert_eq!(error.code(), "IO_ERROR");
    }

    #[test]
    fn parse_config_reads_assignments_and_ignores_comments() {
        let vars = parse_config(
            "# ignored\n\nFOO=1\nexport BAR=\"two words\"\n",
            Scope::User,
            "~/.zshrc",
        )
        .unwrap();
        assert_eq!(vars.len(), 2);
        assert_eq!(vars[0].value, "1");
        assert_eq!(vars[1].value, "two words");
        assert_eq!(vars[1].source.as_deref(), Some("~/.zshrc"));
    }

    #[test]
    fn render_change_replaces_existing_assignment_and_quotes_values() {
        let output = render_change(
            "FOO=old\nBAR=2\n",
            &EnvVar::new("FOO", "new value", Scope::User),
            false,
        )
        .unwrap();
        assert!(output.contains("export FOO=\"new value\""));
        assert!(output.contains("BAR=2"));
    }

    #[test]
    fn render_change_rejects_invalid_variable_name() {
        let error =
            render_change("", &EnvVar::new("BAD NAME", "1", Scope::User), false).unwrap_err();

        assert_eq!(error.code(), "PARSE_ERROR");
    }

    #[test]
    fn render_change_preserves_dynamic_existing_assignment() {
        let contents = "export PATH=\"$HOME/bin:$PATH\"\n";
        let mut variable = EnvVar::new("PATH", "$HOME/bin:$PATH", Scope::User);
        variable.raw_value = Some("\"$HOME/bin:$PATH\"".into());
        let output = render_change(contents, &variable, false).unwrap();

        assert_eq!(output, contents);
    }

    #[test]
    fn atomic_write_replaces_file_contents() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("config");
        std::fs::write(&path, "old").unwrap();
        atomic_write(&path, "new").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }
}

use crate::core::{
    error::EnvError,
    model::{EnvVar, Scope},
};
use std::{
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub fn parse_config(contents: &str, scope: Scope, source: &str) -> Result<Vec<EnvVar>, EnvError> {
    let mut positions: HashMap<String, usize> = HashMap::new();
    let mut variables = Vec::new();

    for line in contents.lines() {
        let Some(assignment) = parse_assignment(line) else {
            continue;
        };
        let variable = EnvVar {
            name: assignment.name,
            value: assignment.value,
            scope,
            is_expandable: false,
            raw_value: Some(assignment.raw_value),
            source: Some(source.to_owned()),
        };

        if let Some(&index) = positions.get(&variable.name) {
            variables[index] = variable;
        } else {
            positions.insert(variable.name.clone(), variables.len());
            variables.push(variable);
        }
    }

    Ok(variables)
}

pub fn render_change(contents: &str, variable: &EnvVar, remove: bool) -> Result<String, EnvError> {
    if !is_valid_name(&variable.name) {
        return Err(EnvError::parse("变量名无效"));
    }

    let assignment = format_assignment(variable);
    let mut output_lines = Vec::new();
    let mut replaced = false;

    for line in contents.lines() {
        if let Some(parsed) = parse_assignment(line) {
            if parsed.name == variable.name {
                if remove || replaced {
                    continue;
                }
                if variable.raw_value.as_deref() == Some(parsed.raw_value.as_str())
                    && parsed.value == variable.value
                {
                    output_lines.push(line.to_owned());
                } else {
                    output_lines.push(assignment.clone());
                }
                replaced = true;
                continue;
            }
        }
        output_lines.push(line.to_owned());
    }

    let appended = !replaced && !remove;
    if appended {
        output_lines.push(assignment);
    }

    let mut output = output_lines.join("\n");
    if appended || contents.ends_with('\n') {
        output.push('\n');
    }
    Ok(output)
}

pub fn atomic_write(path: &Path, contents: &str) -> Result<(), EnvError> {
    let parent = path
        .parent()
        .ok_or_else(|| EnvError::io(format!("无效的配置文件路径：{}", path.display())))?;
    let existing_permissions = match fs::metadata(path) {
        Ok(metadata) => Some(metadata.permissions()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => return Err(error.into()),
    };
    let mut temporary = tempfile::NamedTempFile::new_in(parent)?;
    temporary.write_all(contents.as_bytes())?;
    if let Some(permissions) = existing_permissions {
        temporary.as_file().set_permissions(permissions)?;
    }
    temporary
        .persist(path)
        .map_err(|error| EnvError::io(error.to_string()))?;
    Ok(())
}

pub fn backup_files(
    sources: &[PathBuf],
    directory: &Path,
    timestamp: &str,
) -> Result<Vec<PathBuf>, EnvError> {
    let sources = sources
        .iter()
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    if sources.is_empty() {
        return Ok(Vec::new());
    }

    fs::create_dir_all(directory)?;
    let mut backups = Vec::with_capacity(sources.len());
    for source in sources {
        let file_name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| EnvError::io("配置文件名无效"))?;
        let target = directory.join(format!("{file_name}.{timestamp}.bak"));
        fs::copy(source, &target)?;
        backups.push(target);
    }
    Ok(backups)
}

pub fn read_config_contents(path: &Path) -> Result<String, EnvError> {
    fs::read_to_string(path)
        .map_err(|error| EnvError::io(format!("无法读取配置文件 {}: {error}", path.display())))
}

struct Assignment {
    name: String,
    value: String,
    raw_value: String,
}

fn parse_assignment(line: &str) -> Option<Assignment> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    let body = trimmed
        .strip_prefix("export ")
        .map(str::trim)
        .unwrap_or(trimmed);
    let separator = body.find('=')?;
    let name = body[..separator].trim();
    if !is_valid_name(name) {
        return None;
    }

    Some(Assignment {
        name: name.to_owned(),
        value: parse_shell_value(body[separator + 1..].trim()),
        raw_value: body[separator + 1..].trim().to_owned(),
    })
}

fn parse_shell_value(raw: &str) -> String {
    if raw.len() >= 2 && raw.starts_with('"') && raw.ends_with('"') {
        unescape_double_quoted(&raw[1..raw.len() - 1])
    } else if raw.len() >= 2 && raw.starts_with('\'') && raw.ends_with('\'') {
        raw[1..raw.len() - 1].to_owned()
    } else {
        raw.to_owned()
    }
}

fn unescape_double_quoted(value: &str) -> String {
    value
        .replace("\\\\", "\u{0}")
        .replace("\\\"", "\"")
        .replace("\\$", "$")
        .replace("\\`", "`")
        .replace('\u{0}', "\\")
}

fn format_assignment(variable: &EnvVar) -> String {
    let value = variable
        .value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('$', "\\$")
        .replace('`', "\\`");
    format!("export {}=\"{}\"", variable.name, value)
}

fn is_valid_name(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some('_' | 'a'..='z' | 'A'..='Z'))
        && characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}
