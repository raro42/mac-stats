// Error codes shared with Rust (`src-tauri/src/error.rs`) and Swift (`LlmPlugin.swift`).
// This array is the single list: a Rust unit test and `scripts/check-i18n.sh` read it,
// and every code needs an `error.<code>` message in each dictionary.
// Keep one quoted code per line; the Rust test parses this array.
export const ERROR_CODES = [
  "internal",
  "storage",
  "network",
  "network_offline",
  "network_timeout",
  "cellular_not_allowed",
  "empty_question",
  "too_hot",
  "app_in_background",
  "model_unknown",
  "model_not_installed",
  "model_not_loaded",
  "model_file_missing",
  "model_load_failed",
  "not_enough_memory",
  "context_failed",
  "chat_template_failed",
  "tokenize_failed",
  "conversation_too_long",
  "generation_failed",
  "download_busy",
  "download_cancelled",
  "download_failed",
  "not_enough_storage",
  "checksum_mismatch",
  "insecure_url",
] as const;

export type ErrorCode = (typeof ERROR_CODES)[number];

export function isErrorCode(value: unknown): value is ErrorCode {
  return typeof value === "string" && (ERROR_CODES as readonly string[]).includes(value);
}
