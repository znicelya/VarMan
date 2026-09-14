# VarMan Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a diff-first, cross-platform environment-variable manager with Tauri 2, Rust, React, Tailwind, Zustand, and shadcn/ui.

**Architecture:** The UI calls typed Tauri commands only. Rust's `core` layer owns pure models and logic, `platform` owns OS-specific providers, and `commands` wraps one provider asynchronously. Writes always preview, check permission, backup, apply, and return a diff.

**Tech Stack:** Tauri 2.x, Rust, React 19, TypeScript, Vite, Tailwind CSS 4, Zustand, shadcn/ui, Sonner.

## Global Constraints

- Frontend must never call a system API directly.
- Every command returns `Result<T, EnvError>` where serialized errors contain `code` and `message`.
- System permission failures must return code `PERMISSION_DENIED`.
- Windows `REG_EXPAND_SZ` must be expanded on read and preserved on write when already expandable.
- Unix writes must use a temporary file and atomic rename.
- Unix writes must backup to `~/.VarMan/backups`; Windows backups go to `%APPDATA%\VarMan\backups`.
- Path separator is `;` on Windows and `:` on Unix.
- Windows duplicate Path entries compare case-insensitively; Unix compares case-sensitively.
- Use shadcn/ui components before custom markup.
- Do not commit changes; the repository has no initial commit and the user did not request commits.

---

### Task 1: Rust Core Models, Errors, and Provider Contract

**Files:**

- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/core/mod.rs`
- Create: `src-tauri/src/core/model.rs`
- Create: `src-tauri/src/core/error.rs`
- Create: `src-tauri/src/core/traits.rs`
- Test: `src-tauri/src/core/model.rs`
- Test: `src-tauri/src/core/error.rs`

**Interfaces:**

- Produces: `Scope`, `EnvVar`, `PathEntry`, `Change`, `EnvDiff`, `BackupInfo`, `EnvError`, `EnvProvider`.
- `EnvProvider::backup` returns `std::path::PathBuf`.

- [ ] **Step 1: Add failing model tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_displays_chinese_labels() {
        assert_eq!(Scope::User.to_string(), "用户");
        assert_eq!(Scope::System.to_string(), "系统");
    }

    #[test]
    fn env_var_serializes_snake_case_fields() {
        let value = EnvVar {
            name: "PATH".into(),
            value: "/bin".into(),
            scope: Scope::User,
            is_expandable: true,
            source: Some("~/.zshrc".into()),
        };
        let json = serde_json::to_value(&value).unwrap();
        assert_eq!(json["isExpandable"], true);
        assert_eq!(json["source"], "~/.zshrc");
    }
}
```

- [ ] **Step 2: Add failing error tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_error_serializes_code_and_message() {
        let error = EnvError::permission_denied("需要管理员权限");
        let json = serde_json::to_value(&error).unwrap();
        assert_eq!(json["code"], "PERMISSION_DENIED");
        assert_eq!(json["message"], "需要管理员权限");
    }
}
```

- [ ] **Step 3: Run tests and verify failure**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: compilation fails because `core` modules do not exist.

- [ ] **Step 4: Implement models and errors**

Add serde `rename_all = "camelCase"` to all serialized structs and enums. Implement constructors for stable error codes and `std::fmt::Display`.

- [ ] **Step 5: Run tests and verify pass**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

### Task 2: Pure Diff and Path Logic

**Files:**

- Create: `src-tauri/src/core/diff.rs`
- Create: `src-tauri/src/core/path.rs`
- Test: `src-tauri/src/core/diff.rs`
- Test: `src-tauri/src/core/path.rs`

**Interfaces:**

- Produces: `build_diff(current: &[EnvVar], changes: &[Change]) -> Result<EnvDiff, EnvError>`
- Produces: `parse_path(value: &str, separator: char, case_sensitive: bool) -> Vec<PathEntry>`
- Produces: `join_path(entries: &[PathEntry], separator: char) -> String`

- [ ] **Step 1: Write failing diff tests**

```rust
#[test]
fn build_diff_classifies_add_modify_and_remove() {
    let old = EnvVar::new("A", "1", Scope::User);
    let modified = EnvVar::new("A", "2", Scope::User);
    let added = EnvVar::new("B", "3", Scope::User);
    let diff = build_diff(
        &[old.clone()],
        &[Change::Modify(modified.clone()), Change::Add(added.clone())],
    ).unwrap();
    assert_eq!(diff.added, vec![added]);
    assert_eq!(diff.modified[0].0, old);
    assert_eq!(diff.modified[0].1, modified);
}
```

- [ ] **Step 2: Write failing path tests**

```rust
#[test]
fn parse_path_marks_duplicates_and_joins_entries() {
    let entries = parse_path("/bin:/missing:/bin", ':', true);
    assert_eq!(entries[0].is_duplicate, true);
    assert_eq!(entries[1].exists, false);
    assert_eq!(join_path(&entries, ':'), "/bin:/missing:/bin");
}
```

- [ ] **Step 3: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: missing functions cause compilation failure.

- [ ] **Step 4: Implement minimal pure logic**

Reject empty names, scope mismatch, and modifying/removing a missing variable with `EnvError`.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

### Task 3: Unix Configuration File Engine

**Files:**

- Create: `src-tauri/src/core/unix_config.rs`
- Test: `src-tauri/src/core/unix_config.rs`

**Interfaces:**

- Produces: `parse_config(contents: &str, scope: Scope, source: &str) -> Result<Vec<EnvVar>, EnvError>`
- Produces: `render_change(contents: &str, var: &EnvVar, remove: bool) -> Result<String, EnvError>`
- Produces: `atomic_write(path: &Path, contents: &str) -> Result<(), EnvError>`

- [ ] **Step 1: Write failing parser test**

```rust
#[test]
fn parse_config_reads_assignments_and_ignores_comments() {
    let vars = parse_config("# ignored\n\nFOO=1\nexport BAR=\"two words\"\n", Scope::User, "~/.zshrc").unwrap();
    assert_eq!(vars.len(), 2);
    assert_eq!(vars[1].value, "two words");
    assert_eq!(vars[1].source.as_deref(), Some("~/.zshrc"));
}
```

- [ ] **Step 2: Write failing writer test**

```rust
#[test]
fn render_change_replaces_existing_assignment_and_quotes_values() {
    let output = render_change("FOO=old\nBAR=2\n", &EnvVar::new("FOO", "new value", Scope::User), false).unwrap();
    assert!(output.contains("export FOO=\"new value\""));
    assert!(output.contains("BAR=2"));
}
```

- [ ] **Step 3: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: `unix_config` tests fail to compile.

- [ ] **Step 4: Implement parser, renderer, and atomic write**

Use `tempfile::NamedTempFile::new_in(path.parent()?)`, persist to the final path, and map failures to `IO_ERROR`.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

### Task 4: Platform Providers

**Files:**

- Create: `src-tauri/src/platform/mod.rs`
- Create: `src-tauri/src/platform/windows.rs`
- Create: `src-tauri/src/platform/unix.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**

- Consumes: `EnvProvider`, `build_diff`, `parse_path`, `join_path`, Unix config helpers.
- Produces: `pub fn create_provider() -> Box<dyn EnvProvider + Send + Sync>`

- [ ] **Step 1: Write permission and provider dispatch tests**

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn provider_dispatch_is_available() {
        let provider = crate::platform::create_provider();
        assert!(provider.check_permission(crate::core::model::Scope::User).unwrap());
    }
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: `platform` module is missing.

- [ ] **Step 3: Implement Windows provider**

Use `windows-registry` for registry reads/writes and Windows APIs for environment expansion and `WM_SETTINGCHANGE`. Export a backup as JSON. Return `PERMISSION_DENIED` when system registry write access is unavailable.

- [ ] **Step 4: Implement Unix provider**

Select existing candidate files, parse them, perform permission checks with `libc::geteuid`, backup before write, and use `atomic_write`.

- [ ] **Step 5: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

### Task 5: Tauri Commands and Restore

**Files:**

- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/commands/mod.rs`

**Interfaces:**

- Produces all commands named in `docs/prompt.txt`.
- `apply_changes` and `remove_env_var` preview, check permission, backup, apply, and return diff.
- `restore_backup` validates the backup path and restores values.

- [ ] **Step 1: Write a failing command test**

```rust
#[test]
fn apply_changes_rejects_scope_mismatch() {
    let error = apply_changes(vec![Change::Add(EnvVar::new("X", "1", Scope::System))]).unwrap_err();
    assert_eq!(error.code(), "PARSE_ERROR");
}
```

- [ ] **Step 2: Verify RED**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: commands do not exist.

- [ ] **Step 3: Implement commands**

Use a `OnceLock<Mutex<Box<dyn EnvProvider + Send + Sync>>>`, wrap blocking calls in `spawn_blocking`, and register every command in `generate_handler!`.

- [ ] **Step 4: Verify GREEN**

Run: `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: all tests pass.

### Task 6: shadcn/ui Initialization and Frontend State

**Files:**

- Create: `components.json`
- Modify: `package.json`
- Modify: `src/index.css`
- Create: `src/types/env.ts`
- Create: `src/lib/api.ts`
- Create: `src/stores/env-store.ts`
- Create: `src/components/ui/*`

**Interfaces:**

- Produces typed wrappers for every Tauri command.
- Produces Zustand state: `currentScope`, `envVars`, `pendingChanges`, `isLoading`, `lastError`, `backups`.

- [ ] **Step 1: Initialize shadcn**

Run: `pnpm dlx shadcn@latest init --preset base-nova`

Expected: `components.json` and semantic Tailwind tokens are created.

- [ ] **Step 2: Add components**

Run: `pnpm dlx shadcn@latest add alert badge button card dialog field input scroll-area separator sheet skeleton sonner table textarea tooltip`

Expected: shadcn components exist under `src/components/ui`.

- [ ] **Step 3: Add API tests**

Use TypeScript's type system to require every command wrapper. Then run:

`pnpm exec tsc --noEmit`

Expected: wrappers and shared types compile.

- [ ] **Step 4: Implement store and API**

Map errors to Chinese messages and expose actions for loading, switching scope, previewing, applying, removing, and refreshing backups.

- [ ] **Step 5: Verify frontend compile**

Run: `pnpm exec tsc --noEmit`

Expected: no TypeScript errors.

### Task 7: Application UI

**Files:**

- Modify: `src/App.tsx`
- Create: `src/components/Sidebar.tsx`
- Create: `src/components/Toolbar.tsx`
- Create: `src/components/VariableTable.tsx`
- Create: `src/components/VariableDialog.tsx`
- Create: `src/components/DiffDialog.tsx`
- Create: `src/components/PathEditor.tsx`
- Create: `src/components/BackupDialog.tsx`
- Create: `src/components/PermissionAlert.tsx`

**Interfaces:**

- Consumes Zustand store and shadcn components.
- Produces a complete list, edit, remove, diff, Path editor, backup, restore, and permission flow.

- [ ] **Step 1: Implement layout**

Use Sidebar, Toolbar, VariableTable, PermissionAlert, and status bar with semantic shadcn tokens.

- [ ] **Step 2: Implement edit and diff flow**

Open `VariableDialog`, call `previewChanges`, show `DiffDialog`, then call `applyChanges`.

- [ ] **Step 3: Implement Path editor**

Use a Sheet with drag-and-drop rows, delete/add controls, red missing badges, yellow duplicate badges, and live joined value.

- [ ] **Step 4: Implement backup flow**

List backups, create backups, and restore only after a confirmation dialog.

- [ ] **Step 5: Verify UI build**

Run: `pnpm build`

Expected: TypeScript and Vite production build pass.

### Task 8: Packaging and CI

**Files:**

- Modify: `src-tauri/tauri.conf.json`
- Create: `src-tauri/Info.plist`
- Create: `.github/workflows/build.yml`
- Create packaging metadata as required by Tauri.

**Interfaces:**

- Produces Windows MSI/NSIS bundles, macOS DMG configuration, and Linux deb/AppImage configuration.

- [ ] **Step 1: Configure app identity**

Set `productName: "VarMan"`, identifier `com.yourname.VarMan`, window size, and bundle targets.

- [ ] **Step 2: Configure platform packaging**

Use `asInvoker` for Windows, add macOS privacy strings for shell files, and configure deb/AppImage/DMG targets.

- [ ] **Step 3: Add GitHub Actions matrix**

Build on `windows-latest`, `ubuntu-latest`, and `macos-latest`; upload artifacts for each platform.

- [ ] **Step 4: Final verification**

Run:

```powershell
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
pnpm build
pnpm tauri build
```

Expected: all commands pass without warnings or errors.

