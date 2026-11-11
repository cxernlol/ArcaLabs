# Arca Labs marketing site — design direction

Filter: [antislop](https://github.com/miqdadbadjuber/anti-slop) core + [antislop-ui](https://github.com/miqdadbadjuber/anti-slop/blob/main/skills/antislop-ui/SKILL.md) for visual work. This file is the soul; skills are the filter.

## Identity

Lightweight Windows streaming app. Automatic encoder settings. Logo: angular “A”, teal on near-black.

## Palette (60 / 30 / 10)

| Share | Role | Hex |
|-------|------|-----|
| 60% | Ground | `#0a0b0c` |
| 30% | Structure (type, panels) | `#e6f2ef` |
| 10% | Accent | `#1295ab` |

Muted supporting text: `#9eb5b0` (must stay AA on ground).

## Typography

**DM Sans** (400/500/600). Reason: geometric, readable, not the default Inter/Geist stack; fits sharp logo without monospace theatrics.

## Mood

Quiet, technical, confident. Dark ground is brand (logo), not generic “tech dark mode.”

## Layout reference

[antislop.ai.id](https://antislop.ai.id/) inspired structure only: editorial hero, **before/after** comparison, read-only config preview. Colors and logo remain Arca (60/30/10 teal).

## Layout

- One primary interactive proof: the **optimize demo** (real states: idle, running, done).
- Side-by-side **Before / After** columns plus monospace **read-only preview** panel.
- No testimonial, FAQ, pricing, or logo bar sections.
- “First run” copy is a single stacked block, not three equal columns.

## Motion

Optimize card reveal only. Respect `prefers-reduced-motion`.

## Dials (liveliness)

ENERGY 2 / RHYTHM 2 / MOTION 1

## Privacy (site only)

- **Vercel Web Analytics** on this marketing deployment: page views, no ties to the desktop app.
- Optimize demo stays client-side; nothing from that flow is sent to our servers.
- Desktop app privacy rules in repo-root `AGENTS.md` are unchanged.

## Purpose notes (antislop-ui)

- **Accent placement (R-29):** Teal only on primary CTA **Optimize for this PC** and optimized values in the After column. Links, borders, and labels stay neutral.
- **Hero wedge:** Flat low-opacity fill + triangle clip, echoes logo. Not a purple/cyan gradient (R-01).
- **Settings preview:** Flat panel mimicking on-disk config keys, not a fake terminal window (R-06, R-38).
- **JetBrains Mono:** Confined to the preview block (readable config), not page headings.
- **Motion (MOTION 1):** Spinner while running only; no infinite pulses (R-19).
- **Radii (R-11):** 2px controls, 8px card; no universal pills.
