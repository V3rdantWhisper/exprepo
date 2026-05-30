import { For, Show, createEffect, createSignal, type Component } from "solid-js";
import { ChevronDown, Trash2 } from "lucide-solid";
import { state, setPanelTab, togglePanel, clearLogs } from "../lib/store";
import SourceControl from "../views/Exploits/SourceControl";

const LogView: Component = () => {
  const [selected, setSelected] = createSignal<string>("");
  let logEnd: HTMLDivElement | undefined;

  // Default the selection to the first running (or any logged) environment.
  createEffect(() => {
    if (!selected() || !state.envs.find((e) => e.id === selected())) {
      const candidate = state.runningEnvs[0] ?? Object.keys(state.logs)[0] ?? "";
      if (candidate) setSelected(candidate);
    }
  });

  const lines = () => state.logs[selected()] ?? [];

  createEffect(() => {
    lines().length;
    logEnd?.scrollIntoView({ block: "end" });
  });

  return (
    <div class="logview">
      <div class="logview-bar">
        <select value={selected()} onChange={(e) => setSelected(e.currentTarget.value)}>
          <option value="">— select environment —</option>
          <For each={state.envs}>
            {(env) => (
              <option value={env.id}>
                {env.name}
                {state.runningEnvs.includes(env.id) ? " (running)" : ""}
              </option>
            )}
          </For>
        </select>
        <button class="row-action" title="Clear" onClick={() => selected() && clearLogs(selected())}>
          <Trash2 size={13} />
        </button>
      </div>
      <div class="logview-body">
        <For each={lines()}>
          {(l) => <div class={`logline ${l.stream}`}>{l.line}</div>}
        </For>
        <div ref={logEnd} />
      </div>
    </div>
  );
};

const Panel: Component = () => {
  const tabBtn = (active: boolean) => `panel-tab${active ? " active" : ""}`;
  return (
    <div class="panel">
      <div class="panel-header">
        <div class="panel-tabs">
          <button class={tabBtn(state.panelTab === "logs")} onClick={() => setPanelTab("logs")}>
            Logs
          </button>
          <button class={tabBtn(state.panelTab === "scm")} onClick={() => setPanelTab("scm")}>
            Source Control
          </button>
        </div>
        <button class="row-action" title="Close panel" onClick={() => togglePanel(false)}>
          <ChevronDown size={16} />
        </button>
      </div>
      <div class="panel-body">
        <Show when={state.panelTab === "logs"} fallback={<SourceControl />}>
          <LogView />
        </Show>
      </div>
    </div>
  );
};

export default Panel;
