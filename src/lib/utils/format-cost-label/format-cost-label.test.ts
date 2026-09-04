import { describe, expect, it } from "vitest";

import { formatCostDetail, formatCostPeriodLabel } from "./format-cost-label";

describe("formatCostPeriodLabel", () => {
  it("title-cases kebab-case periods so raw ids never render as labels", () => {
    expect(formatCostPeriodLabel("billing-period")).toBe("Billing period");
    expect(formatCostPeriodLabel("monthly")).toBe("Monthly");
  });

  it("falls back to On-demand for missing periods", () => {
    expect(formatCostPeriodLabel(null)).toBe("On-demand");
    expect(formatCostPeriodLabel(undefined)).toBe("On-demand");
    expect(formatCostPeriodLabel("")).toBe("On-demand");
    expect(formatCostPeriodLabel("-")).toBe("On-demand");
    expect(formatCostPeriodLabel("--")).toBe("On-demand");
  });
});

describe("formatCostDetail", () => {
  it("renders used vs limit money line", () => {
    expect(formatCostDetail(7.54, 71.93, "USD")).toBe("$7.54 / $71.93");
  });

  it("renders used-only line when there is no limit", () => {
    expect(formatCostDetail(12.5, 0, "USD")).toBe("$12.50");
  });
});
