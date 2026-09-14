#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_path_marks_duplicates_and_joins_entries() {
        let entries = parse_path("/bin:/missing:/bin", ':', true);
        assert_eq!(entries[0].is_duplicate, true);
        assert_eq!(entries[1].exists, false);
        assert_eq!(join_path(&entries, ':'), "/bin:/missing:/bin");
    }

    #[test]
    fn windows_duplicates_ignore_case() {
        let entries = parse_path("C:\\Bin;c:\\bin", ';', false);
        assert!(entries.iter().all(|entry| entry.is_duplicate));
    }
}

use crate::core::model::PathEntry;
use std::{collections::HashMap, path::Path};

pub fn parse_path(value: &str, separator: char, case_sensitive: bool) -> Vec<PathEntry> {
    let mut counts: HashMap<String, usize> = HashMap::new();
    let paths: Vec<&str> = value
        .split(separator)
        .filter(|path| !path.is_empty())
        .collect();

    for path in &paths {
        let key = path_key(path, case_sensitive);
        *counts.entry(key).or_default() += 1;
    }

    paths
        .into_iter()
        .map(|path| {
            let key = path_key(path, case_sensitive);
            PathEntry {
                path: path.to_owned(),
                exists: Path::new(path).exists(),
                is_duplicate: counts[&key] > 1,
            }
        })
        .collect()
}

pub fn join_path(entries: &[PathEntry], separator: char) -> String {
    entries
        .iter()
        .map(|entry| entry.path.as_str())
        .collect::<Vec<_>>()
        .join(&separator.to_string())
}

fn path_key(path: &str, case_sensitive: bool) -> String {
    if case_sensitive {
        path.to_owned()
    } else {
        path.to_lowercase()
    }
}
