# Provider Parity Matrix

> Tracks CodexBar provider coverage vs Mochi v1. CodexBar logic referenced under MIT license
> ([CodexBar](https://github.com/steipete/CodexBar)); see `docs/providers.md` in that repo.

**Legend**

| Column         | Meaning                                                                               |
| -------------- | ------------------------------------------------------------------------------------- |
| **Strategies** | Ordered fetch strategies (CodexBar auto order)                                        |
| **Auth**       | Required credentials / session sources                                                |
| **Settings**   | Notable CodexBar config fields                                                        |
| **Status**     | External status page integration                                                      |
| **Cost**       | Local session-cost / spend tracking                                                   |
| **Mochi**      | `stub` = static placeholder, `partial` = incomplete fetch, `done` = parity target met |

## Mochi v1 providers (10)

| Provider    | CodexBar ID         | Strategies                                                       | Auth                                                                         | Settings fields                                           | Status                     | Cost                              | Mochi       |
| ----------- | ------------------- | ---------------------------------------------------------------- | ---------------------------------------------------------------------------- | --------------------------------------------------------- | -------------------------- | --------------------------------- | ----------- |
| Codex       | `codex`             | OAuth API → CLI RPC → web dashboard (manual cookie)              | OAuth tokens, manual browser cookie, `codex` CLI                             | `cookieSource`, battery saver, history window             | Statuspage.io (OpenAI)     | JSONL session scan (`CODEX_HOME`) | **done**    |
| Claude      | `claude`            | OAuth API → Web API (manual cookie); Admin API + CLI PTY planned | OAuth (`~/.claude/.credentials.json`), `MOCHI_CLAUDE_*` env, browser session | Admin key, token accounts, `cookieSource`, history window | Statuspage.io (Anthropic)  | JSONL project logs (planned)      | **partial** |
| Cursor      | `cursor`            | Web API (`MOCHI_CURSOR_COOKIE*`); browser import planned         | Manual cookie header via env/file                                            | `cookieSource`, manual cookie header                      | Statuspage.io (Cursor)     | —                                 | **partial** |
| Gemini      | `gemini`            | OAuth-backed quota API (Gemini CLI credentials)                  | Google OAuth via Gemini CLI (`~/.gemini/oauth_creds.json`, `MOCHI_GEMINI_*`) | CLI credential path overrides                             | Google Workspace incidents | —                                 | **partial** |
| Copilot     | `copilot`           | OAuth token → `copilot_internal` API; device-flow UI planned     | `MOCHI_COPILOT_TOKEN*`, GitHub OAuth token(s)                                | Token accounts, enterprise host                           | Statuspage.io (GitHub)     | —                                 | **partial** |
| Antigravity | `antigravity`       | Local LSP/HTTP probe (`GetUserStatus`)                           | Local Antigravity language server                                            | Host/port overrides                                       | Google Workspace incidents | —                                 | **stub**    |
| Factory     | `factory` / `droid` | Web cookies → stored tokens → local storage → WorkOS cookies     | Factory/WorkOS session, bearer tokens                                        | `cookieSource`, manual cookie                             | status.factory.ai (link)   | —                                 | **stub**    |
| z.ai        | `zai`               | API token → quota API                                            | `providers[].apiKey`, `Z_AI_API_KEY`                                         | API key, region/host overrides                            | none                       | —                                 | **stub**    |
| Kiro        | `kiro`              | CLI: `kiro-cli chat --no-interactive "/usage"`                   | AWS Builder ID via `kiro-cli` login                                          | —                                                         | AWS Health (manual link)   | —                                 | **stub**    |
| Augment     | `augment`           | CLI (`auggie`) → web cookies fallback                            | CLI session, browser cookies                                                 | `cookieSource`, manual cookie                             | none                       | —                                 | **stub**    |

## CodexBar full registry (46 providers)

Condensed from [CodexBar `docs/providers.md`](../../CodexBar/docs/providers.md). Mochi v1 targets the 10 rows above; remaining rows are Phase B+ backlog.

| Provider             | Strategies (summary)         | Auth (summary)                  | Status               | Cost              | Mochi   |
| -------------------- | ---------------------------- | ------------------------------- | -------------------- | ----------------- | ------- |
| Codex                | oauth → cli → web cookies    | OAuth, manual cookie, CLI       | OpenAI Statuspage    | session JSONL     | done    |
| OpenAI               | Admin API / legacy balance   | Admin or API key                | —                    | org usage         | —       |
| Azure OpenAI         | API deployment probe         | API key + endpoint + deployment | Azure status link    | —                 | —       |
| Claude               | oauth → web (admin/cli TBD)  | Admin, OAuth, cookies, CLI      | Anthropic Statuspage | session JSONL     | partial |
| Gemini               | OAuth quota API              | Gemini CLI OAuth                | Google incidents     | —                 | partial |
| Antigravity          | local probe                  | localhost LSP                   | Google incidents     | —                 | stub    |
| Cursor               | web (`MOCHI_CURSOR_COOKIE*`) | manual cookie env/file          | Cursor Statuspage    | —                 | partial |
| OpenCode             | web dashboard                | browser cookies                 | —                    | —                 | —       |
| OpenCode Go          | web dashboard                | cookies + workspace ID          | —                    | —                 | —       |
| Alibaba Coding Plan  | web RPC; API fallback        | cookies, API key                | Aliyun status link   | —                 | —       |
| Droid/Factory        | web multi-fallback           | cookies, tokens, WorkOS         | factory.ai status    | —                 | stub    |
| z.ai                 | API quota                    | API token                       | —                    | —                 | stub    |
| Manus                | web credits API              | `session_id` cookie             | —                    | —                 | —       |
| MiniMax              | web or API token             | cookies / API key               | —                    | billing charts    | —       |
| Kimi                 | web usage API                | `kimi-auth` JWT                 | —                    | —                 | —       |
| Kilo                 | API; CLI fallback            | API key / CLI auth file         | —                    | —                 | —       |
| Copilot              | OAuth internal API           | `MOCHI_COPILOT_TOKEN*`          | GitHub Statuspage    | —                 | partial |
| Kimi K2 (unofficial) | API credits                  | API key                         | —                    | —                 | —       |
| Kiro                 | CLI `/usage`                 | `kiro-cli` login                | AWS Health link      | —                 | stub    |
| Vertex AI            | OAuth ADC + Monitoring       | gcloud ADC                      | —                    | Claude-log filter | —       |
| Augment              | CLI → web                    | CLI / cookies                   | —                    | —                 | stub    |
| JetBrains AI         | local XML quota              | IDE config dir                  | —                    | —                 | —       |
| Amp                  | web settings HTML            | browser cookies                 | —                    | —                 | —       |
| Warp                 | GraphQL limits               | API token                       | —                    | —                 | —       |
| ElevenLabs           | subscription API             | API key                         | status link          | —                 | —       |
| Windsurf             | web localStorage → SQLite    | browser session                 | —                    | —                 | —       |
| Ollama               | web settings                 | cookies / API key               | —                    | —                 | —       |
| Synthetic            | API quota                    | API key                         | —                    | —                 | —       |
| OpenRouter           | credits API                  | API token                       | status link          | spend windows     | —       |
| Perplexity           | web credits                  | cookies / session token         | —                    | —                 | —       |
| Xiaomi MiMo          | web balance                  | browser cookies                 | —                    | —                 | —       |
| Doubao               | API probe                    | API key                         | —                    | —                 | —       |
| Abacus AI            | web billing                  | browser cookies                 | —                    | —                 | —       |
| Mistral              | console billing              | Ory session cookies             | —                    | —                 | —       |
| DeepSeek             | balance API                  | API key                         | —                    | —                 | —       |
| Moonshot             | balance API                  | API key                         | —                    | —                 | —       |
| Codebuff             | usage API                    | API token / CLI login           | —                    | —                 | —       |
| Crof                 | credits API                  | API key                         | —                    | —                 | —       |
| Venice               | balance API                  | API key                         | —                    | —                 | —       |
| Command Code         | web billing                  | session cookies                 | —                    | —                 | —       |
| StepFun              | web login / token            | username/password or token      | —                    | —                 | —       |
| AWS Bedrock          | Cost Explorer                | AWS credentials                 | —                    | budget tracking   | —       |
| Grok                 | CLI → web gRPC               | CLI / Chrome cookies            | —                    | —                 | —       |
| GroqCloud            | Prometheus metrics           | API key                         | —                    | —                 | —       |
| LLM Proxy            | `/v1/quota-stats`            | API key + base URL              | —                    | —                 | —       |
| Deepgram             | usage breakdown              | API key                         | —                    | —                 | —       |

## Gap summary (v1)

| Gap                                       | CodexBar    | Mochi today                | Phase         |
| ----------------------------------------- | ----------- | -------------------------- | ------------- |
| Credential store (Keychain/libsecret/Win) | Yes         | None                       | 1 (stub) → 2+ |
| Cookie import                             | Yes         | None                       | 2+            |
| Usage cache / LKG                         | Yes         | Live fetch every call      | 1             |
| Provider metadata registry                | Yes         | Minimal `ProviderMetadata` | 1             |
| Real fetch (8 stubs)                      | Yes         | Static snapshots           | 3–4           |
| Codex full parity                         | Yes         | Partial CLI/cookies        | 2             |
| OS-native UI shells                       | N/A (Swift) | Custom shadcn              | 5             |
| CLI parity                                | Yes         | Skeleton                   | 6             |
| Release smoke                             | Yes         | Shallow                    | 7             |
