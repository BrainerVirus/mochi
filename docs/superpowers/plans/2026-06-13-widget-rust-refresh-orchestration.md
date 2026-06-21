# Rust Push-Model Refresh Orchestration — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the race-condition-prone `tray-refresh` event-based refresh with a single Rust command that runs the refresh to completion, then pushes results to all windows via `"usage-refresh-complete"`.

**Architecture:** A single Rust `refresh_all_providers` command runs `refresh_enabled_snapshots()`, then emits `"usage-refresh-complete"` with serialized `ProviderUsageState[]`. Both frontend windows listen for this event and update their TanStack Query cache directly via `setQueryData`. Individual per-provider refresh goes through `refresh_single_provider` with the same push event.

**Tech Stack:** Rust (Tauri v2 commands), TypeScript (TanStack Query, `@tauri-apps/api/event`)

---

## 2026-06-20 Follow-up: Serialize overlapping provider refreshes

The original push-model tasks below were completed by PRs #86–#88 and are retained only as
historical context. They must not be replayed. One race remains: the nonblocking provider guard
lets an overlapping Rust entry point skip active work and immediately emit a stale completion
payload.

### Task 1: Wait for active work on the same provider

**Files:**

- Modify: `src-tauri/src/status/refresh_controller.rs`
- Modify: `src-tauri/src/status/mod.rs`

- [x] **Step 1: Write deterministic concurrency tests**

Replace the try-lock test with tests that poll acquisition directly: a second acquisition for the
same provider must remain pending, while another provider must acquire immediately.

```rust
let first = controller
    .begin_provider_refresh(ProviderId::Claude)
    .await;
let mut second = Box::pin(controller.begin_provider_refresh(ProviderId::Claude));

tokio::select! {
    biased;
    _ = &mut second => panic!("same-provider refresh must wait"),
    () = std::future::ready(()) => {}
}

drop(first);
let _second = second.await;
```

- [x] **Step 2: Run the focused test and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml refresh_controller
```

Expected: compilation fails because `begin_provider_refresh` does not exist.

- [x] **Step 3: Replace the nonblocking set with per-provider async mutexes**

```rust
#[derive(Default)]
pub struct RefreshController {
    providers: Mutex<HashMap<ProviderId, Arc<Mutex<()>>>>,
}

impl RefreshController {
    pub async fn begin_provider_refresh(&self, provider: ProviderId) -> OwnedMutexGuard<()> {
        let provider_lock = {
            let mut providers = self.providers.lock().await;
            Arc::clone(
                providers
                    .entry(provider)
                    .or_insert_with(|| Arc::new(Mutex::new(()))),
            )
        };
        provider_lock.lock_owned().await
    }
}
```

Await this guard in both `refresh_enabled_snapshots` and `refresh_single_provider_inner`. This
serializes only overlapping work for the same provider; different providers remain independent.

- [x] **Step 4: Run focused and status tests and verify GREEN**

```bash
cargo test --manifest-path src-tauri/Cargo.toml refresh_controller
cargo test --manifest-path src-tauri/Cargo.toml status::tests
```

Expected: all selected tests pass without network-dependent assertions.

- [x] **Step 5: Run repository validation, commit, push, and open a PR**

Use the required commands from `AGENTS.md`, commit with a conventional message, push
`fix/serialize-refresh-completions`, and open a PR without merging it.

---

### Task 1: Add `RefreshCompletePayload` and `refresh_all_providers` command (Rust)

**Files:**

- Modify: `src-tauri/src/status/mod.rs`

- [ ] **Step 1: Add the payload struct and two new public functions**

Add after the existing imports at the top (around line 13):

```rust
use serde::Serialize;
use crate::core::usage_state::ProviderUsageState;
```

Add before the `#[cfg(test)]` block (around line 357):

```rust
#[derive(Serialize)]
pub struct RefreshCompletePayload {
    pub states: Vec<ProviderUsageState>,
}

/// Refresh all enabled providers to completion, then return updated states.
/// Called from the tray handler; does NOT emit events — caller decides.
pub async fn refresh_all_providers_inner(
    app: &AppHandle,
    store: &UsageStore,
    settings: &MochiSettings,
) -> Result<RefreshCompletePayload, ProviderError> {
    refresh_enabled_snapshots(store, settings).await?;
    let states = read_cached_usage_states(store, settings);
    Ok(RefreshCompletePayload { states })
}
```

The `ProviderError` import may need to be added at the top: it's already imported via `use crate::core::provider::...`.

- [ ] **Step 2: Add the Tauri command wrapper**

```rust
#[tauri::command]
pub async fn refresh_all_providers(
    app: AppHandle,
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
) -> Result<RefreshCompletePayload, String> {
    let settings = settings_state.current()?;
    let payload = refresh_all_providers_inner(&app, &store, &settings)
        .await
        .map_err(|e| e.to_string())?;
    let _ = app.emit("usage-refresh-complete", &payload);
    Ok(payload)
}
```

- [ ] **Step 3: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/status/mod.rs
git commit -m "feat(status): add refresh_all_providers command with push payload"
```

---

### Task 2: Add `refresh_single_provider` command (Rust)

**Files:**

- Modify: `src-tauri/src/status/mod.rs`

- [ ] **Step 1: Add the `refresh_single_provider` command**

Add after `refresh_all_providers`:

```rust
#[tauri::command]
pub async fn refresh_single_provider(
    app: AppHandle,
    store: State<'_, UsageStore>,
    settings_state: State<'_, SettingsState>,
    provider: String,
) -> Result<RefreshCompletePayload, String> {
    let settings = settings_state.current()?;
    let provider_id = ProviderId::parse(&provider)
        .ok_or_else(|| format!("unknown provider: {provider}"))?;
    let ctx = FetchContext::from_settings(&settings);

    if !provider_has_credentials(provider_id, &ctx) {
        store.record_error(
            provider_id,
            "credentials missing",
            failed_attempt("live-fetch", &ProviderError::NotConfigured),
        );
        return Err("credentials missing".to_string());
    }

    let Some(_guard) = refresh_controller().try_begin_provider_refresh(provider_id) else {
        return Err("refresh already in progress".to_string());
    };

    match fetch_provider_snapshot(provider_id, &ctx).await {
        Ok(Some(snapshot)) => {
            store.record_success(snapshot);
        }
        Ok(None) => {
            store.record_error(
                provider_id,
                "provider returned no snapshot",
                failed_attempt("live-fetch", &ProviderError::NotConfigured),
            );
        }
        Err(error) => {
            if store.record_failure(provider_id, &error, failed_attempt("live-fetch", &error)).is_none() {
                store.record_error(
                    provider_id,
                    error.to_string(),
                    failed_attempt("live-fetch", &error),
                );
            }
        }
    }

    let states = read_cached_usage_states(&store, &settings);
    let payload = RefreshCompletePayload { states };
    let _ = app.emit("usage-refresh-complete", &payload);
    Ok(payload)
}
```

- [ ] **Step 2: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/status/mod.rs
git commit -m "feat(status): add refresh_single_provider command with push payload"
```

---

### Task 3: Change tray "refresh" handler to call new command (Rust)

**Files:**

- Modify: `src-tauri/src/tray/mod.rs`

- [ ] **Step 1: Replace the `"refresh"` menu event handler**

The current handler (line 235-236):

```rust
"refresh" => {
    let _ = app.emit("tray-refresh", ());
}
```

Replace with:

```rust
"refresh" => {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Some(store) = app.try_state::<crate::core::usage_store::UsageStore>() {
            if let Some(settings_state) = app.try_state::<crate::settings::SettingsState>() {
                if let Ok(settings) = settings_state.current() {
                    let _ = crate::status::refresh_all_providers_inner(
                        &app, &store, &settings,
                    ).await.map(|payload| {
                        let _ = app.emit("usage-refresh-complete", &payload);
                    });
                }
            }
        }
    });
}
```

This calls the inner function directly (no need for a full Tauri command invocation from the tray handler) and emits the push event.

- [ ] **Step 2: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/tray/mod.rs
git commit -m "feat(tray): use refresh_all_providers instead of tray-refresh event"
```

---

### Task 4: Register new commands in `lib.rs` (Rust)

**Files:**

- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Add new commands to `generate_handler!`**

Add after line 119 (`status::refresh_enabled_providers`):

```rust
            status::refresh_all_providers,
            status::refresh_single_provider,
```

- [ ] **Step 2: Run Rust compilation check**

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: compiles without errors.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat(lib): register refresh_all_providers and refresh_single_provider commands"
```

---

### Task 5: Add frontend command bindings (TypeScript)

**Files:**

- Modify: `src/lib/tauri/commands/commands.ts`

- [ ] **Step 1: Add `refreshAllProviders` binding**

Add after `refreshEnabledProviders` (line 71):

```typescript
import type { ProviderUsageStates } from "@/lib/schemas/usage";

export function refreshAllProviders(): Promise<ProviderUsageStates> {
  return invoke<unknown>("refresh_all_providers").then((result) =>
    ProviderUsageStatesSchema.parse(result),
  );
}

export function refreshSingleProvider(provider: ProviderId): Promise<ProviderUsageStates> {
  return invoke<unknown>("refresh_single_provider", { provider }).then((result) =>
    ProviderUsageStatesSchema.parse(result),
  );
}
```

Note: `ProviderUsageStatesSchema` is exported from `@/lib/schemas/usage` — verify it's exported.

- [ ] **Step 2: Update imports**

Add to the existing imports:

```typescript
import {
  // ...existing imports...
  ProviderUsageStatesSchema,
  type ProviderUsageStates,
} from "@/lib/schemas/usage";
```

- [ ] **Step 3: Commit**

```bash
git add src/lib/tauri/commands/commands.ts
git commit -m "feat(commands): add refreshAllProviders and refreshSingleProvider bindings"
```

---

### Task 6: Replace `"tray-refresh"` listener with `"usage-refresh-complete"` (TypeScript)

**Files:**

- Modify: `src/features/tray/hooks/use-tray-events/use-tray-events.ts`

- [ ] **Step 1: Replace the `"tray-refresh"` listener**

Current code (around line 65):

```typescript
listen("tray-refresh", () => {
  const settings =
    queryClient.getQueryData<MochiSettings>(queryKeys.settings) ?? DEFAULT_MOCHI_SETTINGS;
  void runTrayRefreshEventSequence(
    queryClient,
    settings,
    refreshEnabledProviders,
    syncCurrentTrayUsage,
  );
}),
```

Replace with:

```typescript
listen<{ states: ProviderUsageState[] }>("usage-refresh-complete", (event) => {
  queryClient.setQueryData(queryKeys.usageSnapshots, event.payload.states);
}),
```

- [ ] **Step 2: Remove unused functions**

Remove:

- `runTrayRefreshEventSequence` (lines 114-123)
- `shouldRunProviderRefreshForTrayEvent` (lines 110-112)
- `TrayRefreshEventQueryClient` interface (lines 144-146)

- [ ] **Step 3: Clean up imports**

Remove these no-longer-needed imports:

- `refreshEnabledProviders` from `@/lib/tauri/commands`
- `DEFAULT_MOCHI_SETTINGS` from `@/lib/schemas/settings`
- `shouldRunProviderRefreshForTrayEvent` (function to remove)
- `runTrayRefreshEventSequence` (function to remove)

**KEEP** these — they're still used by `reconcileSettingsSaveSuccess`:

- `syncCurrentTrayUsage` from tray-ui-store
- `queryKeys` from `@/lib/query/keys`

Add import:

- `type ProviderUsageState` from `@/lib/schemas/usage` (for event payload type)

- [ ] **Step 4: Commit**

```bash
git add src/features/tray/hooks/use-tray-events/use-tray-events.ts
git commit -m "feat(events): replace tray-refresh listener with usage-refresh-complete push listener"
```

---

### Task 7: Simplify `useTrayPanelRefresh` (TypeScript)

**Files:**

- Modify: `src/features/tray/hooks/use-tray-panel-refresh/use-tray-panel-refresh.ts`

- [ ] **Step 1: Simplify to just invoke the Rust command**

Current full file content. Replace with:

```typescript
import { useCallback, useState } from "react";

import { refreshAllProviders } from "@/lib/tauri/commands";

export function useTrayPanelRefresh() {
  const [isRefreshingAll, setIsRefreshingAll] = useState(false);

  const refreshAll = useCallback(async () => {
    if (isRefreshingAll) return;
    setIsRefreshingAll(true);
    try {
      await refreshAllProviders();
    } finally {
      setIsRefreshingAll(false);
    }
  }, [isRefreshingAll]);

  return { refreshAll, isRefreshingAll };
}
```

Remove:

- `UseTrayPanelRefreshOptions` interface
- `useQueryClient` from `@tanstack/react-query`
- `queryKeys` from `@/lib/query/keys`
- `type TraySelectedTab` from tray-ui-store
- `type ProviderId` from `@/lib/schemas/usage`
- `refreshEnabledProviders` and `syncTrayUsage` from `@/lib/tauri/commands`
- `enabledProviders`, `refetch`, `selectedTab` parameters
- All the manual `invalidateQueries` + `syncTrayUsage` + `refetch` logic

- [ ] **Step 2: Commit**

```bash
git add src/features/tray/hooks/use-tray-panel-refresh/use-tray-panel-refresh.ts
git commit -m "refactor(refresh): simplify useTrayPanelRefresh to delegate to Rust command"
```

---

### Task 8: Update `useTrayPanelState` refresh paths (TypeScript)

**Files:**

- Modify: `src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts`

- [ ] **Step 1: Update the `useTrayPanelRefresh` call**

Current:

```typescript
const { refreshAll, isRefreshingAll } = useTrayPanelRefresh({
  enabledProviders,
  refetch: () => refetch(),
  selectedTab,
});
```

Replace with:

```typescript
const { refreshAll, isRefreshingAll } = useTrayPanelRefresh();
```

- [ ] **Step 2: Update `handleRefreshProvider` to use the Rust command**

Current imports: `refreshProviderMutation` (from useRefreshProvider).

Replace `handleRefreshProvider`:

```typescript
function handleRefreshProvider(provider: ProviderId) {
  setRefreshingProvider(provider);
  refreshSingleProvider(provider)
    .catch(() => {
      // Errors are already recorded on Rust side
    })
    .finally(() => {
      setRefreshingProvider(null);
    });
}
```

Add import:

```typescript
import { refreshSingleProvider } from "@/lib/tauri/commands";
```

- [ ] **Step 3: Remove `useRefreshProvider` usage**

Remove:

- `const refreshProviderMutation = useRefreshProvider();`
- The `useRefreshProvider` import
- In the returned object, remove `refreshProviderMutation` and just keep `refreshingProvider`

- [ ] **Step 4: Commit**

```bash
git add src/features/tray/hooks/use-tray-panel-state/use-tray-panel-state.ts
git commit -m "refactor(state): use Rust refresh commands instead of manual mutation"
```

---

### Task 9: Frontend validation

**Files:**

- Project root

- [ ] **Step 1: Run TypeScript check and lint**

```bash
pnpm build
pnpm lint
```

Expected: builds and lints without errors.

- [ ] **Step 2: Run frontend tests**

```bash
pnpm test
```

Expected: all existing tests pass. May need to update mocks in:

- `src/features/tray/hooks/use-tray-events/use-tray-events.test.ts` — remove references to `runTrayRefreshEventSequence`, `refreshEnabledProviders` mock
- `src/features/tray/hooks/use-tray-panel-refresh/` — if tests exist, update mocks

- [ ] **Step 3: Commit any test fixes**

```bash
git add src/features/tray/hooks/use-tray-events/use-tray-events.test.ts
git commit -m "test(events): update tests for usage-refresh-complete push model"
```

---

### Task 10: Rust tests

**Files:**

- Modify: `src-tauri/src/status/mod.rs` (tests at bottom)

- [ ] **Step 1: Verify existing Rust tests pass**

```bash
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

Expected: all tests pass.

- [ ] **Step 2: Commit**

```bash
git commit -m "test: verify Rust refresh commands work"
```
