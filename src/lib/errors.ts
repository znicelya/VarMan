import { toast } from "sonner";
import i18n from "./i18n";
import type { EnvError } from "@/types/env";

export function toEnvError(error: unknown): EnvError {
  if (typeof error === "object" && error !== null && "code" in error && "message" in error) {
    return error as EnvError;
  }
  return {
    code: "UNKNOWN",
    message: error instanceof Error ? error.message : String(error),
  };
}

export function envErrorMessage(error: EnvError): string {
  return i18n.t(`errors.${error.code}`, { defaultValue: error.message });
}

export function showEnvError(error: unknown) {
  const envError = toEnvError(error);
  toast.error(envErrorMessage(envError), { description: envError.message });
}
