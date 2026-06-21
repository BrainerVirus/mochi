import { cn } from "@/lib/utils";

export function getTrayTabChevronButtonClassName(visible: boolean) {
  return cn(
    "pointer-events-auto shrink-0 cursor-pointer rounded-full",
    "bg-transparent text-foreground shadow-none ring-0",
    "hover:bg-transparent hover:text-foreground",
    "disabled:opacity-100",
    visible ? "opacity-100" : "pointer-events-none",
  );
}
