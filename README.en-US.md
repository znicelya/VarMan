# VarMan

English | [简体中文](./README.md)

VarMan is a cross-platform visual manager for system environment variables. It replaces the built-in editors on Windows, Linux, and macOS with a unified list view, diff preview, automatic backups, restore, and permission protection. VarMan is built with Tauri 2, React 19, TypeScript, Tailwind CSS 4, and shadcn/ui.

> The current version is `0.1.0`. Core workflows are implemented, but system variables should still be modified with care in production environments.

## Contents

- [Highlights](#highlights)
- [Platform Behavior](#platform-behavior)
- [Safety Model](#safety-model)
- [Backups and Restore](#backups-and-restore)
- [Requirements](#requirements)
- [Quick Start](#quick-start)
- [Useful Commands](#useful-commands)
- [Project Structure](#project-structure)
- [Development Notes](#development-notes)
- [Testing and Verification](#testing-and-verification)
- [Packaging](#packaging)
- [Continuous Integration](#continuous-integration)
- [FAQ](#faq)

## Highlights

### Variable management

- View user and system environment variables with real-time name filtering.
- Show each variable's source: the registry on Windows or a specific configuration file on Unix.
- Create, edit, and delete environment variables.
- Values containing `;` automatically switch to an entry list editor, where entries can be added, removed, and edited individually.
- Both single-value and list modes support manual input and native directory/file selection.
- Windows preserves `REG_EXPAND_SZ` references: expanded values are displayed, while original references are restored when possible during writes.

### Change preview

- Every write operation is preceded by a diff preview.
- The diff uses side-by-side Before/After columns.
- Semicolon-separated values are split into entries and compared line by line.
- Added, removed, and modified entries are marked in green, red, and per-entry diff styles.
- Unchanged entries remain neutral so actual changes stand out.

### Dedicated Path editor

- Path variables open in a dedicated drawer.
- Every path entry has its own row and supports drag-and-drop reordering.
- Missing directories and duplicate paths are marked automatically.
- The joined final Path value is previewed in real time.
- Entries can be removed individually or added manually.

### UI and language

- Light and dark theme support.
- Chinese and English UI with instant switching and persistent language selection.
- Read-only mode and privilege guidance are shown when permissions are insufficient.

## Platform Behavior

### Windows

| Item | Details |
| --- | --- |
| User variables | `HKEY_CURRENT_USER\Environment` |
| System variables | `HKEY_LOCAL_MACHINE\SYSTEM\CurrentControlSet\Control\Session Manager\Environment` |
| Variable types | `REG_SZ` and `REG_EXPAND_SZ` |
| Reference preservation | `REG_EXPAND_SZ` raw references are retained and restored on write |
| Write notification | Broadcasts `WM_SETTINGCHANGE` after writes |
| Elevation | Supports on-demand restart as administrator when system variables are read-only |
| Backup format | JSON snapshots |
| Backup directory | `%APPDATA%\VarMan\backups` |

### Linux / macOS

| Scope | Configuration files |
| --- | --- |
| User variables | Prefer `~/.zshrc`; otherwise try `~/.bashrc`, `~/.bash_profile`, and `~/.profile` |
| System variables | `/etc/environment` and `/etc/profile` |

Unix implementation details:

- Parses both `export VAR=value` and `VAR=value`.
- Ignores comments and blank lines.
- Modifies file contents in memory, writes a temporary file, and atomically replaces the original.
- Quotes values containing spaces or special characters and escapes embedded double quotes.
- Requires root to write system variables.
- Stores backups in `~/.VarMan/backups`.

## Safety Model

1. **Preview required**: Applying changes and restoring backups both generate a diff first.
2. **Permission checks**: Write permission for the target scope is checked before writing.
3. **Automatic backup**: A backup is created before the actual write, without requiring a manual step.
4. **Atomic writes**: Unix configuration files use a temporary file plus atomic replacement to reduce corruption risk.
5. **Read-only fallback**: The UI becomes read-only when permissions are insufficient.
6. **Restore protection**: Only backup files inside VarMan's backup directory can be restored.

Errors are returned as structured objects containing stable codes and messages. The UI maps these codes to localized messages:

| Code | Meaning |
| --- | --- |
| `PERMISSION_DENIED` | Insufficient permissions; elevation is required |
| `NOT_FOUND` | Environment variable or file not found |
| `PARSE_ERROR` | Configuration or request format error |
| `IO_ERROR` | System read/write failure |
| `UNSUPPORTED` | Operation unsupported on the current platform |

## Backups and Restore

- The target scope is automatically backed up before every change.
- Manual backups can be created for either user or system scope.
- The backup list shows the path, creation time, and scope.
- Restore shows a diff preview before any write.
- Restore itself creates a new backup first.
- Windows uses JSON snapshots; Unix backs up all existing candidate configuration files.

## Requirements

| Dependency | Recommended version |
| --- | --- |
| Node.js | 22 or newer |
| pnpm | 10 or newer |
| Rust | Stable toolchain |
| Windows | Windows 10 or newer with WebView2 Runtime |
| Linux | WebKitGTK 4.1 and Tauri 2 system dependencies |
| macOS | Xcode Command Line Tools |

Example dependency installation for Ubuntu/Debian:

```bash
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

## Quick Start

```bash
# Install frontend dependencies
pnpm install

# Start the desktop app in development mode
pnpm tauri dev
```

Vite uses fixed port `1420` during development. If the port is occupied, the command fails; free the port before retrying.

## Useful Commands

| Command | Description |
| --- | --- |
| `pnpm dev` | Start only the Vite frontend dev server |
| `pnpm tauri dev` | Start the complete Tauri desktop app |
| `pnpm build` | Run TypeScript checks and build the frontend |
| `pnpm preview` | Preview the built frontend |
| `pnpm tauri build` | Build installers for the current platform |
| `cargo test --manifest-path src-tauri/Cargo.toml` | Run Rust unit tests |
| `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` | Check Rust formatting |

## Project Structure

```text
VarMan/
├── .github/workflows/
│   └── build.yml                  # Cross-platform CI workflow
├── docs/
│   ├── prompt.txt                 # Original product brief
│   └── superpowers/               # Design and implementation plans
├── src/
│   ├── App.tsx                    # App layout and edit workflow
│   ├── components/
│   │   ├── BackupDialog.tsx       # Backup and restore
│   │   ├── DiffDialog.tsx         # Side-by-side diff preview
│   │   ├── PathEditor.tsx         # Dedicated Path editor
│   │   ├── PermissionAlert.tsx    # Permission guidance
│   │   ├── Sidebar.tsx            # Scope and language switching
│   │   ├── Toolbar.tsx            # Search and primary actions
│   │   ├── VariableDialog.tsx     # Create/edit variables
│   │   ├── VariableTable.tsx      # Variable list
│   │   └── ui/                    # shadcn/ui components
│   ├── lib/
│   │   ├── api.ts                 # Tauri command wrappers
│   │   ├── errors.ts              # Error mapping
│   │   └── i18n.ts                # i18next setup
│   ├── locales/                   # Chinese and English resources
│   ├── stores/env-store.ts        # Zustand state
│   └── types/env.ts               # Shared frontend types
└── src-tauri/
    ├── capabilities/default.json  # Tauri permissions
    ├── src/commands/mod.rs        # Tauri command entry point
    ├── src/core/                  # Models, errors, diff, and Path logic
    ├── src/platform/              # Windows and Unix implementations
    └── tauri.conf.json            # Main Tauri configuration
```

## Development Notes

### Frontend state

Core state is managed with Zustand:

- Current scope
- Environment variable list
- Pending changes
- Loading state
- Latest error
- Backup list
- Write permission state
- Latest operation and backup path

### Tauri commands

All system operations are invoked through Tauri commands; the frontend never calls system APIs directly. The main commands are:

| Command | Description |
| --- | --- |
| `list_env_vars` | List variables for a scope |
| `get_env_var` | Get one variable |
| `preview_changes` | Generate a change preview |
| `apply_changes` | Check permissions, back up, and apply changes |
| `remove_env_var` | Delete a variable |
| `check_permission` | Check write permission |
| `create_backup` | Create a backup |
| `list_backups` | List backups |
| `preview_backup_restore` | Preview a backup restore |
| `restore_backup` | Restore a backup |
| `parse_path_var` | Split a Path value |
| `join_path_var` | Join Path entries |
| `restart_as_admin` | Restart as administrator on Windows |

### File and directory selection

Variable editing supports native dialogs for choosing either a directory or a file. The UI can switch between Directory and File modes, and the selected path fills the current input or list entry.

### Languages

Language resources live in `src/locales/`:

- `zh-CN.json`
- `en.json`

The selected language is persisted in browser `localStorage` under `varman.locale`. If no value is stored, VarMan matches the browser language and falls back to Simplified Chinese.

## Testing and Verification

```bash
# Rust format check
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check

# Rust unit tests
cargo test --manifest-path src-tauri/Cargo.toml

# Frontend type check and build
pnpm build
```

Run all three commands before submitting changes.

## Packaging

```bash
pnpm tauri build
```

Bundles are written to `src-tauri/target/release/bundle/`:

| Platform | Artifacts |
| --- | --- |
| Windows | MSI and NSIS installers |
| Linux | deb and AppImage |
| macOS | DMG |

## Continuous Integration

`.github/workflows/build.yml` builds VarMan on Windows, Ubuntu, and macOS. The workflow:

1. Installs pnpm, Node.js, and Rust.
2. Installs platform dependencies.
3. Runs Rust formatting checks and unit tests.
4. Builds the frontend.
5. Builds desktop bundles.
6. Uploads build artifacts.

macOS signing and notarization are configured through GitHub Secrets:

- `APPLE_SIGNING_IDENTITY`
- `APPLE_ID`
- `APPLE_PASSWORD`
- `APPLE_TEAM_ID`

### Automatic releases

After all three platforms build and test successfully, CI creates a GitHub Release and uploads installers:

- Windows: MSI and NSIS installers
- Linux: deb and AppImage
- macOS: DMG

Releases are triggered by `v*` tags, for example:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The release job verifies that the tag, `package.json`, and `src-tauri/tauri.conf.json` versions match; mismatches fail the release. If a release with the same tag already exists, its assets are updated with `--clobber`, which is useful when repairing a release run.

## FAQ

### Why do some already-running programs not see updated variables?

Updating system variables does not inject new values into already-running processes. VarMan broadcasts `WM_SETTINGCHANGE` on Windows, so applications that listen for it may refresh automatically, but most applications still need to restart. Unix shells must be reopened or their configuration reloaded.

### How do I modify system variables on Linux/macOS?

Run VarMan as root. For development, use an elevated terminal; for a packaged app, use the launcher path with `sudo` or your desktop environment's supported elevation mechanism.

### What can I do without write permission?

VarMan enters read-only mode: viewing, searching, and previewing remain available, but writes are disabled. Windows shows a Restart as Administrator action; Unix recommends restarting with `sudo` or `pkexec`.

### Why does Unix show multiple sources?

Multiple Unix user configuration files can exist at the same time. VarMan parses every existing candidate file and shows its path in the Source column, preventing the misconception that a variable came from a single file.
