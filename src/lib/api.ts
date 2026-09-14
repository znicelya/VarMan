import { invoke } from "@tauri-apps/api/core";
import type {
  BackupInfo,
  Change,
  EnvDiff,
  EnvError,
  EnvVar,
  PathEntry,
  Scope,
} from "@/types/env";

function asEnvError(error: unknown): EnvError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    return error as EnvError;
  }
  return { code: "UNKNOWN", message: error instanceof Error ? error.message : String(error) };
}

async function command<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    throw asEnvError(error);
  }
}

export const envApi = {
  listEnvVars: (scope: Scope) => command<EnvVar[]>("list_env_vars", { scope }),
  getEnvVar: (name: string, scope: Scope) =>
    command<EnvVar | null>("get_env_var", { name, scope }),
  previewChanges: (changes: Change[]) => command<EnvDiff>("preview_changes", { changes }),
  applyChanges: (changes: Change[]) => command<EnvDiff>("apply_changes", { changes }),
  removeEnvVar: (name: string, scope: Scope) =>
    command<EnvDiff>("remove_env_var", { name, scope }),
  checkPermission: (scope: Scope) => command<boolean>("check_permission", { scope }),
  createBackup: (scope: Scope) => command<string>("create_backup", { scope }),
  listBackups: () => command<BackupInfo[]>("list_backups"),
  previewBackupRestore: (path: string) =>
    command<EnvDiff>("preview_backup_restore", { path }),
  restoreBackup: (path: string) => command<void>("restore_backup", { path }),
  parsePathVar: (value: string) => command<PathEntry[]>("parse_path_var", { value }),
  joinPathVar: (entries: PathEntry[]) =>
    command<string>("join_path_var", { entries }),
  restartAsAdmin: () => command<void>("restart_as_admin"),
};
