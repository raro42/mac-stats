// Turns errors into translated text. Rust commands reject with `{ code, params }` (see
// `src-tauri/src/error.rs`); anything else is unexpected, so it is logged and shown as a
// generic message instead of leaking raw system text into the UI.
import { bytes } from "../format";
import { isErrorCode, type ErrorCode } from "./error-codes";
import { t, type TextKey } from "./index";

/** Error parameters that hold byte counts and are shown as sizes. */
const BYTE_PARAMS = new Set(["needed", "available", "size"]);

export interface AppError {
  code: ErrorCode;
  params?: Record<string, string | number>;
}

export function isAppError(error: unknown): error is AppError {
  return typeof error === "object" && error !== null && isErrorCode((error as { code?: unknown }).code);
}

export function errorText(error: unknown): string {
  if (!isAppError(error)) {
    console.error("unexpected error", error);
    return t("error.internal");
  }
  const params: Record<string, string | number> = {};
  for (const [name, value] of Object.entries(error.params ?? {})) {
    params[name] = BYTE_PARAMS.has(name) && typeof value === "number" ? bytes(value) : value;
  }
  return t(`error.${error.code}` as TextKey, params);
}
