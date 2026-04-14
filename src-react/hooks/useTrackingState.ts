import { useEffect, useState, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import { getTrackingState, pauseTracking, resumeTracking } from "../lib/tauri";

// "idle" is an internal backend concept (OS idle time exceeded threshold).
// From the user's perspective it is indistinguishable from "paused" — we
// never surface it as a separate state in the UI.
export type TrackingUIState = "running" | "paused";

function normalise(raw: string): TrackingUIState {
  return raw === "running" ? "running" : "paused";
}

export function useTrackingState() {
  const [state, setState] = useState<TrackingUIState>("paused");

  useEffect(() => {
    getTrackingState()
      .then((s) => setState(normalise(s)))
      .catch(() => setState("paused"));

    const unlisten = listen<string>("tracking-state-changed", (e) => {
      setState(normalise(e.payload));
    });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const pause = useCallback(async () => {
    await pauseTracking();
    setState("paused");
  }, []);

  const resume = useCallback(async () => {
    await resumeTracking();
    setState("running");
  }, []);

  return { state, pause, resume };
}
