import { format, parseISO } from "date-fns";

export function formatDurationHuman(secs: number): string {
  if (secs <= 0) return "0m";
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  if (h > 0 && m > 0) return `${h}h ${m}m`;
  if (h > 0) return `${h}h`;
  return `${m}m`;
}

export function formatDurationJira(secs: number): string {
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const parts: string[] = [];
  if (h > 0) parts.push(`${h}h`);
  if (m > 0) parts.push(`${m}m`);
  return parts.join(" ") || "0m";
}

export function formatTime(isoString: string): string {
  try {
    return format(parseISO(isoString), "HH:mm:ss");
  } catch {
    return "–";
  }
}

export function formatDateLabel(isoString: string): string {
  try {
    return format(parseISO(isoString), "EEE d MMM yyyy");
  } catch {
    return isoString;
  }
}

export function todayDate(): string {
  return format(new Date(), "yyyy-MM-dd");
}

export function totalDurationSecs(sessions: { duration_secs: number }[]): number {
  return sessions.reduce((acc, s) => acc + s.duration_secs, 0);
}
