import { Skeleton } from "@/components/ui/skeleton";

/**
 * First-paint placeholder for the tray popover: tab strip plus one provider
 * section per row, mirroring the tabbed usage-section geometry (compact
 * meters, header with provider icon + refresh action).
 */
export function TrayPanelSkeleton({ providerCount }: { providerCount: number }) {
  const rowCount = Math.max(providerCount, 1);
  return (
    <>
      <output className="sr-only">Loading provider usage…</output>
      <div aria-hidden="true">
        <div className="border-border min-w-0 overflow-hidden rounded-t-[var(--radius-tray-panel)] border-b px-3 pb-2">
          <div className="flex h-11 items-center gap-1.5">
            <Skeleton className="h-6 w-20 rounded-full motion-reduce:animate-none" />
            <Skeleton className="h-6 w-16 rounded-full motion-reduce:animate-none" />
            <Skeleton className="h-6 w-24 rounded-full motion-reduce:animate-none" />
          </div>
        </div>
        <div className="px-3 pt-3">
          {Array.from({ length: rowCount }, (_, index) => (
            <div key={index} data-testid="tray-panel-skeleton-row" className="flex flex-col">
              <div className="flex items-start justify-between gap-2">
                <div className="flex min-w-0 flex-col gap-1">
                  <div className="flex items-center gap-1.5">
                    <Skeleton className="size-4 rounded-full motion-reduce:animate-none" />
                    <Skeleton className="h-3.5 w-24 motion-reduce:animate-none" />
                  </div>
                  <Skeleton className="h-2.5 w-16 motion-reduce:animate-none" />
                </div>
                <Skeleton className="size-7 shrink-0 rounded-md motion-reduce:animate-none" />
              </div>
              <div className="mt-2 flex flex-col gap-3">
                <div className="flex flex-col gap-1">
                  <Skeleton className="h-3 w-20 motion-reduce:animate-none" />
                  <Skeleton className="h-1 w-full rounded-full motion-reduce:animate-none" />
                  <div className="flex items-baseline justify-between gap-2">
                    <Skeleton className="h-2.5 w-14 motion-reduce:animate-none" />
                    <Skeleton className="h-2.5 w-10 motion-reduce:animate-none" />
                  </div>
                </div>
                <div className="flex flex-col gap-1">
                  <Skeleton className="h-3 w-24 motion-reduce:animate-none" />
                  <Skeleton className="h-1 w-full rounded-full motion-reduce:animate-none" />
                  <div className="flex items-baseline justify-between gap-2">
                    <Skeleton className="h-2.5 w-12 motion-reduce:animate-none" />
                    <Skeleton className="h-2.5 w-10 motion-reduce:animate-none" />
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>
    </>
  );
}
