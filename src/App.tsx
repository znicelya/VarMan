import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { BackupDialog } from "@/components/BackupDialog";
import { DiffDialog } from "@/components/DiffDialog";
import { PathEditor } from "@/components/PathEditor";
import { PermissionAlert } from "@/components/PermissionAlert";
import { Sidebar } from "@/components/Sidebar";
import { Toolbar } from "@/components/Toolbar";
import { VariableDialog } from "@/components/VariableDialog";
import { VariableTable } from "@/components/VariableTable";
import { Separator } from "@/components/ui/separator";
import { Toaster } from "@/components/ui/sonner";
import { envApi } from "@/lib/api";
import { showEnvError } from "@/lib/errors";
import { useEnvStore } from "@/stores/env-store";
import type { Change, EnvDiff, EnvVar } from "@/types/env";

const platform = navigator.platform;
const isWindows = platform.startsWith("Win");
const shell = isWindows
  ? "PowerShell"
  : platform.includes("Mac")
    ? "zsh"
    : "bash";
const pathSeparator = isWindows ? ";" : ":";

function App() {
  const { t } = useTranslation();
  const currentScope = useEnvStore((state) => state.currentScope);
  const envVars = useEnvStore((state) => state.envVars);
  const isLoading = useEnvStore((state) => state.isLoading);
  const hasPermission = useEnvStore((state) => state.hasPermission);
  const backups = useEnvStore((state) => state.backups);
  const lastOperation = useEnvStore((state) => state.lastOperation);
  const lastBackupPath = useEnvStore((state) => state.lastBackupPath);
  const loadScope = useEnvStore((state) => state.loadScope);
  const refresh = useEnvStore((state) => state.refresh);
  const applyChanges = useEnvStore((state) => state.applyChanges);
  const createBackup = useEnvStore((state) => state.createBackup);
  const restoreBackup = useEnvStore((state) => state.restoreBackup);

  const [search, setSearch] = useState("");
  const [variableDialogOpen, setVariableDialogOpen] = useState(false);
  const [dialogMode, setDialogMode] = useState<"create" | "edit">("create");
  const [editingVariable, setEditingVariable] = useState<EnvVar | null>(null);
  const [pathEditorOpen, setPathEditorOpen] = useState(false);
  const [pathVariable, setPathVariable] = useState<EnvVar | null>(null);
  const [backupDialogOpen, setBackupDialogOpen] = useState(false);
  const [diffDialogOpen, setDiffDialogOpen] = useState(false);
  const [pendingChange, setPendingChange] = useState<Change | null>(null);
  const [pendingDiff, setPendingDiff] = useState<EnvDiff | null>(null);
  const [pendingRestorePath, setPendingRestorePath] = useState<string | null>(null);

  useEffect(() => {
    void loadScope("user");
  }, [loadScope]);

  useEffect(() => {
    setPendingChange(null);
    setPendingRestorePath(null);
    setPendingDiff(null);
    setDiffDialogOpen(false);
  }, [currentScope]);

  const filteredVariables = useMemo(() => {
    const keyword = search.trim().toLowerCase();
    if (!keyword) {
      return envVars;
    }
    return envVars.filter((variable) =>
      variable.name.toLowerCase().includes(keyword),
    );
  }, [envVars, search]);

  async function requestChange(change: Change) {
    try {
      const diff = await envApi.previewChanges([change]);
      setPendingChange(change);
      setPendingDiff(diff);
      setDiffDialogOpen(true);
      return true;
    } catch (error) {
      showEnvError(error);
      return false;
    }
  }

  function openCreateDialog() {
    setDialogMode("create");
    setEditingVariable(null);
    setVariableDialogOpen(true);
  }

  function openEditDialog(variable: EnvVar) {
    setDialogMode("edit");
    setEditingVariable(variable);
    setVariableDialogOpen(true);
  }

  async function submitVariable(variable: EnvVar) {
    const changeType = dialogMode === "edit" ? "modify" : "add";
    const submitted = await requestChange({ type: changeType, value: variable });
    if (submitted) {
      setVariableDialogOpen(false);
    }
  }

  async function deleteVariable(variable: EnvVar) {
    await requestChange({ type: "remove", value: variable });
  }

  async function previewRestore(path: string) {
    try {
      const diff = await envApi.previewBackupRestore(path);
      setPendingChange(null);
      setPendingRestorePath(path);
      setPendingDiff(diff);
      setBackupDialogOpen(false);
      setDiffDialogOpen(true);
    } catch (error) {
      showEnvError(error);
    }
  }

  async function savePath(value: string, isExpandable: boolean) {
    if (!pathVariable) {
      return;
    }
    const submitted = await requestChange({
      type: "modify",
      value: {
        ...pathVariable,
        value,
        isExpandable,
      },
    });
    if (submitted) {
      setPathEditorOpen(false);
    }
  }

  async function confirmApply() {
    if (pendingRestorePath) {
      const restored = await restoreBackup(pendingRestorePath);
      if (restored) {
        setDiffDialogOpen(false);
        setPendingRestorePath(null);
        setPendingDiff(null);
      }
      return;
    }

    if (!pendingChange) {
      return;
    }
    const diff = await applyChanges([pendingChange]);
    if (diff) {
      setDiffDialogOpen(false);
      setPendingChange(null);
      setPendingDiff(null);
    }
  }

  return (
    <div className="flex h-screen overflow-hidden bg-background text-foreground">
      <Sidebar
        currentScope={currentScope}
        onScopeChange={(scope) => void loadScope(scope)}
        platform={platform}
        shell={shell}
      />
      <div className="flex min-w-0 flex-1 flex-col">
        <Toolbar
          search={search}
          onSearchChange={setSearch}
          onCreate={openCreateDialog}
          onOpenBackups={() => setBackupDialogOpen(true)}
          onRefresh={() => void refresh()}
          canWrite={hasPermission}
        />
        <PermissionAlert
          visible={currentScope === "system" && !hasPermission}
          platform={platform}
        />
        <main className="min-h-0 flex-1 p-4">
          <VariableTable
            variables={filteredVariables}
            isLoading={isLoading}
            canWrite={hasPermission}
            platform={platform}
            onEdit={openEditDialog}
            onDelete={(variable) => void deleteVariable(variable)}
            onEditPath={(variable) => {
              setPathVariable(variable);
              setPathEditorOpen(true);
            }}
          />
        </main>
        <footer className="flex h-9 shrink-0 items-center gap-4 border-t bg-background px-4 text-xs text-muted-foreground">
          <span>{lastOperation}</span>
          {lastBackupPath ? (
            <>
              <Separator orientation="vertical" className="h-4" />
              <span className="max-w-96 truncate" title={lastBackupPath}>
                {t("footer.backupPrefix")} {lastBackupPath}
              </span>
            </>
          ) : null}
          <Separator orientation="vertical" className="ml-auto h-4" />
          <span>{t("footer.variableCount", { count: envVars.length })}</span>
          <span>{t("footer.backupCount", { count: backups.length })}</span>
        </footer>
      </div>

      <VariableDialog
        open={variableDialogOpen}
        onOpenChange={setVariableDialogOpen}
        mode={dialogMode}
        scope={currentScope}
        variable={editingVariable}
        canWrite={hasPermission}
        onSubmit={(variable) => void submitVariable(variable)}
      />
      <PathEditor
        open={pathEditorOpen}
        onOpenChange={setPathEditorOpen}
        variable={pathVariable}
        separator={pathSeparator}
        canWrite={hasPermission}
        onSave={(value, isExpandable) => void savePath(value, isExpandable)}
      />
      <BackupDialog
        open={backupDialogOpen}
        onOpenChange={setBackupDialogOpen}
        backups={backups}
        currentScope={currentScope}
        isLoading={isLoading}
        onCreate={(scope) => void createBackup(scope)}
        onPreviewRestore={(path) => void previewRestore(path)}
      />
      <DiffDialog
        open={diffDialogOpen}
        onOpenChange={(nextOpen) => {
          setDiffDialogOpen(nextOpen);
          if (!nextOpen) {
            setPendingChange(null);
            setPendingRestorePath(null);
            setPendingDiff(null);
          }
        }}
        diff={pendingDiff}
        isApplying={isLoading}
        onConfirm={() => void confirmApply()}
      />
      <Toaster position="bottom-right" />
    </div>
  );
}

export default App;
