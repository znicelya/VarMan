import { TriangleAlertIcon } from "lucide-react";
import { toast } from "sonner";
import { useTranslation } from "react-i18next";
import { envApi } from "@/lib/api";
import { showEnvError } from "@/lib/errors";
import {
  Alert,
  AlertAction,
  AlertDescription,
  AlertTitle,
} from "@/components/ui/alert";
import { Button } from "@/components/ui/button";

interface PermissionAlertProps {
  visible: boolean;
  platform: string;
}

export function PermissionAlert({ visible, platform }: PermissionAlertProps) {
  const { t } = useTranslation();

  if (!visible) {
    return null;
  }

  const isWindows = platform.startsWith("Win");

  async function restart() {
    if (!isWindows) {
      toast.info(t("permission.nonWindowsRestart"));
      return;
    }
    try {
      await envApi.restartAsAdmin();
    } catch (error) {
      showEnvError(error);
    }
  }

  return (
    <Alert className="border-warning bg-warning/10 text-warning">
      <TriangleAlertIcon />
      <AlertTitle>{t("permission.title")}</AlertTitle>
      <AlertDescription className="text-warning/90">
        {isWindows ? t("permission.windowsDesc") : t("permission.unixDesc")}
      </AlertDescription>
      <AlertAction>
        <Button type="button" size="sm" onClick={restart}>
          {t("permission.restartAsAdmin")}
        </Button>
      </AlertAction>
    </Alert>
  );
}
