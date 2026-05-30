import { Show, type Component } from "solid-js";
import { state } from "../lib/store";
import CveTree from "../views/Exploits/CveTree";
import EnvList from "../views/Environments/EnvList";

const SideBar: Component = () => {
  return (
    <div class="sidebar-inner">
      <Show when={state.view === "exploits"} fallback={<EnvList />}>
        <CveTree />
      </Show>
    </div>
  );
};

export default SideBar;
