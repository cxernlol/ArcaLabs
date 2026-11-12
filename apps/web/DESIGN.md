# Arca Labs marketing site

Direction for `apps/web`. Dark ground is the logo, not a generic tech theme. One accent: logo teal `#1295ab`. Red is live state on the desktop app only.

## Palette

| Role | Hex |
|------|-----|
| Ground | `#0a0b0c` |
| Type | `#e6f2ef` |
| Muted (AA on ground) | `#9eb5b0` |
| Accent | `#1295ab` |

The page is near-black, a little lighter at the top so the header has air. Teal is not used on this screen. The primary button is the light type color.

## Type

DM Sans 400/500/600. Not Inter. No monospace on this page.

Radii: 6px controls, 10px panels. No shadow, glow, or gradient.

## Page, in order

1. Header, one row: wordmark, Workflow in the center, GitHub on the right.
2. One centered headline, one paragraph, and a GitHub button. No encoder table, no numbered steps, no download button. v0.1.0 has no installer.
3. The workflow diagram, on this same page. Scrolling through it moves the view along the nodes, then back out to the whole diagram. The header link scrolls to it. Reduced motion keeps the diagram still.
4. One footer line: prototype, license, analytics.

No decisions changelog, no YAML preview, no logo art, no eyebrow, no testimonials, no pricing.

`/flow.html` sends people back to the diagram on the homepage.

## Motion

The workflow camera follows the page scroll through the diagram. Reduced motion keeps the diagram still and usable.

## Privacy

Vercel Web Analytics on this site only. The desktop app collects nothing (`docs/internal/AGENTS.md`).
