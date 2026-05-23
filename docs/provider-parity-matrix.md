# Provider Parity Matrix

> Tracks CodexBar provider coverage vs Mochi v1. CodexBar logic referenced under MIT license
> ([CodexBar](https://github.com/steipete/CodexBar)); see `docs/providers.md` in that repo.

**Legend**

| Column         | Meaning                                                                                                                       |
| -------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| **Strategies** | Ordered fetch strategies (CodexBar auto order)                                                                                |
| **Auth**       | Required credentials / session sources                                                                                        |
| **Settings**   | Notable CodexBar config fields                                                                                                |
| **Status**     | External status page integration                                                                                              |
| **Cost**       | Local session-cost / spend tracking                                                                                           |
| **Mochi**      | `stub` = static placeholder, `partial` = incomplete fetch, `done` = usage bars match CodexBar labels/windows when creds exist |

## Bar parity audit (v1 — 12 providers)

| Provider        | CodexBar windows / bars                                              | CodexBar labels                                 | Extra (cost, reserve, pace)                                 | Mochi current                                                      | Gap                                                                  |
| --------------- | -------------------------------------------------------------------- | ----------------------------------------------- | ----------------------------------------------------------- | ------------------------------------------------------------------ | -------------------------------------------------------------------- |
| **Codex**       | Session, Daily, Weekly (from window seconds / dashboard text)        | Session / Daily / Weekly                        | Session JSONL cost; historical pace for weekly (Codex-only) | OAuth + CLI + web cookies → same labels; `session_cost` enrichment | Historical pace for Codex weekly not ported (linear pace only in UI) |
| **Claude**      | Session, Weekly; Sonnet/Opus weekly when present                     | Session, Weekly, Sonnet weekly, Opus weekly     | Web `extra_usage` USD spend limit                           | OAuth + web → same windows                                         | `extra_usage` spend-limit bar not mapped to `provider_cost` yet      |
| **Cursor**      | Total, Auto+Composer, API                                            | Total, Auto + Composer, API                     | On-demand USD (`providerCost`)                              | Web → Total / Auto + Composer / API + on-demand cost               | —                                                                    |
| **Gemini**      | Pro, Flash, Flash Lite                                               | Pro, Flash, Flash Lite                          | —                                                           | OAuth quota → Pro / Flash / Flash Lite (tertiary)                  | —                                                                    |
| **Copilot**     | Premium, Chat                                                        | Premium, Chat                                   | —                                                           | OAuth internal API → Premium / Chat                                | Device-flow login UI not in Mochi                                    |
| **OpenCode**    | 5-hour, Weekly (+ Monthly when present)                              | 5-hour, Weekly, Monthly                         | —                                                           | Web `_server` → 5-hour, Weekly, optional Monthly                   | —                                                                    |
| **OpenCode Go** | 5-hour, Weekly, Monthly + Zen balance                                | Same + Zen USD                                  | Zen pay-as-you-go (`providerCost`)                          | Web `/go` page + Zen balance parse                                 | —                                                                    |
| **Antigravity** | Claude + Gemini Pro + Gemini Flash (per-model)                       | Model labels from probe                         | Google incidents                                            | Local LSP probe → Claude / Gemini Pro / Gemini Flash               | OAuth multi-account + remote quota not ported                        |
| **Factory**     | Web usage lanes (plan-specific)                                      | Standard / Premium                              | factory.ai status link                                      | Web cookies + CodexBar session → Standard / Premium                | WorkOS refresh + local storage token chain not ported                |
| **z.ai**        | Token window + Monthly (+ short token window tertiary when multiple) | Dynamic window labels (`5 hours`, `Monthly`, …) | Optional model-usage charts                                 | API quota → token + monthly bars                                   | Model-usage chart UI not in Mochi                                    |
| **Kiro**        | Credits %, optional bonus credits                                    | Credits / Bonus                                 | AWS Health link                                             | `kiro-cli` `/usage` → Credits / Bonus                              | Context usage sub-metrics not ported                                 |
| **Augment**     | CLI / web usage lanes                                                | Credits                                         | —                                                           | `auggie` CLI → web cookies → Credits                               | Session keepalive not ported                                         |

### Shared UI / metadata

| Area            | CodexBar                                        | Mochi                                                                         | Gap                                                               |
| --------------- | ----------------------------------------------- | ----------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Window ordering | `primary` → `secondary` → `tertiary` → extras   | `rate_windows()` same                                                         | —                                                                 |
| Reserve / pace  | `UsagePace` + historical samples (Codex weekly) | Linear pace from `resets_at` + label-derived window minutes (`usage-pace.ts`) | Codex historical pace; per-window `windowMinutes` not on snapshot |
| Cost section    | `providerCost` meter                            | `ProviderCostSection` for Cursor on-demand, OpenCode Go Zen                   | Claude `extra_usage` cost                                         |
| Config import   | `tokenAccounts`, cookies                        | CodexBar settings import + workspace ID fields                                | Broader cookie auto-import (Claude, Augment)                      |

> **Zen disambiguation:** The **Zen browser** is a cookie source (used for Cursor/OpenCode import). **Zen balance** is OpenCode Go pay-as-you-go USD shown as `provider_cost`, not a separate AI provider.

## Mochi v1 providers (12) — summary

| Provider    | CodexBar ID         | Usage windows (CodexBar labels)             | Strategies                                 | Auth                                      | Status                    | Cost                              | Mochi     |
| ----------- | ------------------- | ------------------------------------------- | ------------------------------------------ | ----------------------------------------- | ------------------------- | --------------------------------- | --------- |
| Codex       | `codex`             | Session / Daily / Weekly                    | OAuth API → CLI RPC → web dashboard        | OAuth, manual cookie, `codex` CLI         | Statuspage.io (OpenAI)    | JSONL session scan                | **done**  |
| Claude      | `claude`            | Session, Weekly (+ Sonnet/Opus weekly)      | OAuth API → Web API                        | OAuth, `MOCHI_CLAUDE_*`, browser session  | Statuspage.io (Anthropic) | JSONL (planned) / web extra_usage | **done**† |
| Cursor      | `cursor`            | Total, Auto + Composer, API + on-demand USD | Web API                                    | Manual cookie / Zen cookies               | Statuspage.io (Cursor)    | On-demand spend meter             | **done**  |
| Gemini      | `gemini`            | Pro, Flash, Flash Lite                      | OAuth quota API                            | Gemini CLI OAuth                          | Google incidents (manual) | —                                 | **done**  |
| Copilot     | `copilot`           | Premium, Chat                               | OAuth → `copilot_internal` API             | `MOCHI_COPILOT_TOKEN*`                    | Statuspage.io (GitHub)    | —                                 | **done**  |
| OpenCode    | `opencode`          | 5-hour, Weekly (+ Monthly)                  | Web `_server` dashboard                    | Browser cookies, `MOCHI_OPENCODE_COOKIE*` | —                         | —                                 | **done**  |
| OpenCode Go | `opencodego`        | 5-hour, Weekly, Monthly + Zen balance       | Web `_server` + `/go` page + workspace Zen | Cookies + workspace ID                    | —                         | Zen pay-as-you-go balance         | **done**  |
| Antigravity | `antigravity`       | Multi-model quotas                          | Local LSP probe                            | Local Antigravity language server         | Google incidents          | —                                 | **done**† |
| Factory     | `factory` / `droid` | Standard / Premium                          | Web cookies → CodexBar session             | Factory/WorkOS session                    | status.factory.ai         | —                                 | **done**† |
| z.ai        | `zai`               | Token + monthly (+ short-window tertiary)   | API token → quota API                      | `api_key`, `Z_AI_API_KEY`, region host    | none                      | —                                 | **done**  |
| Kiro        | `kiro`              | Credits (+ bonus)                           | CLI `/usage`                               | `kiro-cli` login                          | AWS Health (link)         | —                                 | **done**  |
| Augment     | `augment`           | Credits                                     | CLI → web cookies                          | CLI session, browser cookies              | none                      | —                                 | **done**† |

† Claude usage bars match; optional `extra_usage` cost meter still missing.

## CodexBar full registry (46 providers)

Condensed from [CodexBar `docs/providers.md`](../../CodexBar/docs/providers.md). Mochi v1 targets the 12 rows above; remaining rows are Phase B+ backlog.

| Provider      | Strategies (summary)       | Auth (summary)             | Status               | Cost          | Mochi |
| ------------- | -------------------------- | -------------------------- | -------------------- | ------------- | ----- |
| Codex         | oauth → cli → web cookies  | OAuth, manual cookie, CLI  | OpenAI Statuspage    | session JSONL | done  |
| OpenAI        | Admin API / legacy balance | Admin or API key           | —                    | org usage     | —     |
| Azure OpenAI  | API deployment probe       | API key + endpoint         | Azure status link    | —             | —     |
| Claude        | oauth → web                | Admin, OAuth, cookies, CLI | Anthropic Statuspage | session JSONL | done† |
| Gemini        | OAuth quota API            | Gemini CLI OAuth           | Google incidents     | —             | done  |
| Antigravity   | local probe                | localhost LSP              | Google incidents     | —             | done† |
| Cursor        | web cookies                | manual cookie              | Cursor Statuspage    | on-demand USD | done  |
| OpenCode      | web dashboard              | browser cookies            | —                    | —             | done  |
| OpenCode Go   | web dashboard              | cookies + workspace ID     | —                    | Zen balance   | done  |
| Droid/Factory | web cookies + session      | cookies, CodexBar session  | factory.ai status    | —             | done† |
| z.ai          | API quota                  | API token                  | —                    | —             | done  |
| Kiro          | CLI `/usage`               | `kiro-cli` login           | AWS Health link      | —             | done  |
| Augment       | CLI → web                  | CLI / cookies              | —                    | —             | done† |
| …             | (see CodexBar docs)        | …                          | …                    | …             | —     |

## Gap summary (v1)

| Gap                                       | CodexBar    | Mochi today                                                | Phase       |
| ----------------------------------------- | ----------- | ---------------------------------------------------------- | ----------- |
| Credential store (Keychain/libsecret/Win) | Yes         | Settings JSON (`provider_configs`)                         | 1 (partial) |
| Cookie import                             | Yes         | Cursor + OpenCode + CodexBar import                        | 2+          |
| Usage cache / LKG                         | Yes         | Live fetch every call                                      | 1           |
| Historical pace (Codex weekly)            | Yes         | Linear pace projection only                                | 2+          |
| Real fetch (4 stubs)                      | Yes         | Antigravity OAuth, Factory WorkOS chain, Augment keepalive | 5–6         |
| Claude `extra_usage` cost bar             | Yes         | Not mapped                                                 | 2           |
| OS-native UI shells                       | N/A (Swift) | Custom shadcn                                              | 5           |
| CLI parity                                | Yes         | Skeleton                                                   | 6           |
