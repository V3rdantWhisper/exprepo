import { createSignal } from "solid-js";

export interface PromptRequest {
  kind: "prompt" | "confirm";
  title: string;
  label: string;
  value: string;
  resolve: (value: string | null) => void;
}

export const [dialogState, setDialogState] = createSignal<PromptRequest | null>(null);

/** Ask for a line of text. Resolves to the string, or null if cancelled. */
export function promptText(title: string, label: string, value = ""): Promise<string | null> {
  return new Promise((resolve) =>
    setDialogState({ kind: "prompt", title, label, value, resolve }),
  );
}

/** Ask for confirmation. Resolves to "" if confirmed, null if cancelled. */
export function confirmAction(title: string, label: string): Promise<string | null> {
  return new Promise((resolve) =>
    setDialogState({ kind: "confirm", title, label, value: "", resolve }),
  );
}
