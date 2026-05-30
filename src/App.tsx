import { Show, onMount, type Component } from "solid-js";
import { listen } from "@tauri-apps/api/event";
import ActivityBar from "./components/ActivityBar";
import SideBar from "./components/SideBar";
import EditorArea from "./components/EditorArea";
import Panel from "./components/Panel";
import Splitter from "./components/Splitter";
import Modal from "./components/Modal";
import {
  state,
  setSidebarWidth,
  setPanelHeight,
  appendLog,
  setRunningEnvs,
  refreshConfig,
  refreshCves,
  refreshEnvs,
  refreshStatus,
} from "./lib/store";
import * as api from "./lib/api";
import { initHighlighter } from "./lib/markdown";
import "./styles/shell.css";

interface LogPayload {
  env_id: string;
  stream: string;
  line: string;
}
interface ExitPayload {
  env_id: string;
  code: number | null;
}

const App: Component = () => {
  onMount(async () => {
    initHighlighter();
    await refreshConfig();
    await Promise.all([refreshCves(), refreshEnvs(), refreshStatus()]);

    await listen<LogPayload>("env-log", (e) => {
      appendLog(e.payload.env_id, e.payload.stream, e.payload.line);
    });
    await listen<ExitPayload>("env-exit", async (e) => {
      appendLog(
        e.payload.env_id,
        "stderr",
        `\n[process exited${e.payload.code === null ? "" : ` with code ${e.payload.code}`}]`,
      );
      setRunningEnvs(await api.envRunning());
    });
  });

  return (
    <div class="app">
      <ActivityBar />
      <div class="sidebar" style={{ width: `${state.sidebarWidth}px` }}>
        <SideBar />
      </div>
      <Splitter orientation="vertical" onDelta={(d) => setSidebarWidth(state.sidebarWidth + d)} />
      <div class="main">
        <EditorArea />
        <Show when={state.panelOpen}>
          <Splitter
            orientation="horizontal"
            onDelta={(d) => setPanelHeight(state.panelHeight - d)}
          />
          <div class="panel-wrap" style={{ height: `${state.panelHeight}px` }}>
            <Panel />
          </div>
        </Show>
      </div>

      <Modal />
      <Show when={state.toast}>
        {(t) => <div class={`toast toast-${t().kind}`}>{t().message}</div>}
      </Show>
    </div>
  );
};

export default App;
