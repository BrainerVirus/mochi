import { Fragment } from "react";

import { Skeleton } from "@/components/ui/skeleton";

/**
 * First-paint placeholder for the widget window: stacked provider sections
 * with dividers, mirroring the full-height overview geometry (non-compact
 * meters, full-width bars).
 */
export function WidgetSkeleton({ providerCount }: { providerCount: number }) {
  const sectionCount = Math.max(providerCount, 1);
  return (
    <>
      <output className="sr-only">Loading provider usage…</output>
      <div aria-hidden="true" className="flex flex-col pt-3">
        {Array.from({ length: sectionCount }, (_, index) => (
          <Fragment key={index}>
            <div data-testid="widget-skeleton-section" className="flex flex-col px-3">
              <div className="flex items-start justify-between gap-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <div className="flex items-center gap-1.5">
                    <Skeleton className="size-4 motion-reduce:animate-none" />
                    <Skeleton className="h-4 w-28 motion-reduce:animate-none" />
                  </div>
                  <Skeleton className="h-3 w-20 motion-reduce:animate-none" />
                </div>
                <Skeleton className="size-8 shrink-0 rounded-md motion-reduce:animate-none" />
              </div>
              <div className="mt-2.5 flex flex-col gap-2.5">
                <div className="flex flex-col gap-1.5">
                  <Skeleton className="h-3.5 w-24 motion-reduce:animate-none" />
                  <Skeleton className="h-1.5 w-full rounded-full motion-reduce:animate-none" />
                  <div className="flex items-baseline justify-between gap-2">
                    <Skeleton className="h-3 w-16 motion-reduce:animate-none" />
                    <Skeleton className="h-3 w-12 motion-reduce:animate-none" />
                  </div>
                </div>
                <div className="flex flex-col gap-1.5">
                  <Skeleton className="h-3.5 w-20 motion-reduce:animate-none" />
                  <Skeleton className="h-1.5 w-full rounded-full motion-reduce:animate-none" />
                  <div className="flex items-baseline justify-between gap-2">
                    <Skeleton className="h-3 w-14 motion-reduce:animate-none" />
                    <Skeleton className="h-3 w-12 motion-reduce:animate-none" />
                  </div>
                </div>
              </div>
            </div>
            {index < sectionCount - 1 ? (
              <div className="px-3 pt-2 pb-0">
                <div className="bg-border h-px w-full" />
              </div>
            ) : null}
          </Fragment>
        ))}
      </div>
    </>
  );
}
