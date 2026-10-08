// Number and unit formatting for the active language and region (see `i18n/locale()`).
// Sizes are 1024-based like the rest of the app, but use the locale's unit names
// (for example "Go" in French). Values under 1 kB are shown as a fraction of a kilobyte.
import { locale, t } from "./i18n";

const UNITS = ["kilobyte", "megabyte", "gigabyte", "terabyte"] as const;

const cache = new Map<string, Intl.NumberFormat>();

function formatter(options: Intl.NumberFormatOptions): Intl.NumberFormat {
  const key = `${locale()}|${JSON.stringify(options)}`;
  let fmt = cache.get(key);
  if (!fmt) {
    fmt = new Intl.NumberFormat(locale(), options);
    cache.set(key, fmt);
  }
  return fmt;
}

function scaled(value: number): { v: number; unit: (typeof UNITS)[number]; digits: number } {
  let v = value / 1024;
  let i = 0;
  while (v >= 1024 && i < UNITS.length - 1) {
    v /= 1024;
    i++;
  }
  // One decimal from GB up and below 1 kB; whole numbers for kB and MB.
  const digits = i >= 2 || v < 1 ? 1 : 0;
  return { v, unit: UNITS[i], digits };
}

export function bytes(value: number): string {
  const { v, unit, digits } = scaled(value);
  return formatter({ style: "unit", unit, unitDisplay: "short", maximumFractionDigits: digits }).format(v);
}

export function rate(bytesPerSecond: number): string {
  const { v, unit, digits } = scaled(bytesPerSecond);
  return formatter({
    style: "unit",
    unit: `${unit}-per-second`,
    unitDisplay: "short",
    maximumFractionDigits: digits,
  }).format(v);
}

/** `value` is 0–100. */
export function percent(value: number): string {
  return formatter({ style: "percent", maximumFractionDigits: 0 }).format(value / 100);
}

export function number(value: number, fractionDigits = 0): string {
  return formatter({ minimumFractionDigits: fractionDigits, maximumFractionDigits: fractionDigits }).format(value);
}

export function seconds(value: number): string {
  return formatter({ style: "unit", unit: "second", unitDisplay: "short", maximumFractionDigits: 1 }).format(value);
}

export function duration(value: number, unit: "minute" | "hour"): string {
  return formatter({ style: "unit", unit, unitDisplay: "short" }).format(value);
}

export function tokensPerSecond(value: number): string {
  return t("unit.tokensPerSecond", { value: number(value, 1) });
}

/** Joins parts with the locale's separator (" · "), skipping empty ones. */
export function joined(parts: (string | null | undefined | false)[]): string {
  return parts.filter(Boolean).join(t("common.separator"));
}
