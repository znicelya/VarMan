import { useEffect, useState } from "react";
import { FileIcon, FolderOpenIcon, FolderPlusIcon, GripVerticalIcon, Trash2Icon } from "lucide-react";
import { useTranslation } from "react-i18next";
import { open as openDirectoryDialog } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import {
  Field,
  FieldContent,
  FieldDescription,
  FieldError,
  FieldGroup,
  FieldLabel,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { showEnvError } from "@/lib/errors";
import type { EnvVar, Scope } from "@/types/env";

interface VariableDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  mode: "create" | "edit";
  scope: Scope;
  variable: EnvVar | null;
  canWrite: boolean;
  onSubmit: (variable: EnvVar) => void;
}

export function VariableDialog({
  open,
  onOpenChange,
  mode,
  scope,
  variable,
  canWrite,
  onSubmit,
}: VariableDialogProps) {
  const { t } = useTranslation();
  const [name, setName] = useState("");
  const [value, setValue] = useState("");
  const [isExpandable, setIsExpandable] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [entries, setEntries] = useState<string[]>([]);
  const [newEntry, setNewEntry] = useState("");
  const [chooserKind, setChooserKind] = useState<"directory" | "file">("directory");

  useEffect(() => {
    if (!open) {
      return;
    }
    setName(variable?.name ?? "");
    setValue(variable?.value ?? "");
    setEntries(variable?.value ? variable.value.split(";") : []);
    setIsExpandable(variable?.isExpandable ?? false);
    setError(null);
    setNewEntry("");
  }, [open, variable]);

  const isListValue = entries.length > 1 || value.includes(";");

  function updateEntry(index: number, nextValue: string) {
    setEntries((current) =>
      current.map((entry, entryIndex) =>
        entryIndex === index ? nextValue : entry,
      ),
    );
  }

  function removeEntry(index: number) {
    setEntries((current) => current.filter((_, entryIndex) => entryIndex !== index));
  }

  function addEntry() {
    const entry = newEntry.trim();
    if (!entry) {
      return;
    }
    setEntries((current) => [...current, entry]);
    setNewEntry("");
  }

  async function browseSelection(kind: "directory" | "file"): Promise<string | null> {
    try {
      const selected = await openDirectoryDialog({
        directory: kind === "directory",
        multiple: false,
      });
      return typeof selected === "string" ? selected : null;
    } catch (error) {
      showEnvError(error);
      return null;
    }
  }

  async function browseNewEntry(kind: "directory" | "file") {
    const selected = await browseSelection(kind);
    if (selected) {
      setNewEntry(selected);
    }
  }

  async function browseEntry(index: number, kind: "directory" | "file") {
    const selected = await browseSelection(kind);
    if (selected) {
      updateEntry(index, selected);
    }
  }

  async function browseValue(kind: "directory" | "file") {
    const selected = await browseSelection(kind);
    if (selected) {
      setValue(selected);
    }
  }

  function submit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!name.trim()) {
      setError(t("dialog.nameRequired"));
      return;
    }
    setError(null);
    const finalValue = isListValue
      ? entries.map((entry) => entry.trim()).join(";")
      : value;
    onSubmit({
      name: name.trim(),
      value: finalValue,
      scope,
      isExpandable,
      rawValue: variable?.rawValue ?? null,
      source: variable?.source ?? null,
    });
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="grid max-h-[calc(100vh-2rem)] grid-rows-[auto_minmax(0,1fr)_auto] overflow-hidden sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>
            {t(mode === "create" ? "dialog.createTitle" : "dialog.editTitle")}
          </DialogTitle>
          <DialogDescription>{t("dialog.previewHint")}</DialogDescription>
        </DialogHeader>
        <form
          id="variable-form"
          className="flex min-h-0 flex-col gap-4 overflow-y-auto pr-1"
          onSubmit={submit}
        >
          <FieldGroup>
            <Field data-invalid={error ? true : undefined}>
              <FieldLabel htmlFor="variable-name">{t("dialog.nameLabel")}</FieldLabel>
              <Input
                id="variable-name"
                value={name}
                onChange={(event) => setName(event.target.value)}
                disabled={mode === "edit" || !canWrite}
                aria-invalid={Boolean(error)}
              />
              {error ? <FieldError>{error}</FieldError> : null}
            </Field>
            <Field>
              <FieldLabel htmlFor="variable-value">{t("dialog.valueLabel")}</FieldLabel>
              {isListValue ? (
                <div className="max-h-56 overflow-y-auto rounded-lg border p-2">
                  <div className="flex flex-col gap-2">
                    {entries.map((entry, index) => (
                      <div
                        key={`${entry}-${index}`}
                        className="flex items-center gap-2 rounded-lg border bg-background px-3 py-2"
                      >
                        <GripVerticalIcon className="size-4 shrink-0 text-muted-foreground" />
                        <Input
                          value={entry}
                          onChange={(event) => updateEntry(index, event.target.value)}
                          disabled={!canWrite}
                          className="font-mono text-xs"
                        />
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-sm"
                          disabled={!canWrite}
                          onClick={() => void browseEntry(index, chooserKind)}
                        >
                          {chooserKind === "directory" ? <FolderOpenIcon /> : <FileIcon />}
                          <span className="sr-only">
                            {t("dialog.choose")} {t(chooserKind === "directory" ? "dialog.directory" : "dialog.file")}
                          </span>
                        </Button>
                        <Button
                          type="button"
                          variant="ghost"
                          size="icon-sm"
                          disabled={!canWrite}
                          onClick={() => removeEntry(index)}
                        >
                          <Trash2Icon className="text-destructive" />
                          <span className="sr-only">
                            {t("dialog.deleteEntry", { entry })}
                          </span>
                        </Button>
                      </div>
                    ))}
                    {entries.length === 0 ? (
                      <p className="py-4 text-center text-sm text-muted-foreground">
                        {t("dialog.entryEmpty")}
                      </p>
                    ) : null}
                    <div className="flex items-center gap-2">
                      <Input
                        value={newEntry}
                        onChange={(event) => setNewEntry(event.target.value)}
                        placeholder={t("dialog.entryPlaceholder")}
                        disabled={!canWrite}
                      />
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        onClick={addEntry}
                        disabled={!canWrite || !newEntry.trim()}
                      >
                        <FolderPlusIcon data-icon="inline-start" />
                        {t("dialog.addEntry")}
                      </Button>
                      <Button
                        type="button"
                        variant="outline"
                        size="sm"
                        disabled={!canWrite}
                        onClick={() => void browseNewEntry(chooserKind)}
                      >
                        {chooserKind === "directory" ? <FolderOpenIcon data-icon="inline-start" /> : <FileIcon data-icon="inline-start" />}
                        {t(chooserKind === "directory" ? "dialog.directory" : "dialog.file")}
                      </Button>
                      <Button
                        type="button"
                        variant="ghost"
                        size="icon-sm"
                        disabled={!canWrite}
                        onClick={() =>
                          setChooserKind((current) =>
                            current === "directory" ? "file" : "directory",
                          )
                        }
                        title={t(
                          chooserKind === "directory"
                            ? "dialog.file"
                            : "dialog.directory",
                        )}
                      >
                        {chooserKind === "directory" ? <FileIcon /> : <FolderOpenIcon />}
                        <span className="sr-only">
                          {t(
                            chooserKind === "directory"
                              ? "dialog.file"
                              : "dialog.directory",
                          )}
                        </span>
                      </Button>
                    </div>
                  </div>
                </div>
              ) : (
                <div className="flex items-center gap-2">
                  <Input
                    id="variable-value"
                    value={value}
                    onChange={(event) => setValue(event.target.value)}
                    disabled={!canWrite}
                    className="font-mono"
                  />
                  <Button
                    type="button"
                    variant="outline"
                    disabled={!canWrite}
                    onClick={() => void browseValue(chooserKind)}
                  >
                    {chooserKind === "directory" ? <FolderOpenIcon data-icon="inline-start" /> : <FileIcon data-icon="inline-start" />}
                    {t(chooserKind === "directory" ? "dialog.directory" : "dialog.file")}
                  </Button>
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    disabled={!canWrite}
                    onClick={() =>
                      setChooserKind((current) =>
                        current === "directory" ? "file" : "directory",
                      )
                    }
                    title={t(
                      chooserKind === "directory"
                        ? "dialog.file"
                        : "dialog.directory",
                    )}
                  >
                    {chooserKind === "directory" ? <FileIcon /> : <FolderOpenIcon />}
                    <span className="sr-only">
                      {t(
                        chooserKind === "directory"
                          ? "dialog.file"
                          : "dialog.directory",
                      )}
                    </span>
                  </Button>
                </div>
              )}
            </Field>
            <Field>
              <FieldContent>
                <div className="flex items-center gap-2">
                  <Checkbox
                    id="variable-expandable"
                    checked={isExpandable}
                    onCheckedChange={(checked) => setIsExpandable(checked === true)}
                    disabled={!canWrite}
                  />
                  <FieldLabel htmlFor="variable-expandable">
                    {t("dialog.keepExpandable")}
                  </FieldLabel>
                </div>
                <FieldDescription>
                  {t("dialog.expandableDescription")}
                </FieldDescription>
                {(isListValue ? entries.join(";").includes("%") : value.includes("%")) && !isExpandable ? (
                  <FieldDescription className="text-warning">
                    {t("dialog.expandableHint")}
                  </FieldDescription>
                ) : null}
              </FieldContent>
            </Field>
          </FieldGroup>
        </form>
        <DialogFooter>
          <DialogClose render={<Button type="button" variant="outline" />}>
            {t("common.cancel")}
          </DialogClose>
          <Button type="submit" form="variable-form" disabled={!canWrite}>
            {t("dialog.saveAndPreview")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
