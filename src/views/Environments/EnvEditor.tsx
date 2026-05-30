import { Show, createMemo, type Component } from "solid-js";
import { createStore } from "solid-js/store";
import { open } from "@tauri-apps/plugin-dialog";
import { Play, Square, FileText, FolderOpen } from "lucide-solid";
import * as api from "../../lib/api";
import {
  state,
  refreshEnvs,
  reportError,
  toast,
  closeTab,
  openEnvTab,
  openGuideTab,
  setPanelTab,
  togglePanel,
} from "../../lib/store";
import type { Environment } from "../../lib/api";

const ARCHES = ["x86_64", "aarch64", "arm", "riscv64", "i386", "mips"];

const EnvEditor: Component<{ envId: string | null }> = (props) => {
  const existing = (): Environment | undefined =>
    props.envId ? state.envs.find((e) => e.id === props.envId) : undefined;

  const [form, setForm] = createStore<Environment>(
    existing() ? { ...existing()! } : api.emptyEnvironment(),
  );

  const running = createMemo(() => !!form.id && state.runningEnvs.includes(form.id));

  const browse = async (field: "kernel_image" | "rootfs", directory = false) => {
    const picked = await open({ directory, multiple: false });
    if (typeof picked === "string") setForm(field, picked);
  };

  const browseDir = async (field: "working_dir") => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") setForm(field, picked);
  };

  const save = async () => {
    if (!form.name.trim()) {
      reportError("name is required");
      return;
    }
    try {
      const saved = await api.envSave({ ...form });
      await refreshEnvs();
      toast("Environment saved");
      if (!props.envId) {
        closeTab("env:new");
        openEnvTab(saved.id);
      }
    } catch (e) {
      reportError(e);
    }
  };

  const launch = async () => {
    try {
      await api.envLaunch(form.id);
      setPanelTab("logs");
      togglePanel(true);
      await refreshEnvs();
    } catch (e) {
      reportError(e);
    }
  };

  const stop = async () => {
    try {
      await api.envStop(form.id);
    } catch (e) {
      reportError(e);
    }
  };

  return (
    <div class="form-doc">
      <div class="form-head">
        <h2>{props.envId ? "Edit environment" : "New environment"}</h2>
        <div class="form-head-actions">
          <Show when={form.id}>
            <button class="btn" onClick={() => openGuideTab(form.id)}>
              <FileText size={14} /> Build guide
            </button>
            <Show
              when={running()}
              fallback={
                <button class="btn btn-primary" onClick={launch}>
                  <Play size={14} /> Launch
                </button>
              }
            >
              <button class="btn btn-danger" onClick={stop}>
                <Square size={14} /> Stop
              </button>
            </Show>
          </Show>
        </div>
      </div>

      <label class="field">
        <span>Name</span>
        <input value={form.name} onInput={(e) => setForm("name", e.currentTarget.value)} />
      </label>

      <label class="field">
        <span>Architecture</span>
        <select value={form.arch} onChange={(e) => setForm("arch", e.currentTarget.value)}>
          {ARCHES.map((a) => (
            <option value={a}>{a}</option>
          ))}
        </select>
      </label>

      <label class="field">
        <span>QEMU binary</span>
        <input
          placeholder={`qemu-system-${form.arch || "x86_64"} (default)`}
          value={form.qemu_binary}
          onInput={(e) => setForm("qemu_binary", e.currentTarget.value)}
        />
      </label>

      <label class="field">
        <span>Kernel image</span>
        <div class="field-row">
          <input
            value={form.kernel_image ?? ""}
            onInput={(e) => setForm("kernel_image", e.currentTarget.value || null)}
          />
          <button class="btn" onClick={() => browse("kernel_image")}>
            <FolderOpen size={14} />
          </button>
        </div>
      </label>

      <label class="field">
        <span>Root filesystem</span>
        <div class="field-row">
          <input
            value={form.rootfs ?? ""}
            onInput={(e) => setForm("rootfs", e.currentTarget.value || null)}
          />
          <button class="btn" onClick={() => browse("rootfs")}>
            <FolderOpen size={14} />
          </button>
        </div>
      </label>

      <label class="field">
        <span>Kernel cmdline (-append)</span>
        <input
          value={form.append ?? ""}
          placeholder="console=ttyS0 root=/dev/sda rw"
          onInput={(e) => setForm("append", e.currentTarget.value || null)}
        />
      </label>

      <label class="field">
        <span>Extra QEMU args</span>
        <input
          value={form.extra_args}
          placeholder="-nographic -m 1G -smp 2"
          onInput={(e) => setForm("extra_args", e.currentTarget.value)}
        />
      </label>

      <label class="field">
        <span>Working directory</span>
        <div class="field-row">
          <input
            value={form.working_dir ?? ""}
            onInput={(e) => setForm("working_dir", e.currentTarget.value || null)}
          />
          <button class="btn" onClick={() => browseDir("working_dir")}>
            <FolderOpen size={14} />
          </button>
        </div>
      </label>

      <div class="form-actions">
        <button class="btn btn-primary" onClick={save}>
          Save
        </button>
      </div>

      <p class="form-hint">
        Tip: add <code>-nographic</code> (or <code>-serial stdio</code>) to stream the guest
        console into the Logs panel below.
      </p>
    </div>
  );
};

export default EnvEditor;
