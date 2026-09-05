import { Skeleton } from "@/components/ui/skeleton";

export function ProviderListSkeleton({ providerCount }: { providerCount: number }) {
  const rowCount = Math.max(providerCount, 1);
  return (
    <>
      <output className="sr-only">Loading provider usage…</output>
      <div aria-hidden="true">
        {Array.from({ length: rowCount }, (_, index) => (
          <div
            key={index}
            data-testid="provider-skeleton-row"
            className="flex items-center gap-3 py-2"
          >
            <Skeleton className="h-8 w-8 rounded-full motion-reduce:animate-none" />
            <div className="flex-1 space-y-1.5">
              <Skeleton className="h-3 w-2/3 motion-reduce:animate-none" />
              <Skeleton className="h-2 w-full motion-reduce:animate-none" />
            </div>
          </div>
        ))}
      </div>
    </>
  );
}
