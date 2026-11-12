# Arca Labs marketing site

Design for `apps/web`. The site is a product briefing for streamers: the actual v0.1 prototype is the lead visual, not a fictional live dashboard.

Design read: restrained dark interface, real product imagery, and one violet accent.
Dials: ENERGY 2, RHYTHM 2, MOTION 2.

## Palette

| Token | Hex | Role |
|---|---|---|
| `--obsidian-black` | `#0B0B0D` | 60% foundation: page ground and large dark regions |
| `--icy-white` | `#F2F4F8` | 30% structure: primary type and selected light surfaces |
| `--radiant-violet` | `#8B5CF6` | 10% accent: primary actions and meaningful emphasis |
| `--radiant-violet-hover` | `#A78BFA` | Hover state for violet actions |
| `--surface-dark` | `#151519` | Raised dark panels |
| `--border-dark` | `#29292F` | Quiet dividers and panel borders |
| `--text-muted` | `#A1A1AA` | Secondary text |

The 60/30/10 split guides visual emphasis; it is not a requirement to match exact pixel counts. Use Obsidian Black for the dominant ground, Icy White for clear reading and selected surfaces, and keep Radiant Violet to actions and a few meaningful highlights. Use dark text on violet controls.

## Type and assets

- Use the existing locally bundled DM Sans family.
- Keep imagery, fonts, and icons local. Use the supplied logo, prototype screenshots, and the existing workflow diagram.
- Use the home screenshot in a horizontal 16:10 preview frame, cropped above its simulated CPU/FPS footer values so the real setup controls remain visible. Use the logo as a blurred backdrop behind one translucent preview surface.
- Keep screenshot text crisp when optimizing image files. Give meaningful images descriptive alt text.

## Page structure

1. Header with wordmark, roadmap/workflow navigation, and GitHub.
2. Split hero: product statement and accurate v0.1 copy beside the actual prototype preview.
3. Roadmap from Q4 2026 v0.1 through Q4 2027 v1.0.
4. Existing workflow diagram with its current reduced-motion behavior.
5. Footer with Privacy, Terms, copyright `© 2026 Arca Labs`, and privacy controls.
6. Separate Privacy, Terms, and custom 404 pages. The 404 page offers useful links back to the site.

## Interaction and privacy

- The prototype is not described as live-capable. Do not show the simulated CPU/FPS values as real measurements.
- Vercel Web Analytics is limited to the hosted site and loads only after explicit consent. Rejection or unavailable local storage leaves it disabled. The desktop app has no analytics.
- Show the consent choice in a compact, non-modal popup fixed above the viewport's bottom safe area. Keep the surface opaque and the page interactive so the prompt stays readable without shifting content.
- Internal navigation stays in the same tab; external links open in a new tab with safe `rel` attributes.
- Keep the workflow diagram usable with reduced motion enabled.
- Support visible keyboard focus, readable contrast, and layouts at phone, tablet, and desktop widths.
- Use a one-time Framer Motion reveal for the homepage hero and workflow heading/diagram. Each roadmap milestone uses a restrained fade from 0.85 to 1 opacity and a 24px vertical shift between 55% and 45% of the viewport, above the consent popup, so milestones reveal one at a time and reverse with scroll direction. Keep text readable, avoid pinning or delayed access, and show content immediately when reduced motion is enabled.

## Decision notes

- The split desktop hero puts the real prototype beside the product claim; narrow screens stack the same two elements in reading order.
- The 16:10 screenshot crop keeps the setup controls visible while excluding the synthetic CPU/FPS status strip.
- DM Sans stays because it is already bundled locally and is the site's established typeface.
- The roadmap's reversible 24px milestone shift makes each reveal noticeable at ordinary scroll speeds without hiding copy or delaying access. Its interval sits above the consent popup and keeps neighboring reveals distinct; the hero and workflow retain their one-time reveals.
- Violet is reserved for the main action and consent choice; roadmap labels and offline status use neutral text so the accent stays selective.
- A compact bottom consent popup keeps the privacy choice visible without pushing the hero down or dimming the page.
- The existing workflow diagram keeps its own scroll interaction; Framer Motion does not alter it.

## SEO and content

- Give the homepage and each legal page its own title, meta description, canonical URL, and one clear H1.
- Keep public paths short and readable. Preserve `/flow.html`.
- Include the homepage and legal pages in the sitemap; allow indexing in `robots.txt`. Do not index or list the 404 page.
- Proofread visible copy and remove placeholders and default stock assets. Do not invent contact details or a governing-law jurisdiction.

## Motion

Keep motion deliberate: the hero and workflow reveal once, while each roadmap milestone follows its own viewport progress between 55% and 45% of the viewport. This keeps reveals above the consent popup, distinct, and reversible when scrolling back up. Respect `prefers-reduced-motion` by showing all content immediately.
