# AGENTS.md

Instructions for any AI coding agent (Claude Code, Codex, Gemini CLI, Cursor, Copilot, Windsurf, Cline and others) working in this repository. Read this file before starting any task.

## Project overview

Arcalabs is a desktop streaming app for YouTube, Kick and Twitch, built with **Tauri** (Rust core, system webview frontend). It picks the encoder, resolution, frame rate and bitrate for the user's PC automatically, with manual control available underneath. It collects no user data.

- **Icons:** Lucide only.
- **Font:** Inter, bundled locally.
- **Platform:** offline-capable. No CDN assets.

## Reference files

- `design.md`: the visual and interaction design system. All UI work follows it (Lucide icons, Inter, nested radius rule, color tokens, component specs).
- `SPEC.md`: the approved spec for the feature being built. If it doesn't exist or isn't approved, don't write implementation code.

---

## Specification and planning rules

- Never start writing implementation code if the requirements are vague or missing acceptance criteria.
- Before coding any new feature, ask clarifying questions about edge cases, UI states and data inputs. Then write a concise `SPEC.md` and wait for approval.
- Only build what is explicitly listed in the approved spec. Do not add unrequested abstractions, extra dependencies or bonus features.
- If something comes up during implementation that the spec doesn't cover, stop and ask. Don't decide silently, and don't widen the scope.
- If the spec needs to change, update `SPEC.md`, get approval for the change, then continue.

### What counts as vague

Treat a request as vague, and ask before building, when any of these are true:

- No testable definition of "done".
- Unclear inputs, such as which fields, which payloads or which arguments.
- Unclear failure behavior, such as what the user sees when it breaks.
- UI states are unspecified (empty, loading, error, disabled, live/offline).
- It's unclear whether a new dependency or abstraction is wanted.

---

## Privacy rules (non-negotiable)

Arcalabs does not collect user data. These rules apply to every feature, every dependency and every spec.

- **No data collection.** No telemetry, analytics, usage tracking, crash reporting, device fingerprinting or "anonymous" statistics. This includes hardware info gathered by the optimize routine: it is read locally, used locally, and never sent anywhere.
- **No automatic reporting.** The app never sends logs, errors, crash dumps, settings, system info or any other data to GitHub (issues, gists, API) or to any other remote service, and never opens issues or pull requests on the user's behalf.
- **Network access is limited to what the user asked for.** The only outbound connection is the stream itself, to the platform the user chose (YouTube, Kick or Twitch). Any other network call, including update checks, needs to be listed in the approved `SPEC.md` first and must be off or opt-in by default.
- **Data stays on the device.** Settings, scenes, logs and the pre-optimize snapshot (for Undo) are stored locally in the app config directory. The stream key goes in the OS keychain, never in plain text and never in logs.
- **Logs are local and scrubbed.** Never write the stream key, account names or file paths containing the user's name to logs.
- **No dependencies that phone home.** Before adding any crate or npm package, check that it doesn't include telemetry. Don't add a dependency without it being in the spec.
- **Reporting a problem is the user's action.** If a feature ever helps with bug reports, it may only prepare text for the user to review and copy. The user decides what to share and where to send it.
- **Enforce it in Tauri.** Keep the capability list minimal, set a strict CSP, and don't grant HTTP or shell permissions beyond what the approved spec needs.

When writing any spec, include the line "Network calls: none except the stream connection" in Inputs & Outputs, or list each additional call and why it's needed.

---

## Required spec structure

Every feature or app is defined in `SPEC.md` with these five sections, in this order. A spec missing any of them is not ready for approval.

### 1. Goal & User Story
Who the feature is for and the exact problem it solves, in two sentences.

### 2. Inputs & Outputs
The exact data entering the feature (API payloads, form fields, CLI arguments, Tauri command arguments, events) and the exact expected result.

### 3. Acceptance Criteria (Done Definition)
Bulleted, testable conditions that must all be true before the task is complete. Each bullet should be checkable by running the app or a test, and not open to opinion.

### 4. Edge Cases & Error States
What happens when things fail: empty states, invalid inputs, network timeouts, unauthorized users, missing hardware, and anything else relevant. State the visible behavior for each.

### 5. Out of Scope (Non-Goals)
What must not be built right now, to prevent over-engineering and scope creep.

---

## Workflow

1. **Clarify.** Ask questions about edge cases, UI states and data inputs. Keep them short and numbered, and ask all of them in one message.
2. **Draft.** Write `SPEC.md` using the five sections. Keep it concise.
3. **Approve.** Wait for explicit approval. No implementation code before it.
4. **Build.** Implement only what the spec lists. No extra dependencies, abstractions or bonus features.
5. **Verify.** Check the work against each acceptance criterion and report which ones pass.

---

## Template

```markdown
# SPEC: <feature name>

## 1. Goal & User Story
<Who it's for and the problem it solves, in two sentences.>

## 2. Inputs & Outputs
**Inputs**
- <field / payload / argument: type, constraints>

**Outputs**
- <exact result, including UI changes and emitted events>

## 3. Acceptance Criteria
- [ ] <testable condition>
- [ ] <testable condition>

## 4. Edge Cases & Error States
| Situation | Behavior |
|---|---|
| <empty / invalid / timeout / unauthorized / ...> | <what the user sees and what the app does> |

## 5. Out of Scope
- <thing not to build now>
```
