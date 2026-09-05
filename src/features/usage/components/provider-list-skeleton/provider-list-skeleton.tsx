import { Skeleton } from "@/components/ui/skeleton";

export function ProviderListSkeleton({ providerCount }: { providerCount: number }) {
  if (providerCount <= 0) {
    return null;
  }
  return (
    <div aria-hidden="true">
      {Array.from({ length: providerCount }, (_, index) => (
        <div
          key={index}
          data-testid="provider-skeleton-row"
          className="flex items-center gap-3 py-2"
        >
          <Skeleton className="h-8 w-8 rounded-full" />
          <div className="flex-1 space-y-1.5">
            <Skeleton className="h-3 w-2/3" />
            <Skeleton className="h-2 w-full" />
          </div>
        </div>
      ))}
    </div>
  );
}
