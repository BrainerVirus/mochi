# Design System

Before creating or modifying UI, frontend, CSS, Tailwind, components, or accessibility behavior, read the root `DESIGN.md` file.

Treat `DESIGN.md` YAML front matter tokens as authoritative for colors, typography, spacing, radius, elevation, motion, and component styling. Use the Markdown rationale for visual hierarchy, layout density, interaction tone, and do/don't decisions.

When changing design tokens, run `npx @google/design.md lint DESIGN.md` if the package is available. If the package is unavailable, report that the design lint could not be run and continue with `pnpm format:check`.
