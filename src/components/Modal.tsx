import { Show, createEffect, type Component } from "solid-js";
import { dialogState, setDialogState } from "../lib/dialog";

/** Renders the active prompt/confirm dialog requested via lib/dialog. */
const Modal: Component = () => {
  let inputRef: HTMLInputElement | undefined;

  createEffect(() => {
    if (dialogState()?.kind === "prompt") {
      queueMicrotask(() => inputRef?.focus());
    }
  });

  const close = (value: string | null) => {
    const st = dialogState();
    setDialogState(null);
    st?.resolve(value);
  };

  const submit = () => {
    const st = dialogState();
    if (!st) return;
    close(st.kind === "confirm" ? "" : (inputRef?.value ?? st.value));
  };

  return (
    <Show when={dialogState()}>
      {(st) => (
        <div class="modal-backdrop" onClick={() => close(null)}>
          <div class="modal" onClick={(e) => e.stopPropagation()}>
            <div class="modal-title">{st().title}</div>
            <div class="modal-label">{st().label}</div>
            <Show when={st().kind === "prompt"}>
              <input
                ref={inputRef}
                value={st().value}
                onKeyDown={(e) => {
                  if (e.key === "Enter") submit();
                  if (e.key === "Escape") close(null);
                }}
              />
            </Show>
            <div class="modal-actions">
              <button class="btn" onClick={() => close(null)}>
                Cancel
              </button>
              <button class="btn btn-primary" onClick={submit}>
                OK
              </button>
            </div>
          </div>
        </div>
      )}
    </Show>
  );
};

export default Modal;
