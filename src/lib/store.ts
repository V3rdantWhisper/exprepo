import { createStore, produce } from "solid-js/store";
import * as api from "./api";
import type { AppConfig, Cve, Environment, RepoStatus } from "./api";

export type View = "exploits" | "environments";
export type PanelTab = "logs" | "scm";

/** An open editor tab. */
export type Tab =
  | { kind: "markdown"; id: string; title: string; path: string }
  | { kind: "source"; id: string; title: string; path: string }
  | { kind: "env"; id: string; title: string; envId: string | null }
  | { kind: "guide"; id: string; title: string; envId: string }
  | { kind: "settings"; id: string; title: string };

export interface LogLine {
  stream: string;
  line: string;
}

interface State {
  view: View;
  cves: Cve[];
  envs: Environment[];
  runningEnvs: string[];
  tabs: Tab[];
  activeTabId: string | null;
  status: RepoStatus | null;
  config: AppConfig | null;
  hasToken: boolean;
  panelOpen: boolean;
  panelTab: PanelTab;
  sidebarWidth: number;
  panelHeight: number;
  logs: Record<string, LogLine[]>;
  toast: { message: string; kind: "info" | "error" } | null;
}

const [state, setState] = createStore<State>({
  view: "exploits",
  cves: [],
  envs: [],
  runningEnvs: [],
  tabs: [],
  activeTabId: null,
  status: null,
  config: null,
  hasToken: false,
  panelOpen: false,
  panelTab: "logs",
  sidebarWidth: 280,
  panelHeight: 220,
  logs: {},
  toast: null,
});

export { state };

// ---- UI actions ----

export const setView = (view: View) => setState("view", view);
export const setSidebarWidth = (w: number) =>
  setState("sidebarWidth", Math.max(180, Math.min(600, w)));
export const setPanelHeight = (h: number) =>
  setState("panelHeight", Math.max(120, Math.min(600, h)));
export const togglePanel = (open?: boolean) =>
  setState("panelOpen", open ?? !state.panelOpen);
export const setPanelTab = (t: PanelTab) => setState("panelTab", t);

let toastTimer: number | undefined;
export function toast(message: string, kind: "info" | "error" = "info") {
  setState("toast", { message, kind });
  clearTimeout(toastTimer);
  toastTimer = window.setTimeout(() => setState("toast", null), 4000);
}

export function reportError(e: unknown) {
  toast(typeof e === "string" ? e : (e as Error)?.message ?? String(e), "error");
}

// ---- Tabs ----

export function openTab(tab: Tab) {
  if (!state.tabs.find((t) => t.id === tab.id)) {
    setState("tabs", (tabs) => [...tabs, tab]);
  }
  setState("activeTabId", tab.id);
}

export function closeTab(id: string) {
  const idx = state.tabs.findIndex((t) => t.id === id);
  if (idx < 0) return;
  setState(
    produce((s) => {
      s.tabs.splice(idx, 1);
      if (s.activeTabId === id) {
        const next = s.tabs[idx] ?? s.tabs[idx - 1] ?? null;
        s.activeTabId = next ? next.id : null;
      }
    }),
  );
}

export const setActiveTab = (id: string) => setState("activeTabId", id);

export function openFileTab(path: string) {
  const title = path.split("/").pop() ?? path;
  const kind: Tab["kind"] = path.endsWith(".md") ? "markdown" : "source";
  openTab({ kind, id: `file:${path}`, title, path } as Tab);
}

export function openEnvTab(envId: string | null) {
  const env = envId ? state.envs.find((e) => e.id === envId) : null;
  openTab({
    kind: "env",
    id: envId ? `env:${envId}` : "env:new",
    title: env ? env.name || "(env)" : "New environment",
    envId,
  });
}

export function openGuideTab(envId: string) {
  const env = state.envs.find((e) => e.id === envId);
  openTab({
    kind: "guide",
    id: `guide:${envId}`,
    title: `Guide: ${env?.name ?? envId}`,
    envId,
  });
}

// ---- Data loading ----

export async function refreshConfig() {
  try {
    setState("config", await api.getConfig());
    setState("hasToken", await api.secretHasToken());
  } catch (e) {
    reportError(e);
  }
}

export async function refreshCves() {
  try {
    setState("cves", await api.cveList());
  } catch (e) {
    reportError(e);
  }
}

export async function refreshEnvs() {
  try {
    setState("envs", await api.envList());
    setState("runningEnvs", await api.envRunning());
  } catch (e) {
    reportError(e);
  }
}

export async function refreshStatus() {
  try {
    setState("status", await api.repoStatus());
  } catch (e) {
    // Repo may not be initialized yet; surface as null rather than a toast.
    setState("status", null);
  }
}

// ---- Log streaming ----

export function appendLog(envId: string, stream: string, line: string) {
  setState(
    produce((s) => {
      if (!s.logs[envId]) s.logs[envId] = [];
      s.logs[envId].push({ stream, line });
      if (s.logs[envId].length > 5000) s.logs[envId].splice(0, 1000);
    }),
  );
}

export const clearLogs = (envId: string) => setState("logs", envId, []);
export const setRunningEnvs = (ids: string[]) => setState("runningEnvs", ids);
