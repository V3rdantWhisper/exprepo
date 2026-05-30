import { Show, createSignal, onCleanup, onMount, type Component } from "solid-js";
import { Save, Upload } from "lucide-solid";
import * as api from "../../lib/api";
import { state, refreshEnvs, reportError, toast } from "../../lib/store";
import { createEditor, type EditorHandle } from "../../lib/editor";
import { renderMarkdown } from "../../lib/markdown";

/** Edits an environment's inline build-guide markdown. Can export it into the
 *  exp repo's `guides/` dir so it gets committed/synced. */
const GuideEditor: Component<{ envId: string }> = (props) => {
  const env = () => state.envs.find((e) => e.id === props.envId);
  const [content, setContent] = createSignal(env()?.build_guide ?? "");
  const [dirty, setDirty] = createSignal(false);
  const [previewHtml, setPreviewHtml] = createSignal("");
  let host!: HTMLDivElement;
  let handle: EditorHandle | undefined;
  let previewTimer: number | undefined;

  const updatePreview = (doc: string) => {
    clearTimeout(previewTimer);
    previewTimer = window.setTimeout(() => setPreviewHtml(renderMarkdown(doc)), 150);
  };

  const save = async () => {
    const e = env();
    if (!e) return;
    try {
      await api.envSave({ ...e, build_guide: content() });
      setDirty(false);
      await refreshEnvs();
      toast("Guide saved");
    } catch (err) {
      reportError(err);
    }
  };

  const exportToRepo = async () => {
    await save();
    try {
      const rel = await api.envExportGuide(props.envId);
      toast(`Exported to ${rel} — commit & push to sync`);
    } catch (err) {
      reportError(err);
    }
  };

  onMount(() => {
    const initial = env()?.build_guide ?? "";
    setContent(initial);
    updatePreview(initial);
    handle = createEditor({
      parent: host,
      doc: initial,
      path: "guide.md",
      wrap: true,
      onChange: (doc) => {
        setContent(doc);
        setDirty(true);
        updatePreview(doc);
      },
      onSave: save,
    });
  });

  onCleanup(() => handle?.destroy());

  return (
    <div class="editor-doc">
      <div class="editor-toolbar">
        <span class="doc-path">
          Build guide — {env()?.name}
          <Show when={dirty()}>
            <span class="dirty-dot"> ●</span>
          </Show>
        </span>
        <span class="toolbar-right">
          <button class="btn btn-sm" onClick={save} title="Save (Ctrl/Cmd+S)">
            <Save size={13} /> Save
          </button>
          <button class="btn btn-sm" onClick={exportToRepo} title="Export to repo guides/ for syncing">
            <Upload size={13} /> Export to repo
          </button>
        </span>
      </div>
      <div class="editor-body split">
        <div class="cm-host" ref={host} />
        <div class="md-preview markdown-body" innerHTML={previewHtml()} />
      </div>
    </div>
  );
};

export default GuideEditor;
