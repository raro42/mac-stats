// Rust admite una sola suscripción a las métricas (hay una sola ventana): este módulo
// la comparte entre el monitor y el chat.
import { subscribeMetrics, type Snapshot } from "./ipc";

type Listener = (snapshot: Snapshot) => void;

const listeners = new Set<Listener>();
let latest: Snapshot | null = null;
let subscription: Promise<void> | null = null;

/** Registra `listener` y devuelve la última lectura disponible. */
export async function onMetrics(listener: Listener): Promise<Snapshot | null> {
  listeners.add(listener);
  subscription ??= subscribeMetrics((snapshot) => {
    latest = snapshot;
    for (const l of listeners) l(snapshot);
  }).then((first) => {
    latest ??= first;
  });
  await subscription;
  return latest;
}
