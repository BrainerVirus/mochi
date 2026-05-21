import { Progress as ProgressPrimitive } from "radix-ui";
import * as React from "react";

import { cn } from "@/lib/utils";

function Progress({
  className,
  value,
  indicatorRef,
  animateIndicator = false,
  ...props
}: React.ComponentProps<typeof ProgressPrimitive.Root> & {
  indicatorRef?: React.Ref<HTMLDivElement>;
  animateIndicator?: boolean;
}) {
  return (
    <ProgressPrimitive.Root
      data-slot="progress"
      className={cn(
        "relative flex h-1 w-full items-center overflow-x-hidden rounded-full bg-muted",
        className,
      )}
      {...props}
    >
      <ProgressPrimitive.Indicator
        ref={indicatorRef}
        data-slot="progress-indicator"
        className={cn(
          "bg-primary size-full flex-1",
          !animateIndicator && "transition-all",
        )}
        style={
          animateIndicator
            ? undefined
            : { transform: `translateX(-${100 - (value || 0)}%)` }
        }
      />
    </ProgressPrimitive.Root>
  );
}

export { Progress };
