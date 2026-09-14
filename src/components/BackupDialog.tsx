import { ArchiveIcon, RotateCcwIcon } from "lucide-react";
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
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { ScrollArea } from "@/components/ui/scroll-area";
import type { BackupInfo, Scope } from "@/types/env";

interface BackupDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  backups: BackupInfo[];
  currentScope: Scope;
  isLoading: boolean;
  onCreate: (scope: Scope) => void;
  onPreviewRestore: (path: string) => void;
}

export function BackupDialog({
  open,
  onOpenChange,
  backups,
  currentScope,
  isLoading,
  onCreate,
  onPreviewRestore,
}: BackupDialogProps) {
  const { t } = useTranslation();

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-h-[calc(100vh-2rem)] overflow-hidden sm:max-w-xl">
        <DialogHeader>
          <DialogTitle>{t("backup.title")}</DialogTitle>
          <DialogDescription>
            {t("backup.description")}
          </DialogDescription>
        </DialogHeader>
        <ScrollArea className="min-h-0 min-w-0 max-h-80">
          {backups.length === 0 ? (
            <Empty className="border">
              <EmptyHeader>
                <EmptyMedia variant="icon">
                  <ArchiveIcon />
                </EmptyMedia>
                <EmptyTitle>{t("backup.emptyTitle")}</EmptyTitle>
                <EmptyDescription>{t("backup.emptyDescription")}</EmptyDescription>
              </EmptyHeader>
            </Empty>
          ) : (
            <div className="flex flex-col gap-2 pr-3">
              {backups.map((backup) => (
                <div
                  key={backup.path}
                  className="flex items-center gap-3 rounded-lg border bg-background p-3"
                >
                  <div className="min-w-0 flex-1">
                    <p className="truncate font-medium" title={backup.path}>
                      {backup.path}
                    </p>
                    <p className="mt-1 text-xs text-muted-foreground">
                      {new Date(backup.createdAt).toLocaleString()}
                    </p>
                  </div>
                  <Badge variant="outline">
                    {t(backup.scope === "user" ? "common.user" : "common.system")}
                  </Badge>
                  <Button
                    type="button"
                    variant="outline"
                    size="sm"
                    disabled={isLoading}
                    onClick={() => onPreviewRestore(backup.path)}
                  >
                    <RotateCcwIcon data-icon="inline-start" />
                    {t("backup.restore")}
                  </Button>
                </div>
              ))}
            </div>
          )}
        </ScrollArea>
        <DialogFooter>
          <DialogClose render={<Button type="button" variant="outline" />}>
            {t("common.close")}
          </DialogClose>
          <Button
            type="button"
            disabled={isLoading}
            onClick={() => onCreate(currentScope)}
          >
            {t("backup.create")}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
