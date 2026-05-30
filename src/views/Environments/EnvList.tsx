import { For, Show, type Component } from "solid-js";
import { Plus, RefreshCw, Play, Square, Server, FileText, Trash2 } from "lucide-solid";
import * as api from "../../lib/api";
import {
  state,
  refreshEnvs,
  openEnvTab,
  openGuideTab,
  reportError,
  toast,
  setPanelTab,
  togglePanel,
} from "../../lib/store";
import type { Environment } from "../../lib/api";
import { confirmAction } from "../../lib/dialog";

const EnvList: Component = () => {
  const running = (id: string) => state.runningEnvs.includes(id);

  const launch = async (env: Environment) => {
    try {
      await api.envLaunch(env.id);
      setPanelTab("logs");
      togglePanel(true);
      await refreshEnvs();
    } catch (e) {
      reportError(e);
    }
  };

  const stop = async (env: Environment) => {
    try {
      await api.envStop(env.id);
    } catch (e) {
      reportError(e);
    }
  };

  const remove = async (env: Environment) => {
    if ((await confirmAction("Delete environment", `Delete "${env.name}"?`)) === null) return;
    try {
      await api.envDelete(env.id);
      await refreshEnvs();
      toast("Environment deleted");
    } catch (e) {
      reportError(e);
    }
  };

  return (
    <div class="tree">
      <div class="sidebar-header">
        <span>ENVIRONMENTS</span>
        <span class="sidebar-actions">
          <button title="New environment" onClick={() => openEnvTab(null)}>
            <Plus size={16} />
          </button>
          <button title="Refresh" onClick={refreshEnvs}>
            <RefreshCw size={14} />
          </button>
        </span>
      </div>

      <Show
        when={state.envs.length > 0}
        fallback={<div class="tree-empty">No environments. Click + to add one.</div>}
      >
        <For each={state.envs}>
          {(env) => (
            <div class="tree-row env-row" onClick={() => openEnvTab(env.id)}>
              <Server size={14} class={`tree-icon ${running(env.id) ? "running" : ""}`} />
              <span class="tree-label">
                {env.name || "(unnamed)"}
                <Show when={running(env.id)}>
                  <span class="badge-running">running</span>
                </Show>
              </span>
              <Show
                when={running(env.id)}
                fallback={
                  <button class="row-action" title="Launch" onClick={(e) => { e.stopPropagation(); launch(env); }}>
                    <Play size={14} />
                  </button>
                }
              >
                <button class="row-action danger" title="Stop" onClick={(e) => { e.stopPropagation(); stop(env); }}>
                  <Square size={14} />
                </button>
              </Show>
              <button class="row-action" title="Build guide" onClick={(e) => { e.stopPropagation(); openGuideTab(env.id); }}>
                <FileText size={14} />
              </button>
              <button class="row-action danger" title="Delete" onClick={(e) => { e.stopPropagation(); remove(env); }}>
                <Trash2 size={14} />
              </button>
            </div>
          )}
        </For>
      </Show>
    </div>
  );
};

export default EnvList;
