import Decimal from "decimal.js";

/**
 * All financial values arrive from the backend as decimal strings (e.g. "102.45").
 * Never coerce them through JS `number` for arithmetic — only ever for display,
 * and only through Decimal.js so rounding is deterministic.
 */

type MoneyOptions = {
  currency?: string;
  minimumFractionDigits?: number;
  maximumFractionDigits?: number;
  locale?: string;
};

function toDecimal(value: string | number | Decimal): Decimal {
  try {
    return new Decimal(value);
  } catch {
    return new Decimal(0);
  }
}

export function formatMoney(
  value: string | number | undefined | null,
  options: MoneyOptions = {},
): string {
  if (value === undefined || value === null || value === "") return "—";
  const {
    currency = "NGN",
    minimumFractionDigits = 2,
    maximumFractionDigits = 2,
    locale = "en-NG",
  } = options;

  const decimal = toDecimal(value);

  const formatter = new Intl.NumberFormat(locale, {
    style: "currency",
    currency,
    minimumFractionDigits,
    maximumFractionDigits,
  });

  return formatter.format(decimal.toNumber());
}

export function formatCompactMoney(
  value: string | number | undefined | null,
  currency = "NGN",
): string {
  if (value === undefined || value === null || value === "") return "—";
  const decimal = toDecimal(value);

  const formatter = new Intl.NumberFormat("en-NG", {
    style: "currency",
    currency,
    notation: "compact",
    maximumFractionDigits: 1,
  });

  return formatter.format(decimal.toNumber());
}

export function formatPrice(
  value: string | number | undefined | null,
  decimals = 2,
): string {
  if (value === undefined || value === null || value === "") return "—";
  const decimal = toDecimal(value);
  return decimal.toFixed(decimals);
}

export function formatPercentage(
  value: string | number | undefined | null,
  options: { decimals?: number; signed?: boolean } = {},
): string {
  if (value === undefined || value === null || value === "") return "—";
  const { decimals = 2, signed = false } = options;
  const decimal = toDecimal(value);
  const sign = signed && decimal.gt(0) ? "+" : "";
  return `${sign}${decimal.toFixed(decimals)}%`;
}

export function formatQuantity(
  value: string | number | undefined | null,
  decimals = 4,
): string {
  if (value === undefined || value === null || value === "") return "—";
  const decimal = toDecimal(value);
  return new Intl.NumberFormat("en-US", {
    minimumFractionDigits: 0,
    maximumFractionDigits: decimals,
  }).format(decimal.toNumber());
}

export function formatDate(
  value: string | undefined | null,
  options: Intl.DateTimeFormatOptions = {
    year: "numeric",
    month: "short",
    day: "numeric",
  },
): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "—";
  return new Intl.DateTimeFormat("en-US", options).format(date);
}

export function formatDateTime(value: string | undefined | null): string {
  return formatDate(value, {
    year: "numeric",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatRelativeTime(value: string | undefined | null): string {
  if (!value) return "—";
  const date = new Date(value);
  if (Number.isNaN(date.getTime())) return "—";

  const diffMs = date.getTime() - Date.now();
  const diffSeconds = Math.round(diffMs / 1000);
  const divisions: [Intl.RelativeTimeFormatUnit, number][] = [
    ["year", 31536000],
    ["month", 2592000],
    ["week", 604800],
    ["day", 86400],
    ["hour", 3600],
    ["minute", 60],
    ["second", 1],
  ];

  const rtf = new Intl.RelativeTimeFormat("en", { numeric: "auto" });

  for (const [unit, secondsInUnit] of divisions) {
    if (Math.abs(diffSeconds) >= secondsInUnit || unit === "second") {
      return rtf.format(Math.round(diffSeconds / secondsInUnit), unit);
    }
  }
  return rtf.format(0, "second");
}

export function truncateAddress(address: string, chars = 4): string {
  if (address.length <= chars * 2 + 2) return address;
  return `${address.slice(0, chars + 2)}...${address.slice(-chars)}`;
}

export function isPositive(value: string | number | undefined | null): boolean {
  if (value === undefined || value === null || value === "") return false;
  return toDecimal(value).gte(0);
}

export { Decimal };
