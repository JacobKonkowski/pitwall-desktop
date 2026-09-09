import { useEffect, useState } from "react";
import { subscribeToasts, type ToastMessage } from "./toast";

const TTL_MS = 4500;

/** Renders transient toasts from {@link showToast}. Mount once in the shell. */
export function ToastHost() {
  const [items, setItems] = useState<ToastMessage[]>([]);

  useEffect(() => {
    return subscribeToasts((toast) => {
      setItems((prev) => [...prev.slice(-4), toast]);
      window.setTimeout(() => {
        setItems((prev) => prev.filter((t) => t.id !== toast.id));
      }, TTL_MS);
    });
  }, []);

  if (items.length === 0) return null;

  return (
    <div className="toast-host" aria-live="polite">
      {items.map((t) => (
        <div key={t.id} className={`toast toast-${t.kind}`} role="status">
          <span>{t.message}</span>
          <button
            type="button"
            className="toast-dismiss"
            aria-label="Dismiss"
            onClick={() => setItems((prev) => prev.filter((x) => x.id !== t.id))}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  );
}
