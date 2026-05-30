import type { Component } from "solid-js";

/** A draggable divider. Calls `onDelta` with the pointer movement in px. */
const Splitter: Component<{
  orientation: "vertical" | "horizontal";
  onDelta: (delta: number) => void;
}> = (props) => {
  const onPointerDown = (e: PointerEvent) => {
    e.preventDefault();
    const start = props.orientation === "vertical" ? e.clientX : e.clientY;
    let last = start;
    const move = (ev: PointerEvent) => {
      const cur = props.orientation === "vertical" ? ev.clientX : ev.clientY;
      props.onDelta(cur - last);
      last = cur;
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
      document.body.style.cursor = "";
      document.body.style.userSelect = "";
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
    document.body.style.cursor =
      props.orientation === "vertical" ? "col-resize" : "row-resize";
    document.body.style.userSelect = "none";
  };

  return (
    <div
      class={`splitter splitter-${props.orientation}`}
      onPointerDown={onPointerDown}
    />
  );
};

export default Splitter;
