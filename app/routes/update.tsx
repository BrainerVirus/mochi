import { createFileRoute } from "@tanstack/react-router";
import { z } from "zod";

import { UpdatePage } from "@/components/updates/update-page";

const updateSearchSchema = z.object({
  view: z.enum(["notes"]).optional(),
});

export const Route = createFileRoute("/update")({
  validateSearch: updateSearchSchema,
  ssr: false,
  component: UpdatePage,
});
