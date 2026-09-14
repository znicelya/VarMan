import { toast } from "sonner";
import { envApi } from "@/lib/api";
import { showEnvError, toEnvError } from "@/lib/errors";
import i18n from "@/lib/i18n";
import type {
  BackupInfo,
  Change,
  EnvDiff,
  EnvError,
  EnvVar,
  Scope,
} from "@/types/env";
import { create } from "zustand";

interface EnvState {
  currentScope: Scope;
  envVars: EnvVar[];
  pendingChanges: Change[];
  isLoading: boolean;
  lastError: EnvError | null;
  backups: BackupInfo[];
  hasPermission: boolean;
  lastOperation: string;
  lastBackupPath: string;
  loadScope: (scope: Scope) => Promise<void>;
  refresh: () => Promise<void>;
  setPendingChanges: (changes: Change[]) => void;
  applyChanges: (changes: Change[]) => Promise<EnvDiff | null>;
  removeEnvVar: (name: string, scope: Scope) => Promise<EnvDiff | null>;
  createBackup: (scope: Scope) => Promise<boolean>;
  restoreBackup: (path: string) => Promise<boolean>;
  clearError: () => void;
}

function describeDiff(diff: EnvDiff) {
  return i18n.t("store.appliedDiff", {
    added: diff.added.length,
    modified: diff.modified.length,
    removed: diff.removed.length,
  });
}

export const useEnvStore = create<EnvState>()((set, get) => {
  let loadRequestId = 0;

  return {
    currentScope: "user",
    envVars: [],
    pendingChanges: [],
    isLoading: false,
    lastError: null,
    backups: [],
    hasPermission: true,
    lastOperation: i18n.t("store.noOperation"),
    lastBackupPath: "",

    loadScope: async (scope) => {
      const requestId = ++loadRequestId;
      set({ currentScope: scope, isLoading: true, lastError: null });
      try {
        const [hasPermission, envVars, backups] = await Promise.all([
          envApi.checkPermission(scope),
          envApi.listEnvVars(scope),
          envApi.listBackups(),
        ]);
        if (requestId !== loadRequestId) {
          return;
        }
        set({ hasPermission, envVars, backups, isLoading: false });
      } catch (error) {
        if (requestId !== loadRequestId) {
          return;
        }
        set({ lastError: toEnvError(error), isLoading: false });
        showEnvError(error);
      }
    },

  refresh: async () => {
    await get().loadScope(get().currentScope);
  },

  setPendingChanges: (changes) => set({ pendingChanges: changes }),

  applyChanges: async (changes) => {
    set({ isLoading: true, lastError: null });
    try {
      const diff = await envApi.applyChanges(changes);
      let backups = get().backups;
      let envVars = get().envVars;
      try {
        [backups, envVars] = await Promise.all([
          envApi.listBackups(),
          envApi.listEnvVars(get().currentScope),
        ]);
      } catch {
        toast.warning(i18n.t("store.refreshListFailed"));
      }
      set({
        pendingChanges: [],
        envVars,
        backups,
        isLoading: false,
        lastOperation: describeDiff(diff),
      });
      toast.success(i18n.t("store.applied"));
      return diff;
    } catch (error) {
      set({ lastError: toEnvError(error), isLoading: false });
      showEnvError(error);
      return null;
    }
  },

  removeEnvVar: async (name, scope) => {
    return get().applyChanges([{ type: "remove", value: { ...createRemoveVar(name, scope) } }]);
  },

  createBackup: async (scope) => {
    set({ isLoading: true });
    try {
      const path = await envApi.createBackup(scope);
      let backups = get().backups;
      try {
        backups = await envApi.listBackups();
      } catch {
        toast.warning(i18n.t("store.backupListFailed"));
      }
      set({
        backups,
        isLoading: false,
        lastBackupPath: path,
        lastOperation: i18n.t("store.backupCreated"),
      });
      toast.success(i18n.t("store.backupCreated"));
      return true;
    } catch (error) {
      set({ lastError: toEnvError(error), isLoading: false });
      showEnvError(error);
      return false;
    }
  },

  restoreBackup: async (path) => {
    set({ isLoading: true });
    try {
      await envApi.restoreBackup(path);
      await get().refresh();
      set({ lastOperation: i18n.t("store.backupRestored") });
      toast.success(i18n.t("store.backupRestored"));
      return true;
    } catch (error) {
      set({ lastError: toEnvError(error), isLoading: false });
      showEnvError(error);
      return false;
    }
  },

    clearError: () => set({ lastError: null }),
  };
});

function createRemoveVar(name: string, scope: Scope): EnvVar {
  return {
    name,
    value: "",
    scope,
    isExpandable: false,
    rawValue: null,
    source: null,
  };
}
