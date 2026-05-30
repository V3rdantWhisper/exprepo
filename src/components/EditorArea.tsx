import { For, Show, type Component } from "solid-js";
import { X } from "lucide-solid";
import { state, closeTab, setActiveTab, type Tab } from "../lib/store";
import FileEditor from "../views/Exploits/FileEditor";
import EnvEditor from "../views/Environments/EnvEditor";
import GuideEditor from "../views/Environments/GuideEditor";
import Settings from "../views/Settings";

function renderTab(tab: Tab) {
  switch (tab.kind) {
    case "markdown":
      return <FileEditor path={tab.path} preview={true} />;
    case "source":
      return <FileEditor path={tab.path} preview={false} />;
    case "env":
      return <EnvEditor envId={tab.envId} />;
    case "guide":
      return <GuideEditor envId={tab.envId} />;
    case "settings":
      return <Settings />;
  }
}

const Welcome: Component = () => (
  <div class="welcome">
    <h1>ExpRepo</h1>
    <p>Select a writeup or environment from the sidebar, or create a new one.</p>
  </div>
);

const EditorArea: Component = () => {
  return (
    <div class="editor-area">
      <Show when={state.tabs.length > 0} fallback={<Welcome />}>
        <div class="tab-strip">
          <For each={state.tabs}>
            {(tab) => (
              <div
                class={`tab${state.activeTabId === tab.id ? " active" : ""}`}
                onClick={() => setActiveTab(tab.id)}
                onAuxClick={(e) => {
                  if (e.button === 1) closeTab(tab.id);
                }}
              >
                <span class="tab-title">{tab.title}</span>
                <button
                  class="tab-close"
                  onClick={(e) => {
                    e.stopPropagation();
                    closeTab(tab.id);
                  }}
                >
                  <X size={13} />
                </button>
              </div>
            )}
          </For>
        </div>
        <div class="editor-content">
          {/* Keep each open tab mounted so editor state survives tab switches. */}
          <For each={state.tabs}>
            {(tab) => (
              <div
                class="editor-pane"
                style={{ display: state.activeTabId === tab.id ? "flex" : "none" }}
              >
                {renderTab(tab)}
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};

export default EditorArea;
