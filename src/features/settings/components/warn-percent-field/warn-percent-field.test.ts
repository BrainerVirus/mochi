// @vitest-environment happy-dom
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { createElement } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";

import { WarnPercentField } from "./warn-percent-field";

afterEach(cleanup);

describe("WarnPercentField", () => {
  it("renders the current threshold value", () => {
    render(
      createElement(WarnPercentField, {
        id: "warn-percent",
        label: "Warning threshold",
        description: "Warn before limits.",
        value: 80,
        onChange: () => {},
      }),
    );

    expect(screen.getByLabelText("Warning threshold")).toHaveProperty("value", "80");
  });

  it("sends valid percentages to onChange", () => {
    const onChange = vi.fn<(value: number | undefined) => void>();
    render(
      createElement(WarnPercentField, {
        id: "warn-percent",
        label: "Warning threshold",
        description: "Warn before limits.",
        value: 80,
        onChange,
      }),
    );

    fireEvent.change(screen.getByLabelText("Warning threshold"), {
      target: { value: "90" },
    });

    expect(onChange).toHaveBeenCalledWith(90);
  });

  it("clears to undefined so the provider inherits the global default", () => {
    const onChange = vi.fn<(value: number | undefined) => void>();
    render(
      createElement(WarnPercentField, {
        id: "warn-percent",
        label: "Warning threshold",
        description: "Warn before limits.",
        value: 90,
        onChange,
      }),
    );

    fireEvent.change(screen.getByLabelText("Warning threshold"), {
      target: { value: "" },
    });

    expect(onChange).toHaveBeenCalledWith(undefined);
  });

  it("clamps out-of-range input to one to one hundred", () => {
    const onChange = vi.fn<(value: number | undefined) => void>();
    render(
      createElement(WarnPercentField, {
        id: "warn-percent",
        label: "Warning threshold",
        description: "Warn before limits.",
        value: 80,
        onChange,
      }),
    );

    fireEvent.change(screen.getByLabelText("Warning threshold"), {
      target: { value: "150" },
    });

    expect(onChange).toHaveBeenCalledWith(100);
  });
});
