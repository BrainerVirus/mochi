import { z } from "zod";

export const PlatformIdSchema = z.enum(["macos", "windows", "linux", "unknown"]);

export type PlatformId = z.infer<typeof PlatformIdSchema>;
