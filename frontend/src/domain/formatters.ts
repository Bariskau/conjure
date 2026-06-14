import type { CallLog, ExecutionStatus, ToolSummary } from "./types";

export function formatDuration(durationMs: number): string {
  if (durationMs >= 1000) {
    return `${(durationMs / 1000).toFixed(2)}s`;
  }

  return `${durationMs}ms`;
}

export function formatTimeout(timeoutMs: number): string {
  const seconds = timeoutMs / 1000;
  return seconds >= 60 ? `${Math.round(seconds / 60)}m` : `${Math.round(seconds)}s`;
}

export function formatRelativeTime(value?: string | null): string {
  if (!value) {
    return "never";
  }

  const then = new Date(value).getTime();
  const deltaSeconds = Math.max(0, Math.round((Date.now() - then) / 1000));

  if (deltaSeconds < 60) {
    return "now";
  }

  if (deltaSeconds < 3600) {
    return `${Math.round(deltaSeconds / 60)}m ago`;
  }

  if (deltaSeconds < 86_400) {
    return `${Math.round(deltaSeconds / 3600)}h ago`;
  }

  return `${Math.round(deltaSeconds / 86_400)}d ago`;
}

export function getToolStatus(tool: ToolSummary): ExecutionStatus | "idle" {
  return tool.last_run_at ? "success" : "idle";
}

export function summarizeLog(log: CallLog): string {
  const exit = log.exit_code == null ? "exit -" : `exit ${log.exit_code}`;
  return `${formatDuration(log.duration_ms)} · ${exit}`;
}
