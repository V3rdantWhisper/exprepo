import type { Component } from "solid-js";
import { Bug, Server, GitBranch, Settings } from "lucide-solid";
import { state, setView, togglePanel, setPanelTab, openTab } from "../lib/store";

const ActivityBar: Component = () => {
  const item = (active: boolean) =>
    `activity-item${active ? " active" : ""}`;

  return (
    <div class="activity-bar">
      <button
        class={item(state.view === "exploits")}
        title="Exploits (CVEs)"
        onClick={() => setView("exploits")}
      >
        <Bug size={24} />
      </button>
      <button
        class={item(state.view === "environments")}
        title="Environments"
        onClick={() => setView("environments")}
      >
        <Server size={24} />
      </button>
      <button
        class={item(state.panelOpen && state.panelTab === "scm")}
        title="Source Control / Sync"
        onClick={() => {
          setPanelTab("scm");
          togglePanel(true);
        }}
      >
        <GitBranch size={24} />
      </button>
      <div class="activity-spacer" />
      <button
        class="activity-item"
        title="Settings"
        onClick={() => openTab({ kind: "settings", id: "settings", title: "Settings" })}
      >
        <Settings size={24} />
      </button>
    </div>
  );
};

export default ActivityBar;
