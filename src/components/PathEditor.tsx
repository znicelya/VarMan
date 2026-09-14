import { useEffect, useMemo, useRef, useState } from "react";
import {
  FolderPlusIcon,
  GripVerticalIcon,
  Trash2Icon,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetFooter,
  SheetHeader,
  SheetTitle,
} from "@/components/ui/sheet";
import { envApi } from "@/lib/api";
import { showEnvError } from "@/lib/errors";
import type { EnvVar, PathEntry } from "@/types/env";

interface PathEditorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  variable: EnvVar | null;
  separator: string;
  canWrite: boolean;
  onSave: (value: string, isExpandable: boolean) => void;
}

export function PathEditor({
  open,
  onOpenChange,
  variable,
  separator,
  canWrite,
  onSave,
}: PathEditorProps) {
  const { t } = useTranslation();
  const [entries, setEntries] = useState<PathEntry[]>([]);
  const [newPath, setNewPath] = useState("");
  const [isLoading, setIsLoading] = useState(false);
  const dragIndex = useRef<number | null>(null);
  const parseRequestId = useRef(0);

  useEffect(() => {
    if (!open || !variable) {
      return;
    }
    const requestId = ++parseRequestId.current;
    setIsLoading(true);
    envApi
      .parsePathVar(variable.value)
      .then((entries) => {
        if (requestId === parseRequestId.current) {
          setEntries(entries);
        }
      })
      .catch((error) => {
        if (requestId === parseRequestId.current) {
          showEnvError(error);
        }
      })
      .finally(() => {
        if (requestId === parseRequestId.current) {
          setIsLoading(false);
        }
      });
  }, [open, variable]);

  const joinedValue = useMemo(
    () => entries.map((entry) => entry.path).join(separator),
    [entries, separator],
  );

  function moveEntry(from: number, to: number) {
    if (from === to) {
      return;
    }
    setEntries((current) => {
      const next = [...current];
      const [entry] = next.splice(from, 1);
      next.splice(to, 0, entry);
      return next;
    });
  }

  async function addPath() {
    const path = newPath.trim();
    if (!path) {
      return;
    }
    setIsLoading(true);
    try {
      const nextValue = [...entries.map((entry) => entry.path), path].join(separator);
      setEntries(await envApi.parsePathVar(nextValue));
      setNewPath("");
    } catch (error) {
      showEnvError(error);
    } finally {
      setIsLoading(false);
    }
  }

  return (
    <Sheet open={open} onOpenChange={onOpenChange}>
      <SheetContent side="right" className="w-full gap-0 sm:max-w-xl">
        <SheetHeader>
          <SheetTitle>{t("path.title")}</SheetTitle>
          <SheetDescription>{t("path.description")}</SheetDescription>
        </SheetHeader>
        <div className="flex min-h-0 flex-1 flex-col gap-3 px-4">
          <div className="flex items-center gap-2">
            <Input
              value={newPath}
              onChange={(event) => setNewPath(event.target.value)}
              placeholder={t("path.placeholder")}
              disabled={!canWrite || isLoading}
            />
            <Button
              type="button"
              variant="outline"
              onClick={addPath}
              disabled={!canWrite || isLoading || !newPath.trim()}
            >
              <FolderPlusIcon data-icon="inline-start" />
              {t("path.add")}
            </Button>
          </div>
          <ScrollArea className="min-h-0 flex-1">
            <div className="flex flex-col gap-2 pr-3">
              {entries.map((entry, index) => (
                <div
                  key={`${entry.path}-${index}`}
                  draggable={canWrite}
                  onDragStart={() => {
                    dragIndex.current = index;
                  }}
                  onDragOver={(event) => event.preventDefault()}
                  onDrop={() => {
                    if (dragIndex.current !== null) {
                      moveEntry(dragIndex.current, index);
                    }
                    dragIndex.current = null;
                  }}
                  className="flex items-center gap-2 rounded-lg border bg-background px-3 py-2"
                >
                  <GripVerticalIcon className="size-4 shrink-0 text-muted-foreground" />
                  <span className="min-w-0 flex-1 truncate" title={entry.path}>
                    {entry.path}
                  </span>
                  {!entry.exists ? (
                    <Badge variant="destructive">{t("path.missing")}</Badge>
                  ) : null}
                  {entry.isDuplicate ? (
                    <Badge className="bg-warning/15 text-warning">{t("path.duplicate")}</Badge>
                  ) : null}
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon-sm"
                    disabled={!canWrite}
                    onClick={() =>
                      setEntries((current) =>
                        current.filter((_, entryIndex) => entryIndex !== index),
                      )
                    }
                  >
                    <Trash2Icon className="text-destructive" />
                    <span className="sr-only">
                      {t("path.deleteLabel", { path: entry.path })}
                    </span>
                  </Button>
                </div>
              ))}
              {!isLoading && entries.length === 0 ? (
                <p className="py-8 text-center text-sm text-muted-foreground">
                  {t("path.empty")}
                </p>
              ) : null}
            </div>
          </ScrollArea>
        </div>
        <SheetFooter>
          <div className="flex flex-col gap-3">
            <Separator />
            <div>
              <p className="text-xs font-medium text-muted-foreground">{t("path.joined")}</p>
              <p className="mt-1 max-h-20 overflow-auto rounded-md bg-muted p-2 font-mono text-xs break-all">
                {joinedValue || t("path.emptyJoined")}
              </p>
            </div>
            <div className="flex justify-end gap-2">
              <Button
                type="button"
                variant="outline"
                onClick={() => onOpenChange(false)}
              >
                {t("common.cancel")}
              </Button>
              <Button
                type="button"
                disabled={!canWrite || isLoading}
                onClick={() => onSave(joinedValue, variable?.isExpandable ?? false)}
              >
                {t("path.saveAndPreview")}
              </Button>
            </div>
          </div>
        </SheetFooter>
      </SheetContent>
    </Sheet>
  );
}
