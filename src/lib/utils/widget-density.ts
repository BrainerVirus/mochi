export type WidgetDensity = "compact" | "normal" | "expanded";

export interface WidgetDensityClasses {
  root: string;
  card: string;
  title: string;
}

export function widgetDensityClasses(density: WidgetDensity): WidgetDensityClasses {
  switch (density) {
    case "compact":
      return {
        root: "flex flex-col gap-2 p-3",
        card: "gap-2 p-3",
        title: "text-xs font-semibold",
      };
    case "expanded":
      return {
        root: "flex flex-col gap-6 p-6",
        card: "gap-4 p-6",
        title: "text-base font-semibold",
      };
    case "normal":
    default:
      return {
        root: "flex flex-col gap-4 p-4",
        card: "gap-3 p-4",
        title: "text-sm font-semibold",
      };
  }
}
