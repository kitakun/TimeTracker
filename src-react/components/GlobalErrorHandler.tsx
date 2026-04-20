import { useEffect } from "react";
import { getSettings } from "../lib/tauri";
import { useToast } from "../lib/toast";

/**
 * Mounts global listeners for unhandled JS errors and promise rejections.
 * Only shows a toast when the user has enabled "show_unexpected_errors" in
 * App Settings → System.  Renders nothing itself.
 */
export default function GlobalErrorHandler() {
  const { toast } = useToast();

  useEffect(() => {
    let enabled = false;

    // Load setting once on mount; also re-read on settings-changed events.
    async function syncSetting() {
      try {
        const s = await getSettings();
        enabled = s.show_unexpected_errors ?? false;
      } catch {
        // ignore — keep previous value
      }
    }

    syncSetting();
    window.addEventListener("tt:settings-changed", syncSetting);

    function handleError(event: ErrorEvent) {
      if (!enabled) return;
      const msg = event.message ?? "Unknown error";
      const loc = event.filename ? ` (${event.filename}:${event.lineno})` : "";
      toast(`Error: ${msg}${loc}`, "error");
    }

    function handleRejection(event: PromiseRejectionEvent) {
      if (!enabled) return;
      const reason = event.reason;
      const msg =
        reason instanceof Error
          ? reason.message
          : typeof reason === "string"
          ? reason
          : JSON.stringify(reason);
      toast(`Unhandled rejection: ${msg}`, "error");
    }

    window.addEventListener("error", handleError);
    window.addEventListener("unhandledrejection", handleRejection);

    return () => {
      window.removeEventListener("error", handleError);
      window.removeEventListener("unhandledrejection", handleRejection);
      window.removeEventListener("tt:settings-changed", syncSetting);
    };
  }, [toast]);

  return null;
}
