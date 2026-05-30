import { EditorState, type Extension } from "@codemirror/state";
import { EditorView, keymap } from "@codemirror/view";
import { basicSetup } from "codemirror";
import { oneDark } from "@codemirror/theme-one-dark";
import { indentWithTab } from "@codemirror/commands";
import { markdown } from "@codemirror/lang-markdown";
import { cpp } from "@codemirror/lang-cpp";
import { python } from "@codemirror/lang-python";
import { rust } from "@codemirror/lang-rust";
import { javascript } from "@codemirror/lang-javascript";
import { yaml } from "@codemirror/lang-yaml";

/** Pick a CodeMirror language extension from a file path. */
export function languageFor(path: string): Extension[] {
  const ext = path.split(".").pop()?.toLowerCase() ?? "";
  switch (ext) {
    case "md":
    case "markdown":
      return [markdown()];
    case "c":
    case "h":
    case "cpp":
    case "cc":
    case "cxx":
    case "hpp":
      return [cpp()];
    case "py":
      return [python()];
    case "rs":
      return [rust()];
    case "js":
    case "mjs":
    case "ts":
    case "tsx":
    case "jsx":
      return [javascript({ typescript: ext.startsWith("ts") })];
    case "yml":
    case "yaml":
      return [yaml()];
    default:
      return [];
  }
}

export interface EditorHandle {
  view: EditorView;
  setDoc: (doc: string) => void;
  destroy: () => void;
}

export function createEditor(opts: {
  parent: HTMLElement;
  doc: string;
  path: string;
  onChange?: (doc: string) => void;
  onSave?: () => void;
  readOnly?: boolean;
  wrap?: boolean;
}): EditorHandle {
  const saveKey = keymap.of([
    {
      key: "Mod-s",
      preventDefault: true,
      run: () => {
        opts.onSave?.();
        return true;
      },
    },
  ]);
  const extensions: Extension[] = [
    saveKey,
    basicSetup,
    keymap.of([indentWithTab]),
    oneDark,
    ...languageFor(opts.path),
  ];
  if (opts.wrap) extensions.push(EditorView.lineWrapping);
  if (opts.readOnly) extensions.push(EditorState.readOnly.of(true));
  if (opts.onChange) {
    extensions.push(
      EditorView.updateListener.of((u) => {
        if (u.docChanged) opts.onChange!(u.state.doc.toString());
      }),
    );
  }

  const view = new EditorView({
    parent: opts.parent,
    state: EditorState.create({ doc: opts.doc, extensions }),
  });

  return {
    view,
    setDoc(doc: string) {
      view.dispatch({
        changes: { from: 0, to: view.state.doc.length, insert: doc },
      });
    },
    destroy() {
      view.destroy();
    },
  };
}
