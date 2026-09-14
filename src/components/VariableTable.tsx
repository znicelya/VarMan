import {
  FolderIcon,
  PencilIcon,
  Trash2Icon,
} from "lucide-react";
import {
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Empty,
  EmptyDescription,
  EmptyHeader,
  EmptyMedia,
  EmptyTitle,
} from "@/components/ui/empty";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Skeleton } from "@/components/ui/skeleton";
import { useTranslation } from "react-i18next";
import type { EnvVar } from "@/types/env";

interface VariableTableProps {
  variables: EnvVar[];
  isLoading: boolean;
  canWrite: boolean;
  platform: string;
  onEdit: (variable: EnvVar) => void;
  onDelete: (variable: EnvVar) => void;
  onEditPath: (variable: EnvVar) => void;
}

export function VariableTable({
  variables,
  isLoading,
  canWrite,
  platform,
  onEdit,
  onDelete,
  onEditPath,
}: VariableTableProps) {
  const separator = platform.startsWith("Win") ? ";" : ":";
  const { t } = useTranslation();

  return (
    <Card className="h-full min-h-0">
      <CardHeader>
        <CardTitle>{t("table.title")}</CardTitle>
        <CardDescription>
          {isLoading
            ? t("table.loading")
            : t("table.count", { count: variables.length })}
        </CardDescription>
        <CardAction>
          <span className="text-xs text-muted-foreground">
            {canWrite ? t("common.writable") : t("common.readonly")}
          </span>
        </CardAction>
      </CardHeader>
      <CardContent className="min-h-0 p-0">
        {isLoading ? (
          <div className="flex flex-col gap-2 p-4">
            {Array.from({ length: 6 }).map((_, index) => (
              <Skeleton key={index} className="h-10" />
            ))}
          </div>
        ) : variables.length === 0 ? (
          <Empty className="m-4 border">
            <EmptyHeader>
              <EmptyMedia variant="icon">
                <FolderIcon />
              </EmptyMedia>
              <EmptyTitle>{t("table.emptyTitle")}</EmptyTitle>
              <EmptyDescription>
                {t("table.emptyDescription")}
              </EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <div className="max-h-full overflow-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-48">{t("table.name")}</TableHead>
                  <TableHead>{t("table.value")}</TableHead>
                  <TableHead className="w-56">{t("table.source")}</TableHead>
                  <TableHead className="w-24 text-right">{t("table.actions")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {variables.map((variable) => {
                  const isPath = variable.name.toLowerCase() === "path";
                  const pathCount = variable.value
                    .split(separator)
                    .filter(Boolean).length;

                  return (
                    <TableRow key={`${variable.scope}-${variable.name}`}>
                      <TableCell className="font-medium">{variable.name}</TableCell>
                      <TableCell className="max-w-[420px]">
                        {isPath ? (
                          <Button
                            type="button"
                            variant="link"
                            size="sm"
                            className="h-6 px-0"
                            onClick={() => onEditPath(variable)}
                          >
                            {t("table.pathCount", { count: pathCount })}
                          </Button>
                        ) : (
                          <span
                            title={variable.value}
                            className="block max-w-[420px] truncate text-muted-foreground"
                          >
                            {variable.value}
                          </span>
                        )}
                      </TableCell>
                      <TableCell className="truncate text-muted-foreground">
                        {variable.source
                          ?? (platform.startsWith("Win")
                            ? t("table.registry")
                            : t("table.shellConfig"))}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-1">
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon-sm"
                            disabled={!canWrite}
                            onClick={() => onEdit(variable)}
                          >
                            <PencilIcon />
                            <span className="sr-only">
                              {t("table.editLabel", { name: variable.name })}
                            </span>
                          </Button>
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon-sm"
                            disabled={!canWrite}
                            onClick={() => onDelete(variable)}
                          >
                            <Trash2Icon className="text-destructive" />
                            <span className="sr-only">
                              {t("table.deleteLabel", { name: variable.name })}
                            </span>
                          </Button>
                        </div>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
