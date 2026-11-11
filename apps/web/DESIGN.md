# Arca Labs marketing site — design direction

## Identity

Lightweight Windows streaming. Logo: angular “A”, teal on near-black.

## Palette (60 / 30 / 10)

| Share | Role | Hex |
|-------|------|-----|
| 60% | Ground | `#0a0b0c` |
| 30% | Structure (type, panels) | `#e6f2ef` |
| 10% | Accent (logo teal) | `#1295ab` |

Muted: `#9eb5b0` (AA on ground). Red is for live state on the **desktop app** only — not used on this site.

## Typography

**DM Sans** (400/500/600). Geometric, readable, not Inter/Geist. No monospace costume on the marketing page.

## Layout

1. Header (logo wordmark + Optimize / GitHub)
2. Honest prototype banner
3. Brand title + one lede
4. **Hero: Before/After optimize demo** (the one memorable element)
5. First-run numbered steps (real sequence)
6. Short decisions changelog (specificity over template filler)
7. Footer

No logo wedge, eyebrow badge, YAML preview, testimonials, FAQ, or pricing.

## Motion

Optimize spinner while running only. Respect `prefers-reduced-motion`.

## Radii

6px controls, 10px panels.

## Accent placement

Teal only on **Optimize for this PC** and optimized After values. Nav links stay neutral.

## Privacy (site only)

Vercel Web Analytics on the hosted site. Desktop app: no telemetry (see `docs/internal/AGENTS.md`).
