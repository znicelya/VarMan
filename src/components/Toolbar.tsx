import { RefreshCwIcon, SearchIcon, SettingsIcon, PlusIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { useTranslation } from "react-i18next";

interface ToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  onCreate: () => void;
  onOpenBackups: () => void;
  onRefresh: () => void;
  canWrite: boolean;
}

export function Toolbar({
  search,
  onSearchChange,
  onCreate,
  onOpenBackups,
  onRefresh,
  canWrite,
}: ToolbarProps) {
  const { t } = useTranslation();

  return (
    <header className="flex h-14 shrink-0 items-center gap-3 border-b bg-background px-4">
      <div className="relative flex-1 max-w-sm">
        <SearchIcon className="absolute top-1/2 left-2 size-4 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder={t("toolbar.searchPlaceholder")}
          className="pl-8"
          aria-label={t("toolbar.searchLabel")}
        />
      </div>
      <div className="ml-auto flex items-center gap-2">
        <Button type="button" variant="outline" size="sm" onClick={onRefresh}>
          <RefreshCwIcon data-icon="inline-start" />
          {t("toolbar.refresh")}
        </Button>
        <Button type="button" variant="outline" size="sm" onClick={onOpenBackups}>
          <SettingsIcon data-icon="inline-start" />
          {t("toolbar.backups")}
        </Button>
        <Button type="button" size="sm" onClick={onCreate} disabled={!canWrite}>
          <PlusIcon data-icon="inline-start" />
          {t("toolbar.add")}
        </Button>
      </div>
    </header>
  );
}
