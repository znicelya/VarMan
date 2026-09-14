# VarMan Core Design

## Goal

Build a cross-platform Tauri 2 desktop application for viewing, editing, removing, and backing up user and system environment variables with diff-first writes and a visual Path editor.

## Scope

This iteration delivers a runnable core:

- Windows registry provider for user and system variables.
- Linux/macOS shell configuration provider.
- Unified Tauri command API.
- Automatic backups before writes.
- React UI with shadcn/ui components, Zustand state, diff confirmation, Path editor, permission handling, and toast errors.
- Windows-focused packaging plus conditional Unix implementations and GitHub Actions workflow.

## Architecture

The frontend never touches system APIs. React components dispatch typed Tauri commands; Rust owns all environment-variable access and mutation.

Rust is layered as:

- `core`: models, errors, provider trait, pure path/diff/config-file logic.
- `platform`: `windows.rs` and `unix.rs` implementations selected by `#[cfg]`.
- `commands`: thin async Tauri command wrappers over one process-wide provider.

The provider API is synchronous. Tauri commands wrap calls in `tauri::async_runtime::spawn_blocking`, keeping registry and file I/O off the UI thread.

## Data and Error Model

- `Scope` has `User` and `System`; Display outputs `用户` and `系统`.
- `EnvVar` contains `name`, `value`, `scope`, `is_expandable`, and optional `source`.
- `PathEntry` contains `path`, `exists`, and `is_duplicate`.
- `Change` is `Add(EnvVar)`, `Modify(EnvVar)`, or `Remove(EnvVar)`.
- `EnvDiff` contains `added`, `modified`, and `removed`.
- `BackupInfo` contains `path`, `scope`, and `created_at`.
- `EnvError` serializes as `{ code, message }` with stable codes such as `PERMISSION_DENIED`, `NOT_FOUND`, `PARSE_ERROR`, `IO_ERROR`, and `UNSUPPORTED`.

## Write Flow

Every mutation starts with `preview_changes`. The UI shows a red/green diff, then calls `apply_changes`. Rust repeats the preview, checks permission, creates a backup, applies changes one by one, and returns the actual diff.

## Platform Behavior

### Windows

- User variables use `HKEY_CURRENT_USER\Environment`.
- System variables use `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment`.
- `REG_EXPAND_SZ` values are read expanded for display and retain the type when written.
- New values are `REG_SZ`; changing a `REG_SZ` value to one containing `%` keeps `REG_SZ` unless the user explicitly enables expandable mode.
- Successful writes broadcast `WM_SETTINGCHANGE`.
- Backups export registry values as JSON to `%APPDATA%\VarMan\backups`.

### Linux/macOS

- User files are selected from `.zshrc`, `.bashrc`, `.bash_profile`, and `.profile`.
- System files use `/etc/environment` and `/etc/profile`.
- Shell assignments support `export NAME=value` and `NAME=value`, ignoring comments and blank lines.
- Writes replace an existing assignment or append a quoted `export NAME="value"`.
- Writes use a temporary sibling file and atomic rename.
- Backups are copied to `~/.VarMan/backups`.
- System writes require effective UID 0.

## UI

The layout has a left scope sidebar, top search and actions area, main variable table, and bottom status bar. shadcn/ui supplies dialogs, sheets, buttons, badges, alerts, tables, inputs, and Sonner toasts. Path entries use drag-and-drop ordering, existence warnings, duplicate warnings, and a live joined-value preview.

## Verification

- Rust tests cover models, errors, path parsing/joining, diff generation, Unix config parsing/editing, and command-layer flows using a temporary provider.
- `cargo fmt`, `cargo test`, `pnpm build`, and `pnpm tauri build` verify the implementation.

