// Formatos en es-MX (punto decimal, como en México).
const oneDecimal = new Intl.NumberFormat("es-MX", { maximumFractionDigits: 1 });
const noDecimals = new Intl.NumberFormat("es-MX", { maximumFractionDigits: 0 });

const UNITS = ["B", "KB", "MB", "GB", "TB"];

export function bytes(value: number): string {
  let v = value;
  let unit = 0;
  while (v >= 1024 && unit < UNITS.length - 1) {
    v /= 1024;
    unit++;
  }
  const fmt = unit >= 3 ? oneDecimal : noDecimals;
  return `${fmt.format(v)} ${UNITS[unit]}`;
}

export function rate(bytesPerSecond: number): string {
  return `${bytes(bytesPerSecond)}/s`;
}

export function percent(value: number): string {
  return `${noDecimals.format(value)}%`;
}
