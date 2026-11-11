import { getCurrentWindow } from "@tauri-apps/api/window";
import { Minus, Square, X } from "lucide-react";

export function WindowControls() {
  const win = getCurrentWindow();

  return (
    <div className="window-controls">
      <button
        type="button"
        className="window-control"
        aria-label="Minimize"
        onClick={() => win.minimize()}
      >
        <Minus size={18} strokeWidth={1.5} />
      </button>
      <button
        type="button"
        className="window-control"
        aria-label="Maximize"
        onClick={() => win.toggleMaximize()}
      >
        <Square size={16} strokeWidth={1.5} />
      </button>
      <button
        type="button"
        className="window-control window-control-close"
        aria-label="Close"
        onClick={() => win.close()}
      >
        <X size={18} strokeWidth={1.5} />
      </button>
    </div>
  );
}
