import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { EnvDiff, EnvVar } from "@/types/env";

interface DiffDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  diff: EnvDiff | null;
  isApplying: boolean;
  onConfirm: () => void;
}

type DiffRowKind =
  | "added"
  | "removed"
  | "modified-before"
  | "modified-after";

interface DiffRowProps {
  variable: EnvVar;
  compareVariable?: EnvVar;
  kind: DiffRowKind;
  badgeText: string;
}

function splitEntries(value: string) {
  return value ? value.split(";") : [];
}

function diffEntryFlags(current: string[], compare: string[]) {
  const rows = current.length;
  const columns = compare.length;
  const lengths = Array.from({ length: rows + 1 }, () =>
    Array.from({ length: columns + 1 }, () => 0),
  );

  for (let row = rows - 1; row >= 0; row -= 1) {
    for (let column = columns - 1; column >= 0; column -= 1) {
      lengths[row][column] =
        current[row] === compare[column]
          ? lengths[row + 1][column + 1] + 1
          : Math.max(lengths[row + 1][column], lengths[row][column + 1]);
    }
  }

  const flags: boolean[] = [];
  let row = 0;
  let column = 0;

  while (row < rows && column < columns) {
    if (current[row] === compare[column]) {
      flags.push(false);
      row += 1;
      column += 1;
    } else if (lengths[row + 1][column] >= lengths[row][column + 1]) {
      flags.push(true);
      row += 1;
    } else {
      column += 1;
    }
  }

  while (row < rows) {
    flags.push(true);
    row += 1;
  }

  return flags;
}

function DiffRow({
  variable,
  compareVariable,
  kind,
  badgeText,
}: DiffRowProps) {
  const { t } = useTranslation();
  const isAdded = kind === "added";
  const isRemoved = kind === "removed";
  const isBefore = kind === "modified-before";
  const entries = splitEntries(variable.value);

  const flags =
    compareVariable && (kind === "modified-before" || kind === "modified-after")
      ? diffEntryFlags(
          entries,
          splitEntries(compareVariable.value),
        )
      : entries.map(() => true);

  return (
    <div className="flex flex-col gap-2 rounded-lg border bg-background px-3 py-2">
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 truncate text-xs font-semibold">
          {variable.name}
        </span>
        <Badge variant={isRemoved ? "destructive" : isAdded || kind === "modified-after" ? undefined : "outline"} className={isAdded ? "bg-success/15 text-success" : kind === "modified-after" ? "bg-success/15 text-success" : undefined}>
          {badgeText}
        </Badge>
      </div>
      <div className="flex flex-col gap-1">
        {entries.length ? (
          entries.map((entry, index) => {
            const isChanged = flags[index] ?? true;
            const itemClassName = isChanged
              ? isBefore
                ? "bg-destructive/10 text-destructive"
                : "bg-success/10 text-success"
              : "bg-muted/40 text-foreground";

            return (
              <div
                key={`${entry}-${index}`}
                className={`flex items-center gap-2 rounded-md px-2 py-1 ${itemClassName}`}
              >
                <span
                  className="min-w-0 flex-1 truncate font-mono text-xs"
                  title={entry}
                >
                  {entry}
                </span>
              </div>
            );
          })
        ) : (
          <span className="font-mono text-xs text-muted-foreground">{t("diff.none")}</span>
        )}
      </div>
    </div>
  );
}

export function DiffDialog({
  open,
  onOpenChange,
  diff,
  isApplying,
  onConfirm,
}: DiffDialogProps) {
  const { t } = useTranslation();
  const hasChanges = Boolean(
    diff && (diff.added.length || diff.modified.length || diff.removed.length),
  );

  const beforeEntries = diff
    ? [
        ...diff.modified.map(([oldVariable, newVariable]) => ({
          key: `modified-before-${oldVariable.name}`,
          kind: "modified-before" as const,
          variable: oldVariable,
          compareVariable: newVariable,
        })),
        ...diff.removed.map((variable) => ({
          key: `removed-${variable.name}`,
          kind: "removed" as const,
          variable,
          compareVariable: undefined,
        })),
      ]
    : [];

  const afterEntries = diff
    ? [
        ...diff.modified.map(([oldVariable, newVariable]) => ({
          key: `modified-after-${newVariable.name}`,
          kind: "modified-after" as const,
          variable: newVariable,
          compareVariable: oldVariable,
        })),
        ...diff.added.map((variable) => ({
          key: `added-${variable.name}`,
          kind: "added" as const,
          variable,
          compareVariable: undefined,
        })),
      ]
    : [];

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("diff.title")}</DialogTitle>
          <DialogDescription>{t("diff.description")}</DialogDescription>
        </DialogHeader>
        {hasChanges && diff ? (
          <ScrollArea className="max-h-80">
            <div className="grid grid-cols-2 gap-3 pr-3">
              <section className="min-w-0">
                <p className="mb-2 text-xs font-semibold text-muted-foreground">
                  {t("diff.before")}
                </p>
                {beforeEntries.length ? (
                  <div className="flex flex-col gap-2">
                    {beforeEntries.map((entry) => (
                      <DiffRow
                        key={entry.key}
                        variable={entry.variable}
                        compareVariable={entry.compareVariable}
                        kind={entry.kind}
                        badgeText={
                          entry.kind === "removed"
                            ? t("diff.badgeRemoved")
                            : t("diff.badgeModified")
                        }
                      />
                    ))}
                  </div>
                ) : (
                  <p className="text-xs text-muted-foreground">{t("diff.none")}</p>
                )}
              </section>
              <section className="min-w-0">
                <p className="mb-2 text-xs font-semibold text-muted-foreground">
                  {t("diff.after")}
                </p>
                {afterEntries.length ? (
                  <div className="flex flex-col gap-2">
                    {afterEntries.map((entry) => (
                      <DiffRow
                        key={entry.key}
                        variable={entry.variable}
                        compareVariable={entry.compareVariable}
                        kind={entry.kind}
                        badgeText={
                          entry.kind === "added"
                            ? t("diff.badgeAdded")
                            : t("diff.badgeModified")
                        }
                      />
                    ))}
                  </div>
                ) : (
                  <p className="text-xs text-muted-foreground">{t("diff.none")}</p>
                )}
              </section>
            </div>
          </ScrollArea>
        ) : (
          <p className="text-sm text-muted-foreground">{t("diff.empty")}</p>
        )}
        <DialogFooter>
          <DialogClose render={<Button type="button" variant="outline" />}>
            {t("common.cancel")}
          </DialogClose>
          <Button type="button" onClick={onConfirm} disabled={!hasChanges || isApplying}>
            {isApplying ? t("diff.applying") : t("diff.apply")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
