import { z } from "zod";

import { ProviderIdSchema } from "./usage";

export const CatalogSettingsFieldSchema = z.object({
  key: z.string(),
  label: z.string(),
  kind: z.enum([
    "api-key",
    "cookie-source",
    "manual-cookie",
    "token-account",
    "history-window",
    "region-host",
  ]),
});

export const ProviderCatalogEntrySchema = z.object({
  id: ProviderIdSchema,
  displayName: z.string(),
  implementationStatus: z.enum(["stub", "partial", "done"]),
  strategies: z.array(
    z.object({
      id: z.string(),
      kind: z.string(),
      label: z.string(),
    }),
  ),
  authRequirements: z.array(z.string()),
  settingsFields: z.array(CatalogSettingsFieldSchema),
});

export type ProviderCatalogEntry = z.infer<typeof ProviderCatalogEntrySchema>;

export const ProviderCatalogSchema = z.array(ProviderCatalogEntrySchema);

export const ProviderCredentialStatusSchema = z.record(z.string(), z.boolean());
