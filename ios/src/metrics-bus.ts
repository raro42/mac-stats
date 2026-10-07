// Rust supports a single metrics subscription (there is only one window): this module
// shares it between the monitor and the chat.
import { subscribeMetrics, type Snapshot } from "./ipc";

type Listener = (snapshot: Snapshot) => void;

const listeners = new Set<Listener>();
let latest: Snapshot | null = null;
let subscription: Promise<void> | null = null;

/** Registers `listener` and returns the latest available reading. */
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
