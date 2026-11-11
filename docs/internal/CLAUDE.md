# CLAUDE.md

@docs/internal/AGENTS.md

## Core rules (summary)

1. **Spec first.** Don't write implementation code until requirements are clear. Ask clarifying questions about edge cases, UI states and data inputs, then write a concise `SPEC.md` (Goal & User Story, Inputs & Outputs, Acceptance Criteria, Edge Cases & Error States, Out of Scope) and wait for approval.
2. **Build only what the approved spec lists.** No unrequested abstractions, extra dependencies or bonus features. If something is uncovered, stop and ask.
3. **No data collection.** No telemetry, analytics, crash reporting or automatic reporting to GitHub or any other service. The only outbound connection is the stream to YouTube, Kick or Twitch. Any other network call, including update checks, must be in the approved spec and off or opt-in by default.
4. **Stream key** lives in the OS keychain, never in plain text, config or logs.
5. **UI** follows `design.md` (Lucide icons only, Inter bundled locally, nested radius = outer radius − gap).

Full rules, workflow and the `SPEC.md` template are in `docs/internal/AGENTS.md`. If anything here conflicts with that file, `AGENTS.md` wins.
