# Command Code provider — live verification (2026-09-03)

Task 7 of the release-provider-hardening workstream. Goal: verify the new
Command Code provider against the real `api.commandcode.ai` using the session
cookie from a local browser profile on this machine (Ubuntu Linux).

## Path used

Headless CLI (no GUI launch needed): `src-tauri/target/debug/mochi usage
--provider commandcode --refresh --json`. The CLI `refresh` path resolves the
session cookie via mochi's browser import (`resolve_session_cookie`), runs one
provider fetch through `HttpCommandCodeClient`, parses, and stores the
snapshot — the same code path as the tray refresh.

Note: the GUI build (`pnpm tauri:dev`) was not attempted headlessly; this
machine lacks GTK/WebKit dev packages and requires the `/tmp/opencode/root`
sysroot workaround (see task-5-report.md). The CLI exercises the provider
stack fully; GUI launch remains a Task 9 item.

## Cookie source

- Firefox itself is not used on this machine. The session cookie lives in the
  **Zen browser (Flatpak)** profile:
  `~/.var/app/app.zen_browser.zen/.zen/seq4szqg.Default (release)/cookies.sqlite`
  (Firefox was not running; no sqlite lock issues).
- Cookie name: `__Secure-commandcode_prod_.session_token` on host
  `.commandcode.ai` — matches `SESSION_COOKIE_NAMES` in
  `src-tauri/src/providers/commandcode/credentials.rs`.
- Mochi's Zen discovery already covers Flatpak roots
  (`linux_flatpak_home_relative_root(home, "app.zen_browser.zen", ".zen")`),
  so no code change was needed to find it. **The cookie value was never
  written to any file** (extracted with sqlite3 into a shell variable for the
  curl evidence step; redacted below).

## HTTP status

Both endpoints returned **HTTP 200** with `Accept: application/json` and the
session cookie (verified both via curl evidence capture and via the in-app
CLI fetch, which produced a `fresh` snapshot rather than an auth error).

## Raw live JSON (cookie redacted)

`GET https://api.commandcode.ai/internal/billing/credits` → 200:

```json
{
  "credits": {
    "belowThreshold": false,
    "creditThreshold": 0,
    "monthlyCredits": 15.2591649987,
    "purchasedCredits": 0,
    "premiumMonthlyCredits": 0,
    "opensourceMonthlyCredits": 15.2591649987
  },
  "windowLimits": {
    "limited": true,
    "exceeded": null,
    "fiveHour": { "used": 1.5458561125, "cap": 14, "exceeded": false, "resetAt": 1788461403481 },
    "weekly": { "used": 9.9639946525, "cap": 35, "exceeded": false, "resetAt": 1788978906872 }
  }
}
```

`GET https://api.commandcode.ai/internal/usage/summary` → 200:

```json
{
  "totalCount": 7890,
  "totalCost": 56.74708297929999,
  "averageCost": 0.0071922792115716085,
  "successRate": 99.97465145754119,
  "completedCount": 7888,
  "failedCount": 2,
  "totalTokensIn": 1457290826,
  "totalTokensOut": 3907907,
  "totalTokens": 1461198733,
  "totalCredits": 56.74708297929999,
  "totalFreeCredits": 0,
  "totalMonthlyCredits": 56.74708297929999,
  "totalPurchasedCredits": 0,
  "periodBasis": "billing-period"
}
```

## Shape deltas vs the Task 5 fixture (inferred from HAR)

1. **No `monthly` window at all** in `windowLimits` — the API only exposes
   `fiveHour` and `weekly` (`limited`/`exceeded` sit at the `windowLimits`
   top level, not per-window).
2. Window fields are `used` (absolute) + `cap`, **not** `usedPercent`.
   Percent = `used / cap * 100`.
3. Reset timestamps are `resetAt` **epoch milliseconds** (integer), not
   `resetsAt` ISO strings. Converted to RFC 3339 in the parser via
   `time::OffsetDateTime`.
4. `summary.json` shape matched the fixture (only the values moved).

## Fix (TDD, commit on this branch)

`fix(providers): commandcode live shape alignment` — fixture replaced with
the real captured JSON, failing tests written first, then:

- `parse_credits` now reads `used`/`cap` (computes percent) and converts
  `resetAt` epoch-ms → RFC 3339; window is optional (real payload has no
  `monthly`).
- `snapshot_from_commandcode` uses **Weekly as primary** and **5 hours as
  secondary**; Monthly is an extra window only if the API ever returns it.
  Cost `resets_at` follows the weekly reset.

## Parsed snapshot values (CLI output, live run 2026-09-03T16:08Z)

| Window              | used_percent | resets_at                |
| ------------------- | ------------ | ------------------------ |
| Weekly (primary)    | 28.778957    | 2026-09-09T18:35:06.872Z |
| 5 hours (secondary) | 11.817831    | 2026-09-03T18:50:03.481Z |

Cost snapshot: used **$56.813** / limit **$71.964** USD, period
`billing-period`, resets_at `2026-09-09T18:35:06.872Z`
(limit = totalCost + monthlyCredits remaining, 56.81309490429999 +
15.15052480619999).

Note: values moved slightly between the curl evidence capture and the CLI run
(minutes apart); both are live truth. **Owner should eyeball these against
commandcode.ai/settings/usage** — the numbers cannot be compared against the
user's eyes by the agent.

## Validation run

- `cargo fmt -- --check` — clean
- `cargo clippy --all-targets -- -D warnings` — clean
- `cargo test --all-targets` — 309 + 2 passed, 0 failed
- Live CLI refresh produced `kind: fresh`, `health: ok`
