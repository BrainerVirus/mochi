const currencyFormatters = new Map<string, Intl.NumberFormat>();

function formatCurrency(amount: number, currencyCode: string): string {
  try {
    let formatter = currencyFormatters.get(currencyCode);
    if (!formatter) {
      formatter = new Intl.NumberFormat(undefined, {
        style: "currency",
        currency: currencyCode,
        maximumFractionDigits: 2,
      });
      currencyFormatters.set(currencyCode, formatter);
    }
    return formatter.format(amount);
  } catch {
    return `$${amount.toFixed(2)}`;
  }
}

/** Raw period ids ("billing-period") must never render as labels. */
export function formatCostPeriodLabel(period: string | null | undefined): string {
  if (!period) {
    return "On-demand";
  }

  const [first, ...rest] = period.split("-").filter(Boolean);
  if (!first) {
    return "On-demand";
  }
  if (rest.length === 0) {
    return `${first[0].toUpperCase()}${first.slice(1)}`;
  }

  return `${first[0].toUpperCase()}${first.slice(1)} ${rest.join(" ")}`;
}

export function formatCostDetail(used: number, limit: number, currencyCode: string): string {
  return limit > 0
    ? `${formatCurrency(used, currencyCode)} / ${formatCurrency(limit, currencyCode)}`
    : formatCurrency(used, currencyCode);
}
