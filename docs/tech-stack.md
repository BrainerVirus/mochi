# Mochi Tech Stack

This is the source of truth for the initial implementation stack. It reflects the installed repo skills and current npm metadata checked on 2026-05-20.

## Package Versions

Re-check exact patch versions with `npm view <package> version` immediately before installing. The verified current versions were:

- `@tanstack/react-start`: `1.168.9`
- `@tanstack/react-router`: `1.170.6`
- `@tanstack/router-plugin`: `1.168.9`
- `@tanstack/react-query`: `5.100.11`
- `vite`: `8.0.13`
- `react`: `19.2.6`
- `tailwindcss`: `4.3.0`
- `@tailwindcss/vite`: `4.3.0`
- `oxlint`: `1.66.0`
- `oxlint-tailwindcss`: `0.8.0`
- `oxfmt`: `0.51.0`
- `gsap`: `3.15.0`
- `@gsap/react`: `2.1.2`
- `zod`: `4.4.3`

Use compatible major ranges anchored on these:

- `@tanstack/react-start@^1`
- `@tanstack/react-router@^1`
- `@tanstack/router-plugin@^1`
- `@tanstack/react-query@^5`
- `vite@^8`
- `react@^19`
- `react-dom@^19`
- `tailwindcss@^4`
- `@tailwindcss/vite@^4`
- `oxlint@^1`
- `oxlint-tailwindcss@^0.8`
- `oxfmt@^0.51`
- `gsap@^3`
- `@gsap/react@^2`
- `zod@^4`
- `zustand@^5`

## Frontend

Use TanStack Start, not a plain Vite SPA. The frontend should follow the Start layout:

```text
app/
  routes/
  client.tsx
  router.tsx
  ssr.tsx
  routeTree.gen.ts
app.config.ts
```

TanStack Start is powered by Vite and Nitro/Vinxi. Configure Vite through `app.config.ts`; do not add a separate Vite-only app entry unless the scaffold requires it. Tailwind CSS v4 must be wired with the official `@tailwindcss/vite` plugin.

Use React 19, TypeScript strict mode, TanStack Router file routes, TanStack Query for async server/cache state, Zustand for local client UI state, and Zod for server function/API route validation.

## Frontend Folder Structure

Keep the TanStack Start app under `app/` and shared UI/domain code under `src/`:

```text
app/
  routes/                # file routes only; keep route files thin
  client.tsx
  router.tsx
  ssr.tsx
  routeTree.gen.ts       # generated, do not hand edit
src/
  components/
    ui/                  # shadcn-owned components
    usage/
    settings/
    updates/
    mascot/
  features/              # feature folders for stateful UI flows
  hooks/
  lib/
    query/               # query client, query keys, query options
    schemas/             # Zod schemas and inferred types
    stores/              # Zustand stores for local UI state
    tauri/               # typed invoke wrappers
    utils/
  styles/
```

Route files should wire loaders, validation, and layout. Move reusable UI, hooks, schemas, and Tauri calls out of route files once they are used by more than one route or make the route hard to scan.

## Frontend Maintainability Rules

Use these as lint/review standards after scaffold:

- File names: kebab-case for component/module files (`usage-card.tsx`, `provider-status.ts`); route files follow TanStack Router conventions.
- Component names: PascalCase exports from kebab-case files.
- Max file size target: keep TS/TSX files under 250 lines; split before 350 lines unless the file is generated or a small shadcn component.
- Function/component size target: keep functions under 60 lines; extract helpers or child components when branching dominates the body.
- Imports: no deep relative chains beyond `../..`; configure `@/` or `~/` aliases and keep import ordering formatter/linter controlled.
- Types: avoid `any`; use `unknown` at boundaries and validate with Zod before narrowing.
- Zod schemas: export both `FooSchema` and `type Foo = z.infer<typeof FooSchema>` from `src/lib/schemas/`; use `safeParse` for user input, Tauri responses, local files, and JSON.
- React state: do not mirror derived state in `useEffect`; compute derived values during render or memoize only when there is a measurable reason.
- TanStack Query: use for provider usage snapshots, status checks, update checks, and any async data with loading/error/stale states. Keep query keys centralized under `src/lib/query/`.
- Zustand: use only for local client state that is not server/cache state, such as selected provider, panel density, widget layout preferences before persistence, transient filters, and command palette state. Keep stores small and domain-specific.
- Tauri calls: never call `invoke` directly from components; use typed wrappers under `src/lib/tauri/` and validate responses with Zod where the shape can drift.
- Animation: use GSAP for non-trivial animation; in React use `@gsap/react` `useGSAP()`, scoped refs, cleanup via GSAP context, and `gsap.matchMedia()` for responsive/reduced-motion behavior.
- Generated files: do not hand edit `routeTree.gen.ts` or generated shadcn files except for documented project customizations.

## Desktop Shell

Use Tauri v2 for the cross-platform app:

- Rust backend in `src-tauri/`.
- Tauri config at `src-tauri/tauri.conf.json`.
- Frontend dist/dev URL wired from the TanStack Start build/dev command.
- `src-tauri/src/main.rs` stays thin; app setup and command registration live in `src-tauri/src/lib.rs`.
- Add `src-tauri/capabilities/default.json` early. Tauri v2 denies plugin/API access unless permissions are explicit.

Rust should own provider fetching, secure storage, CLI, tray, widget state, updater, status-bar output, notifications, and filesystem/browser access. React should render UI and call typed Tauri commands.

## Tauri/Rust Folder Structure

Keep Rust modules explicit and domain-oriented:

```text
src-tauri/
  Cargo.toml
  tauri.conf.json
  capabilities/
    default.json
  src/
    main.rs              # thin desktop entry
    lib.rs               # Tauri builder, state, command registration
    cli/
    core/                # provider-agnostic models, traits, errors
    providers/           # one module per provider
    auth/
    browser/
    fetch/
    settings/
    status/
    status_bar/
    tray/
    widget/
    notifications/
    updater/
```

`core/` must not depend on Tauri UI modules. Provider modules should depend on `core`, `fetch`, `auth`, and `settings`, not on tray/widget/frontend concerns.

## Tauri/Rust Maintainability Rules

Use these as clippy/review standards after scaffold:

- Keep `main.rs` as a passthrough; all setup lives in `lib.rs`.
- Register every command in `tauri::generate_handler!` and add required permissions in `capabilities/default.json`.
- Command names should be verb-noun and stable (`get_usage_snapshots`, `refresh_provider`, `check_for_update`).
- Async command parameters must own their data (`String`, structs), not borrow (`&str`).
- Return `Result<T, E>` from fallible commands and serialize typed error responses; do not panic for user/runtime failures.
- Use `thiserror` for library/domain errors and `anyhow` only at binary/bootstrap boundaries.
- Avoid `unwrap()` and `expect()` outside tests.
- File size target: keep Rust modules under 300 lines; split before 450 lines unless mostly data definitions or generated code.
- Function size target: keep functions under 80 lines; split parsing, I/O, mapping, and command orchestration.
- Prefer `&str`/`&[T]` parameters for internal sync functions; clone only when ownership is required.
- Provider implementations must declare metadata, fetch strategies, fallback rules, and redaction rules explicitly.
- Unit tests live beside Rust modules for pure logic; integration tests belong under `src-tauri/tests/` once command or provider boundaries are exercised.

## Styling And Design System

Use Tailwind CSS 4 and shadcn/ui.

Before building UI, create or update `DESIGN.md` from `.agents/skills/design-md/SKILL.md`. Treat it as the semantic design-system source for shadcn tokens, component choices, color roles, shape, typography, and layout language.

Tailwind class handling is split deliberately:

- Build integration: use official `@tailwindcss/vite`.
- Linting: use native `oxlint-tailwindcss` through oxlint `jsPlugins`.
- Formatting/class order: use oxfmt `sortTailwindcss`.

For shadcn work:

- Run the shadcn CLI with the project package manager.
- Use existing shadcn components before custom markup.
- Use semantic tokens (`bg-background`, `text-muted-foreground`, `bg-primary`) rather than raw Tailwind colors in components.
- Use `gap-*`, not `space-x-*` or `space-y-*`.
- Use `size-*` for square dimensions.
- Use `cn()` for conditional classes.
- Use lucide icons only if shadcn project context says the icon library is lucide.

## Runtime Libraries From Installed Skills

These are project-approved implementation libraries because the repo has installed skills for them:

- GSAP + `@gsap/react` for meaningful UI animation, timelines, responsive animation, and reduced-motion-aware motion.
- Zod 4 for validation at every untrusted boundary: server functions, Tauri IPC results, local config, provider payloads, JSON files, and form/query data.
- TanStack Query 5 is the default server/cache state layer for Tauri command results and provider/status/update refresh flows that need caching, refetching, stale state, retries, or background refresh.
- Zustand 5 is the default local client-state layer for UI-only state. Do not store server snapshots or provider fetch results in Zustand; keep those in TanStack Query.
- TanStack Devtools may be added for local development diagnostics, but do not ship it in production UI.
- GSAP plugins should only be added when the feature needs the plugin behavior: ScrollTrigger for scroll-driven animation, Flip for layout transitions, Draggable/Inertia for drag interactions.

## Linting And Formatting

Use Oxc tooling for the frontend quality gate:

- JavaScript/TypeScript linting: `oxlint`
- React linting: oxlint React plugin
- Accessibility linting: oxlint JSX a11y plugin
- Import/export linting: oxlint import plugin
- Tailwind lint plugin: `oxlint-tailwindcss`
- Formatting for JS, TS, JSX, TSX, JSON, HTML, CSS, Markdown, TOML, YAML, GraphQL, and package metadata: `oxfmt`

Do not introduce ESLint, Prettier, or Biome unless the user explicitly changes direction.

Expected scripts after scaffold:

```json
{
  "scripts": {
    "dev": "vinxi dev",
    "build": "vinxi build",
    "start": "vinxi start",
    "lint": "oxlint --type-aware --react-plugin --jsx-a11y-plugin --import-plugin --deny-warnings",
    "format": "oxfmt",
    "format:check": "oxfmt --check",
    "tauri": "tauri"
  }
}
```

Use `.oxlintrc.json` for lint categories/plugins and `.oxfmtrc.json` for formatter options. Prefer `correctness: error`, `suspicious: warn`, React and JSX accessibility plugins for UI code, the import plugin for module hygiene, `oxlint-tailwindcss` for Tailwind correctness/conflict checks, and keep style-only lint rules light because formatting belongs to oxfmt.

The Oxc quality gate must cover:

- TypeScript: strict type-aware linting, no `any`, no unused exports/variables, safe narrowing from `unknown`.
- Imports: unresolved imports, duplicate imports, export/import consistency, and no deep relative chains beyond `../..`.
- React: hooks rules, JSX correctness, component naming consistency, stale effect/dependency risks, and no derived-state effects.
- Accessibility: semantic JSX, ARIA validity, labels for controls, and accessible interactive elements.
- JavaScript correctness: suspicious control flow, promises, unsafe equality/coercion, dead code, and performance warnings.
- Tailwind: unknown classes, conflicting classes, and official class ordering.
- HTML/CSS/assets: formatting via oxfmt; Tailwind CSS entrypoint linting via `oxlint-tailwindcss`. Any semantic CSS/HTML checks unsupported by the installed Oxc tools are review requirements until tool support exists.

Suggested `.oxlintrc.json` policy after scaffold:

```jsonc
{
  "categories": {
    "correctness": "error",
    "suspicious": "warn",
    "perf": "warn",
    "pedantic": "off",
    "style": "off",
    "restriction": "off",
  },
  "plugins": ["typescript", "unicorn", "oxc", "react", "jsx-a11y", "import"],
  "jsPlugins": ["oxlint-tailwindcss"],
  "env": {
    "browser": true,
    "node": true,
  },
  "options": {
    "typeAware": true,
  },
  "settings": {
    "tailwindcss": {
      "entryPoint": "src/styles/index.css",
    },
  },
  "overrides": [
    {
      "files": ["app/routeTree.gen.ts", "src/components/ui/**"],
      "rules": {
        "max-lines": "off",
      },
    },
    {
      "files": ["**/*.test.{ts,tsx}"],
      "plugins": ["vitest"],
      "rules": {
        "no-console": "off",
      },
    },
  ],
  "rules": {
    "max-lines": ["warn", 300],
    "max-lines-per-function": ["warn", 80],
    "no-explicit-any": "error",
    "no-duplicate-imports": "error",
    "no-console": "warn",
    "tailwindcss/no-unknown-classes": "error",
    "tailwindcss/no-conflicting-classes": "error",
    "tailwindcss/enforce-sort-order": "warn",
  },
}
```

If a listed rule name is not supported by the installed oxlint/plugin version, keep the policy in review docs and enforce with code review until the tool supports it.

Suggested `.oxfmtrc.json` policy after scaffold:

```json
{
  "printWidth": 100,
  "semi": true,
  "singleQuote": false,
  "jsxSingleQuote": false,
  "trailingComma": "all",
  "bracketSpacing": true,
  "bracketSameLine": false,
  "singleAttributePerLine": false,
  "htmlWhitespaceSensitivity": "css",
  "sortImports": {
    "partitionByNewline": true,
    "newlinesBetween": false
  },
  "sortPackageJson": {
    "sortScripts": true
  },
  "sortTailwindcss": true
}
```

## Verification

After scaffold, a normal local verification pass should be:

```bash
pnpm lint
pnpm format:check
pnpm build
cargo test --manifest-path src-tauri/Cargo.toml
```

For focused Rust changes, run the narrowest meaningful test first, then the broader Rust command:

```bash
cargo test --manifest-path src-tauri/Cargo.toml <test_or_module_name>
cargo check --manifest-path src-tauri/Cargo.toml
```

For focused TS/UI changes, run:

```bash
pnpm lint -- <changed-files>
pnpm build
```

Use Playwright/browser screenshots for meaningful visual changes after the dev server is running.
