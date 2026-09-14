import { useTranslation } from "react-i18next";
import { Button } from "@/components/ui/button";
import { setLocale } from "@/lib/i18n";
import type { Scope } from "@/types/env";

interface SidebarProps {
  currentScope: Scope;
  onScopeChange: (scope: Scope) => void;
  platform: string;
  shell: string;
}

const scopes: Scope[] = ["user", "system"];

export function Sidebar({
  currentScope,
  onScopeChange,
  platform,
  shell,
}: SidebarProps) {
  const { t, i18n } = useTranslation();

  return (
    <aside className="flex w-56 shrink-0 flex-col border-r bg-sidebar text-sidebar-foreground">
      <div className="p-4">
        <p className="font-heading text-base font-medium">VarMan</p>
        <p className="mt-1 text-sm text-muted-foreground">{t("sidebar.tagline")}</p>
      </div>
      <nav className="flex flex-col gap-1 p-2" aria-label={t("sidebar.scopeNavLabel")}>
        {scopes.map((scope) => (
          <Button
            key={scope}
            type="button"
            variant={currentScope === scope ? "secondary" : "ghost"}
            className="flex h-auto flex-col items-start gap-0.5 px-3 py-2 text-left"
            onClick={() => onScopeChange(scope)}
          >
            <span>
              {t(scope === "user" ? "sidebar.userVariable" : "sidebar.systemVariable")}
            </span>
            <span className="text-xs font-normal text-muted-foreground">
              {t(
                scope === "user"
                  ? "sidebar.userDescription"
                  : "sidebar.systemDescription",
              )}
            </span>
          </Button>
        ))}
      </nav>
      <div className="mt-auto border-t p-4 text-xs text-muted-foreground">
        <p>{t("sidebar.platform", { platform })}</p>
        <p className="mt-1">{t("sidebar.shell", { shell })}</p>
        <div className="mt-3 flex items-center gap-1">
          <span className="mr-1">{t("sidebar.language")}</span>
          <Button
            type="button"
            variant={i18n.language === "en" ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setLocale("en")}
          >
            EN
          </Button>
          <Button
            type="button"
            variant={i18n.language === "zh-CN" ? "secondary" : "ghost"}
            size="sm"
            onClick={() => setLocale("zh-CN")}
          >
            中文
          </Button>
        </div>
      </div>
    </aside>
  );
}
