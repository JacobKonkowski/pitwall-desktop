/** Lightweight pub/sub toast bus used by the shell and feature pages. */

export type ToastKind = "info" | "success" | "error";

export interface ToastMessage {
  id: number;
  message: string;
  kind: ToastKind;
}

type Listener = (toast: ToastMessage) => void;

const listeners = new Set<Listener>();
let nextId = 1;

export function showToast(message: string, kind: ToastKind = "info"): void {
  const toast: ToastMessage = { id: nextId++, message, kind };
  listeners.forEach((l) => l(toast));
}

export function subscribeToasts(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
