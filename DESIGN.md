# Arcalabs: [DESIGN.md](http://DESIGN.md)

Design direction for the Arcalabs desktop app and its marketing site. Written to be read by people and by AI coding agents.

> **Status:** v0.1 offline prototype. There is no capture, no RTMP publish and no Go Live yet. Hex values and pixel measurements are estimated from screenshots (frame ≈ 1372 × 882). Treat them as starting tokens and verify against source. Contrast ratios in section 3 were calculated, not estimated. Values live in `tokens.css` next to this file. This file holds the intent, `tokens.css` holds the values. Keep them in sync.

---

## 0. Open decisions

These need the owner's answer. Agents must not pick for the owner. If a build collides with one of these, name the element and the rule, and ask.

| #   | Decision         | Current state                                                                                         | Why it is open                                                                                                                                                        |
| --- | ---------------- | ----------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0.1 | **Accent hue**   | Native app: Acid Lime (`#C6F14A`), selected by the user. Website keeps its separately approved violet palette. | The native desktop follows the latest app-specific palette choice. Revisit the website only as part of a website design request. |
| 0.2 | **Typeface**     | Inter, bundled.                                                                                       | Inter is a common default. Write a brand reason, or use Segoe UI Variable (native on Windows 11, zero font bytes, fits "lightweight").                                |
| 0.3 | **Icon set**     | Lucide, required. `zap` for Optimize.                                                                 | The uniform Lucide look and the lightning glyph are both common defaults. Write why they fit, or choose another set or no icon for Optimize.                          |
| 0.4 | **Dark only**    | Dark only, no light theme.                                                                            | Dark must be a decision, not a trend. Write the reason.                                                                                                               |
| 0.5 | **Product name** | "Arcalabs" (title bar), "Arca Labs" (site, nav), "ArcaLabs" (settings text, repo).                    | Pick one spelling and use it everywhere.                                                                                                                              |

Reasons (proposed drafts, not final; edit or replace):

- 0.1 Accent: Acid Lime (`#C6F14A`) for the native app, selected by the user. The website keeps its separately approved violet palette.
- 0.2 Typeface: keep Inter. One family with tabular figures keeps the status bar and timer steady, stays legible at small sizes on every OS, and is bundled once for both app and site. Reconsider Segoe UI Variable only if installer size matters.
- 0.3 Icons: keep one set (Lucide) at 1.5 stroke so icons stay as quiet as the chrome. Drop the icon on Optimize; the label is enough.
- 0.4 Dark only: streams are set up in dim rooms and the preview should be the brightest thing on screen. Light chrome would compete with it.
- 0.5 Name: **open.** One spelling everywhere.

---

## 1. Product intent

**What it is:** A desktop streaming app for YouTube, Kick and Twitch, in the same category as OBS but built for people who don't want to configure encoders.

**Audience:** New and casual streamers who want to go live quickly. Experienced streamers are secondary, so manual controls stay available but sit one level down.

**Primary job:** Get from launch to "Live" in as few steps as possible. No duration is promised in the UI or on the site until a real first run has been timed.

**Core idea:** The app chooses the encoder, resolution, frame rate and bitrate for the PC. The interface presents those choices as a result to confirm or undo. It does not present them as a form to fill in.

### 1.1 Identity and direction

Proposed direction. Edit freely; everything else in this file follows from it.

- **Personality:** calm, plain-spoken, exact. A tool that stays out of the way and says what it did.
- **Mood:** a quiet dark desk during a stream. The preview is the light source and the chrome recedes.
- **Memorable element, one per surface:**
  - App: the status bar. It shows the app's own CPU and RAM, which is the evidence for "lightweight".
  - Site: the Before / After diff with Undo.
- **The arc:** the three-arc mark is the logo and nothing else. It is not repeated as decoration, background or pattern.
- **Voice:** short verbs, concrete nouns, no hype. It states what changed and what to do next.
- **Should not feel like:** a gamer overlay or an OBS clone. No RGB, neon, angled shapes or skins. Not a generic SaaS landing page, and not a terminal costume.
- **Test:** if a screen could belong to any other streaming app by swapping the logo, it is not designed yet.

---

## 2. Principles

1. **One obvious action per screen.** Main window: *Go Live*. Settings: *Done*. Setup: *Finish setup*. Each is the only solid accent fill on its screen.
2. **Automatic first, manual on request.** Manual controls are present but secondary. They show as a collapsed row ("Manual settings"), or as disabled fields until Automatic is turned off.
3. **Show what changed.** After optimizing, display old → new for every value, with the old value struck through, and offer **Undo**. Make the change reversible and visible instead of asking for confirmation up front.
4. **State is visible everywhere.** Live status appears in the header, the primary button, the status bar and the stream panel at the same time. Nobody should wonder whether they are live.
5. **Quiet chrome, loud content.** The UI is dark and low-contrast so the stream preview and the single accent color carry attention.
6. **Plain language.** Labels say what the thing does ("Optimize for this PC", "Be right back") and avoid encoder jargon. Technical detail (NVENC · 1080p60 · 6,000 kbps) appears as small supporting text and never as a label.
7. **Say what's real.** Anything that is a stub, a simulation or a placeholder is labeled as one where the user can see it. No invented numbers, no controls that look live but do nothing, no unexplained disabled buttons.

---

## 3. Color

### Surfaces

For the desktop app, keep the visual weight near 60/30/10: Obsidian Black grounds the workspace, Icy White carries readable content, and Acid Lime marks the primary action and real active controls. Lime is an accent, not a panel fill. The marketing site retains its separately approved palette.

| Token           | Hex (approx.) | Use                                                                           |
| --------------- | ------------- | ----------------------------------------------------------------------------- |
| `bg.app`        | `#0F1013`     | Window background, settings content area                                      |
| `bg.panel`      | `#17181C`     | Cards and panels (Scenes, Sources, Audio, Stream), setup card, preview canvas |
| `bg.sunken`     | `#0C0D10`     | Inputs, segmented-control track, status and title bars                        |
| `bg.letterbox`  | `#070709`     | Letterbox bars beside the 16:9 preview                                        |
| `bg.selected`   | `#23291A`     | Native app selected list or nav item (lime-tinted)                             |
| `bg.raised`     | `#26272D`     | Active segment in a segmented control, disabled primary button, meter track   |
| `border.subtle` | `#25262C`     | Panel, input and divider borders, 1px                                         |

### Text

| Token            | Hex       | Use                                                                                        |
| ---------------- | --------- | ------------------------------------------------------------------------------------------ |
| `text.primary`   | `#F2F2F5` | Titles, active labels, values after change                                                 |
| `text.secondary` | `#A3A4AC` | Panel titles, inactive tabs, supporting text, status bar, reasons beside disabled controls |
| `text.muted`     | `#8E8F98` | Placeholders, struck-through old values, de-emphasized text                                |

> Changed from the first draft: `text.muted` was `#6E6F78`, which measured 3.55:1 on panels. Struck-through old values are information and placeholders are not exempt, so it now passes 4.5:1 on every surface.

### Accent (Acid Lime — native app)

| Token          | Hex (approx.) | Use                                                                                |
| -------------- | ------------- | ---------------------------------------------------------------------------------- |
| `accent.solid` | `#C6F14A`     | Primary buttons (*Go Live*, *Done*, *Finish setup*), toggle on-state               |
| `accent.line`  | `#C6F14A`     | Outline of secondary action, audio level fill, checkmark, logo mark                |
| `accent.text`  | `#C6F14A`     | Label of the outlined secondary button ("Optimize for this PC", "Re-run optimize") |

### Status

| Token           | Hex (approx.) | Use                                                                  |
| --------------- | ------------- | -------------------------------------------------------------------- |
| `status.live`   | `#F25555`     | Live dot and "Live · 00:12:41" in the header                         |
| `status.danger` | `#C4373B`     | *Stop stream* button fill                                            |
| `status.idle`   | `#6E6F78`     | Offline dot (a dot, not text, so the 3:1 non-text threshold applies) |

### Measured contrast

| Pair                                                                               | Ratio                 |
| ---------------------------------------------------------------------------------- | --------------------- |
| `text.primary` on `bg.panel`                                                       | 15.9                  |
| `text.secondary` on `bg.panel`                                                     | 7.2                   |
| `text.muted` (`#8E8F98`) on `bg.panel` / `bg.sunken` / `bg.selected` / `bg.raised` | 5.5 / 6.0 / 5.2 / 4.6 |
| `text.secondary` on `bg.raised`                                                    | ≈ 6.0                 |
| Black (`#0B0B0D`) on `accent.solid`                                                | 15.1                   |
| `accent.text` on `bg.panel`                                                       | 13.6                   |
| `accent.line` on `bg.panel`                                                       | 13.6                   |
| `status.live` on `bg.app`                                                          | 5.6                   |
| White on `status.danger`                                                           | 5.3                   |

**Rules**

- In the native app, Acid Lime is the only accent; red appears only for live state and stopping. The website palette is maintained separately.
- The accent solid fill is used once per screen. Everything else that needs emphasis uses the outlined style.
- The accent belongs at the key moment only. It is not spread across icons, badges, links and backgrounds.
- No gradients, no glow, no blurred orbs, no glassmorphism.
- Never use pure black or pure white. Surfaces are tinted, and text tops out at `#F2F2F5`.

---

## 4. Typography

**Family:** Inter (single family, weights 400 / 500 / 600), pending decision 0.2. Fall back to `system-ui, -apple-system, "Segoe UI", sans-serif`.

| Role                    | Size     | Weight  | Color     | Example                                                       |
| ----------------------- | -------- | ------- | --------- | ------------------------------------------------------------- |
| Window title / wordmark | 18 px    | 600     | primary   | "Arcalabs"                                                    |
| Page heading            | 22–24 px | 600     | primary   | "Video", "Let's get you live"                                 |
| Primary button          | 17 px    | 600     | white     | "Go Live"                                                     |
| Body / row label        | 15–16 px | 400–500 | primary   | "Resolution", "Webcam"                                        |
| Panel title             | 14 px    | 400     | secondary | "Scenes", "Audio"                                             |
| Helper / description    | 14 px    | 400     | secondary | "Picks the best encoder, resolution and bitrate for this PC." |
| Spec line               | 13–14 px | 400     | secondary | "NVENC · 1080p60 · 6,000 kbps"                                |
| Status bar              | 14 px    | 400     | secondary | "CPU 4% · RAM 62 MB"                                          |

**Rules**

- Sentence case everywhere ("Be right back", "Stop stream"). No all-caps labels, no wide-tracked uppercase.
- No monospace as a style. Monospace is only for text that is literally code or a literal value (an ingest URL may be monospace).
- Numerals in the status bar and header timer use tabular figures so values don't jitter while streaming.
- Weight is the main hierarchy tool. Keep size changes modest.
- Minimum text size is 12 px. Nothing smaller, including labels.
- Line length stays under 80 characters for any paragraph.

---

## 5. Layout

### 5.1 Main window

```
┌──────────────────────────────────────────────────────────┐
│ ◠ Arcalabs                                    ● Offline  │  header ~58px
├──────────────────────────────────────────────────────────┤
│ ┌──────────────────────────────────────────────────────┐ │
│ │ ▓▓▓▓▓│            Screen 1 · 1920×1080         │▓▓▓▓▓ │ │  preview, 16:9
│ └──────────────────────────────────────────────────────┘ │  letterboxed
│ ┌────────┐ ┌─────────┐ ┌────────────────┐ ┌───────────┐ │
│ │ Scenes │ │ Sources │ │ Audio          │ │ Stream    │ │  control row
│ │        │ │         │ │                │ │           │ │
│ └────────┘ └─────────┘ └────────────────┘ └───────────┘ │
├──────────────────────────────────────────────────────────┤
│ Not streaming     CPU 4%  RAM 62 MB  — fps  — kbps  —    │  status bar
└──────────────────────────────────────────────────────────┘

```

- **Preview** takes the top ~45% and is the largest object on screen. The 16:9 canvas is centered with darker letterbox bars at the sides. It is one frame, not two nested frames. The canvas never overflows the window: fix it with `aspect-ratio: 16 / 9` and `object-fit: contain`.
- **Preview empty state** (no capture yet): a short message and the source name, 14 px or larger, `text.secondary`. Example: "No capture yet" with "Screen 1 · 1920×1080" beneath it.
- **Control row** is four panels with a fixed gap (~16 px). Approximate widths, left to right: Scenes 244, Sources 268, Audio 406, Stream 350. Audio is widest because it holds the meters. Stream is the most important panel, so it carries the primary button at its bottom.
- **Outer padding:** 20 px. **Gap between panels:** 16 px.
- **Status bar:** the left side holds the state message and the right side holds metrics, aligned right.
- **Minimum window size:** about 1100 × 700, so the control row never collapses.

### 5.2 Settings

```
┌───────────────┬──────────────────────────────────────────┐
│ ◠ Arcalabs    │                                  [ Done ]│
├───────────────┼──────────────────────────────────────────┤
│ Stream        │  Video                                   │
│ Video  ◀      │  ┌────────────────────────────────────┐  │
│ Audio         │  │ Automatic        [Re-run] [toggle] │  │
│ Hotkeys       │  └────────────────────────────────────┘  │
│ Advanced      │  Manual · turn off Automatic to edit     │
│               │  Resolution ................ [ 1920×1080 ▾]│
│               │  Frame rate ................ [ 60 fps    ▾]│
└───────────────┴──────────────────────────────────────────┘

```

- Left nav is ~268 px wide, separated from content by a 1px divider. The selected item uses `bg.selected` with a 12 px radius.
- Content column is capped at ~810 px and left-aligned, leaving empty space to its right instead of stretching.
- Manual rows are `label ........ select`, separated by 1px hairlines. Selects are ~290 px wide and aligned right.
- Pressing Esc closes settings. The background behind a modal is dimmed.

### 5.3 First-run setup

- A single centered card, ~636 px wide, on `bg.app`.
- Three steps in a vertical sequence. Numbered badges are appropriate here because the content is a real sequence.
- Step content is indented under the title. The card's footer holds the full-width primary button.
- If no stream key is saved, pressing Go Live opens Settings → Stream. It does not sit disabled with no explanation.

**Alignment:** Left-aligned text throughout. Centering is reserved for the setup card as a whole, the preview label, and button labels.

---

## 6. Shape, spacing and elevation

### Nested radius rule

> **Inner radius = outer radius − gap**

When a rounded element sits inside another rounded element, its corner radius is the parent's radius minus the distance between the two edges (the padding or inset). This keeps the two curves concentric, so the space between them looks even around the corner. Mismatched radii make the corners look pinched or bulging.

- **Gap** is the distance from the parent's edge to the child's edge on the side of the corner. Count the parent's border if the child sits against it.
- **Clamp at zero.** `inner = max(0, outer − gap)`. If the gap is as large as the outer radius, the child is too far from the corner for the curves to read as concentric. In that case use the child's own component radius from the table below.
- **Only applies to nesting.** Siblings, and children that sit far from the parent's corners, use their standard radius.
- **Implement it with tokens**, never with hand-typed numbers:

```css
:root {
  --radius-panel: 16px;
  --radius-control: 12px;
}
.nested {
  --gap: 4px;                                           /* inset from parent edge */
  border-radius: max(0px, calc(var(--radius-outer) - var(--gap)));
}

```

Set `--radius-outer` on the parent to its own radius, and set `--gap` to the same value as the padding the child gets.

**Worked examples in Arcalabs**

| Parent (outer radius)           | Child                              | Gap   | Child radius                                                                                |
| ------------------------------- | ---------------------------------- | ----- | ------------------------------------------------------------------------------------------- |
| Segmented control track (12 px) | Active segment                     | 4 px  | **8 px**                                                                                    |
| Dropdown tabs menu (12 px)      | Menu item (hover / current)        | 4 px  | **8 px**                                                                                    |
| Panel, e.g. Scenes (16 px)      | Selected list row                  | 8 px  | **8 px**                                                                                    |
| Panel (16 px)                   | Meter track inside the Audio panel | 8 px  | **8 px** (a pill-shaped track keeps its full pill radius, since it's a fully rounded shape) |
| Setup card (24 px)              | Full-width button                  | 24 px | gap equals the radius, so use the standard **12 px**                                        |
| Setup card (24 px)              | Step badge circles                 | 40 px | Not nested in the visual sense, so they stay full circles                                   |

### Radii and sizes

| Element                                              | Radius                                       | Notes                                              |
| ---------------------------------------------------- | -------------------------------------------- | -------------------------------------------------- |
| Setup card                                           | 24 px                                        | 1px `border.subtle`                                |
| Panels / cards                                       | 16 px                                        | 1px `border.subtle`, no shadow                     |
| Buttons (primary, outlined)                          | 12 px                                        | ~48–58 px tall; primary is the tallest             |
| Inputs / selects                                     | 10 px                                        | ~48 px tall                                        |
| Segmented control                                    | 12 px outer, **8 px** inner segment (12 − 4) | 4 px track padding                                 |
| Dropdown tabs menu                                   | 12 px outer, **8 px** items (12 − 4)         | 4 px menu padding                                  |
| Selected list row (in a panel)                       | **8 px** (16 − 8)                            | 8 px inset from the panel edge                     |
| Selected nav item (settings nav, not in a container) | 12 px                                        | Not nested, so the standard control radius applies |
| Toggle                                               | pill                                         | ~54 × 30 px                                        |

- **Spacing scale:** 4, 8, 12, 16, 20, 24, 32 px. Choose gaps from this scale so nested radii come out as whole numbers.
- **Elevation:** None. Depth comes from surface tint and 1px borders. Do not add drop shadows.
- **Dividers:** 1px `border.subtle`, only between settings rows and between app regions.

---

## 7. Components

### Primary button

Solid `accent.solid`, white 600 label, 12 px radius, full width of its container. Used once per screen.

- **Go Live** (main window)
- **Done** (settings, top right, compact)
- **Finish setup** (onboarding)

**Disabled state.** Fill `bg.raised`, label `text.secondary`. Never a darker shade of the accent, because that looks like a dim version of an active button. Next to or under the button, a line of `text.secondary` text states the reason, for example "No capture in v0.1". A disabled primary without a visible reason is not allowed.

### Destructive button

Same shape as primary, filled `status.danger`. Replaces *Go Live* in place while streaming: **Stop stream**. Position and size don't change, so the user's cursor doesn't have to move.

### Outlined button (secondary)

Transparent fill, 1px `accent.line` border, `accent.text` label, optional leading icon (see Iconography and decision 0.3).

- **Disabled while live:** border and label drop to ~40% opacity. The button stays visible so layout doesn't shift.

### Segmented control

Dark track (`bg.sunken`, 12 px radius, 4 px padding) with a `bg.raised` active segment (8 px radius, per the nested radius rule). Used for platform choice: **YouTube | Kick | Twitch** (see Platform selector). Inactive label is `text.secondary`. Switch to **Dropdown tabs** when there are more than three options.

### Platform selector

Segmented control with three options, in this order: **YouTube | Kick | Twitch**. These are the only supported platforms for now. Don't add placeholders or "coming soon" entries.

- The same control appears in the Stream panel on the main window and as step 1 of first-run setup.
- Labels are text only. Don't add platform logos or brand colors, since the accent is the only hue in the UI (see Color).
- The selected platform sets the helper text under the key field, the status bar message ("Streaming to <platform>"), and the stream destination.
- The stream key is stored per platform, so switching platforms shows the key saved for that platform (masked), or the empty field if none.
- The selector is disabled while live. Switching platform requires stopping the stream first.
- A fourth platform means switching this control to **Dropdown tabs**.

### Dropdown tabs

A tab switcher collapsed into a single dropdown trigger. It's an approved alternative to the segmented control and to a row of tabs. Use it when options outgrow the available width or when there are more than three.

- **Trigger:** looks like a Select (`bg.sunken`, 1px border, 10 px radius) with the current tab's label in `text.primary` and a `chevron-down` at the right. It does not use the muted `text.secondary` style of a Select, because it names the current view and doesn't hold a value.
- **Menu:** `bg.panel`, 1px `border.subtle`, 12 px radius, 4 px inner padding, anchored under the trigger at the same width or wider. No shadow. Separate it from the page with the border only.
- **Items:** 40 px tall, 8 px radius. Hover is `bg.raised`. The current item uses `bg.selected`, `text.primary` and a trailing `check`.
- **Behavior:** opens on click or Enter/Space, arrow keys move, Enter selects, Esc closes and returns focus to the trigger. Selecting an item switches the view immediately and closes the menu.
- **Where to use it:**
  - Platform choice once a fourth platform is added (for example custom RTMP). Keep the segmented control while there are three or fewer.
  - Settings navigation when the window is narrow (below ~720 px), replacing the left nav.
  - Scene switching in a compact layout.
- **Don't** use it for actions or for a setting value. Those are Selects.

### Text field (stream key)

`bg.sunken`, 1px border, 10 px radius. The key is masked with dots by default, with a show/hide toggle and a paste action. Placeholder text is `text.muted`. After saving, the field shows a masked "saved" state with **Replace** and **Clear**; the key itself is never shown again. In onboarding, a helper line beneath says where to find the key.

The ingest URL is validated (`rtmp://` or `rtmps://`) and has a **Reset to default** action.

### List row (Scenes, Sources, settings nav)

Plain row, `text.secondary` at rest. Rows inside a panel use the nested radius (8 px); the settings nav, which has no container, uses 12 px. **Selected** gets `bg.selected` and `text.primary`. Sources carry a trailing visibility (eye) icon. "Add source" is a row with a trailing `+`.

### Audio meter

Icon, label, then a thin track with an `accent.line` fill. Microphone and Desktop audio each have their own bar. Track color is `bg.raised`. Once real capture exists, the meter reads a dB scale with peak hold.

### Toggle

Pill, `accent.solid` when on with a white knob. Paired with a title and a one-line description (the "Automatic" row).

### Select

Right-aligned, `bg.panel` with border, chevron at right, `text.secondary` value. Disabled while Automatic is on and shown at reduced emphasis.

### Optimization result card

Appears under the Automatic row after optimizing.

- Header: `check` in `accent.line` + "Optimized just now", with **Undo** (neutral outlined button) at the right.
- Rows: `Label ........ old → new`. The old value is struck through in `text.muted`. The new value is `text.primary` at weight 500.
- Order: Encoder, Resolution, Frame rate, Bitrate.
- Below it, a collapsed **Manual settings** row with a trailing chevron.
- **While Optimize is a stub** (v0.1), the header reads "Simulated result" instead of "Optimized just now", so nobody mistakes sample values for real hardware detection.

### Status bar

Left: current state ("Not streaming" / "Streaming to <platform>"). Right, in this order: `CPU`, `RAM`, `fps`, `kbps`, `dropped`, elapsed time. Separated by even spacing instead of dividers.

- `CPU` and `RAM` are the **app's own** process figures. They are the proof of the "lightweight" claim, so they are always shown.
- `fps`, `kbps` and `dropped` show "—" until a stream is live. Never show a stand-in number.
- Every value comes from the Rust side as an emitted event. If a value is not wired up yet, it shows "—".

---

## 8. States

| Element                       | Offline                                                                             | Live                                                                |
| ----------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Header status                 | Grey dot + "Offline"                                                                | Red dot + "Live · 00:12:41"                                         |
| Primary button                | Accent **Go Live**                                                                  | Red **Stop stream**                                                 |
| Primary button, not available | Grey disabled **Go Live** + reason ("No capture in v0.1", or "Add your stream key") | n/a                                                                 |
| Optimize button               | Active (outlined)                                                                   | Disabled (dimmed)                                                   |
| Status bar                    | "Not streaming", app CPU and RAM, "—" for the rest                                  | "Streaming to <platform>", app CPU and RAM, live fps, kbps, dropped |
| Settings → Video              | Automatic on: manual hidden behind result card                                      | Manual fields disabled                                              |

**Live dot:** a solid dot. No glow and no pulse animation. The label text and the button change carry the state.

**Automatic vs Manual**

- **Automatic on:** show the optimization result card and a collapsed "Manual settings" row.
- **Automatic off:** show the editable rows (Resolution, Frame rate, Encoder, Bitrate, Keyframe interval) with the caption "Manual · turn off Automatic to edit".

**Empty, loading and error states** name the cause and the next action.

- Empty: "No sources yet. Add one to see it in the preview."
- Loading: say what is loading ("Detecting your GPU…").
- Error: say what failed and what to do next. No apologies.

**Stubs in v0.1:** any control that does not work yet either carries a visible "Not in v0.1" label or is removed. Nothing looks live and does nothing.

---

## 9. Content and voice

- Sentence case, short verbs, no exclamation marks. No emoji in the UI.
- Buttons name the action: **Go Live**, **Stop stream**, **Finish setup**, **Undo**, **Re-run optimize**.
- Keep an action's name stable across the flow. "Optimize for this PC" is the same label in onboarding, on the main window, and on the website.
- Setup copy is reassuring and concrete: "Three steps."
- Technical summary is a single middle-dot line: `NVENC · 1080p60 · 6,000 kbps`. It appears under the optimize action and in the onboarding "Ready:" line.
- Helper text says where to find things, and names the selected platform ("Find it in your YouTube dashboard under Stream settings."). Confirm the exact wording for each platform before release.
- Keychain wording is the same everywhere. Use "Windows Credential Manager" on Windows, and do not alternate between "OS keychain", "credential locker" and other phrasings.
- No claims that are not measured. No number, duration or comparison appears in copy until it has been measured.

---

## 10. Iconography

**Icon set:** Lucide, pending decision 0.3. All UI icons come from one set. Do not mix in other icon sets or hand-drawn substitutes.

**Package:** `lucide-react` (or the framework equivalent). Import icons individually so unused ones are tree-shaken.

**Defaults**

| Property     | Value                                                                               |
| ------------ | ----------------------------------------------------------------------------------- |
| Size         | 18 px in rows and buttons, 20 px in headers and nav                                 |
| Stroke width | 1.5 (set `strokeWidth={1.5}`; Lucide's default of 2 reads heavy on this dark UI)    |
| Color        | `currentColor`, inheriting the text token of its context (`text.secondary` at rest) |
| Caps / joins | Lucide defaults (round)                                                             |

**Icon map**

| Where it's used                        | Lucide icon                                          |
| -------------------------------------- | ---------------------------------------------------- |
| Microphone audio row                   | `mic`                                                |
| Desktop audio row                      | `volume-2`                                           |
| Source visible (toggle)                | `eye`                                                |
| Source hidden (toggle)                 | `eye-off`                                            |
| Add source                             | `plus`                                               |
| Optimize for this PC / Re-run optimize | `zap` (see decision 0.3; may be replaced or removed) |
| Optimization success                   | `check` (in `accent.line`)                           |
| Manual settings row (collapsed)        | `chevron-right`                                      |
| Select dropdown                        | `chevron-down`                                       |
| Settings entry point                   | `settings`                                           |
| Muted microphone / desktop audio       | `mic-off` / `volume-x`                               |
| Stream key saved                       | `lock`                                               |
| Live indicator (if an icon is needed)  | `radio`                                              |

**Rules**

- Icons support a label. None appear on their own, except the eye and `+` in source rows.
- Icon and label share a color. Only `check` and the audio meters use `accent.line`.
- Keep one size per context. Don't scale icons up for emphasis.
- No sparkle, star, magic, robot or "AI" glyphs anywhere.
- Disabled controls dim their icon along with their label.
- Give icon-only buttons an accessible name (`aria-label="Hide Webcam"`) and a tooltip.

**Logo mark:** The three concentric arcs next to the wordmark are brand artwork, not a Lucide icon. Keep them as a custom SVG and don't swap in `radio` or `wifi`. The native desktop mark follows its Acid Lime accent; the website keeps its separately approved mark and palette.

---

## 11. Accessibility

- Text meets 4.5:1 on every surface it is used on (see the measured table in section 3). No text below 12 px.
- Status is never color-only: live state also changes the label text ("Live · 00:12:41") and the button label ("Stop stream").
- Visible keyboard focus: 2px `accent.line` ring with 2px offset on every interactive element.
- Minimum hit target is 40 px. Primary buttons are 48 px or taller.
- Full keyboard use: every control reachable with Tab, Esc closes modals and menus.
- Respect reduced-motion. The only animations needed are toggle slide, button state change and the result card reveal. Nothing loops.

---

## 12. Platform: Tauri

Arcalabs ships as a **Tauri** desktop app (Rust core, system webview frontend). The design has to hold up across the webviews Tauri uses: WebView2 on Windows, WKWebView on macOS and WebKitGTK on Linux. Windows comes first.

**Window**

- Use a custom title bar (`decorations: false`). The header row (~58 px) is the title bar: wordmark and status on the left or right as designed, and native-style window controls at the far end on Windows and Linux. On macOS keep the system traffic lights, inset to align with the 20 px outer padding.
- Mark the header's empty space as a drag region (`data-tauri-drag-region`). Buttons, the status indicator and any controls inside it must not be drag regions.
- Set a minimum window size of roughly 1100 × 700 so the four-panel control row never collapses. Below ~720 px wide, use Dropdown tabs for navigation instead of the left nav.
- Dark only (decision 0.4). Don't follow the system light theme, and set the window background to `bg.app` so there's no white flash at launch.

**Frontend**

- Bundle the typeface locally (woff2). The app must work offline and must not load fonts, icons or scripts from a CDN.
- Lucide is imported as local modules (see Iconography).
- Avoid CSS that renders differently across webviews. Prefer flexbox and grid, and test `backdrop-filter`, `:has()` and `color-mix()` on WebKitGTK before relying on them. This design needs none of them.
- No native-feeling scroll hacks. Use plain overflow with a thin scrollbar styled to `bg.raised` on `bg.panel`.
- Selects and dropdown menus are custom components, because native `<select>` popups look different on each OS and break the dark theme.

**Backend boundary**

- The Rust side owns encoder detection, the optimize routine, capture and the stream itself. The UI only requests and displays.
- Calls are Tauri commands (e.g. `optimize_for_pc`, `go_live`, `stop_stream`). Live numbers (CPU, RAM, fps, kbps, dropped, elapsed time) arrive as emitted events at about 1 Hz and feed the status bar and header. Don't poll from the UI.
- While a command is pending, the triggering button shows a busy state (label stays, spinner replaces the icon) and is disabled. "Optimize for this PC" may take a few seconds, so show progress and not a frozen button.

**Security and storage**

- Store the stream key in the OS keychain, and never in plain-text config or local storage. The UI only ever receives a masked value.
- Keep the Tauri capability list minimal. Grant only the permissions the app uses (window, event, dialog, shell for FFmpeg or encoder probes if needed).
- Set a strict Content Security Policy in the Tauri config.
- Settings (platform, scene list, video choices, hotkeys) persist in the app config directory. The result card's **Undo** needs the previous values, so keep the last pre-optimize snapshot there too.

**Privacy**

- Arcalabs collects no user data. There is no telemetry, analytics, crash reporting or automatic reporting to GitHub or any other service.
- The only outbound connection is the stream to the chosen platform. Hardware detection for "Optimize for this PC" runs locally and its results never leave the device.
- Don't enable HTTP or shell capabilities the app doesn't need.
- Keep logs local, and keep the stream key and personal paths out of them. A local crash log with a "Copy crash report" action is allowed. It prepares text for the user to copy and never uploads.
- Design implication: no consent banners, "help improve Arcalabs" toggles or feedback prompts. There's nothing to ask permission for.

**Hotkeys**

- Global hotkeys (Go Live / Stop stream, scene switching, mute) are registered from Rust. The Hotkeys settings page records a combination with a focused capture field. It shows a conflict warning if the OS or another app already holds it. In v0.1 the page is labeled "Not in v0.1".

---

## 13. Do and don't

**Do**

- Keep one solid accent action on screen.
- Swap *Go Live* and *Stop stream* in the same spot.
- Show before/after values for any automatic change, with Undo.
- Keep advanced controls reachable but collapsed.
- Use Dropdown tabs when a tab row would no longer fit.
- Bundle fonts and icons locally. The app is Tauri and must work offline.
- Set nested corners with inner radius = outer radius − gap.
- Label stubs, simulations and placeholders where the user can see them.
- Explain every disabled primary action with a visible reason.

**Don't**

- Add shadows, gradients, glow or glassmorphism to panels.
- Add pulsing, looping or decorative animation. The live dot is solid.
- Use icons from any set other than the chosen one.
- Use native `<select>` or OS-styled menus. Use the custom components.
- Give a nested element the same radius as its parent.
- Store the stream key in plain text or local storage.
- Add telemetry, analytics, crash reporting or any automatic reporting to GitHub or elsewhere.
- Use red for anything except live and stop.
- Put raw encoder settings on the main window.
- Show invented numbers (CPU, fps, bitrate, benchmarks). Show "—" or a real value.
- Add decorative badges, eyebrows, emoji, "AI" icons, left-edge color stripes or arrows on buttons.
- Add more panels to the control row without removing one. Four is the limit for this layout.

---

## 14. Website (arcalabs.vercel.app)

The site explains the app, is honest about its status, and shares the app's tokens: one accent (decision 0.1), the same type, the same radius scale, and the same dark surfaces. It is not a second brand.

### 14.1 Structure

1. **Header:** wordmark and two links (the demo, GitHub). Same link color for both.
2. **Hero:** headline, one-sentence description, one primary action, and a **real screenshot of the desktop app** labeled "v0.1 prototype. No Go Live yet." No logo art, no triangles or squares, no eyebrow line above the headline.
3. **Demo (Before / After):** the one memorable moment on the page. See 14.2.
4. **First-run steps:** three numbered steps (a real sequence), full width, with the real setup screenshot beside or below.
5. **Footer:** project status line, GitHub link, and the analytics disclosure.

Only the sections above. No testimonials, no logo bar, no pricing, no "trusted by", no feature-card grid, no numbered all-caps feature labels (numbers are only for the three setup steps).

### 14.2 Demo

- Show the **Before / After table only**. Remove the YAML "read-only preview" block (a fake terminal, and it contradicts principle 6).
- The **Optimize for this PC** button lives inside or directly under the After column, next to the result it produces.
- On click, the After values fill in, changed values are highlighted (`x264 → NVENC`), and **Undo** appears. This is the single orchestrated motion on the page. It responds to a click and nothing else animates on its own.
- Until the demo reads real hardware, label it **"Simulated example"** next to the button. The result card header reads "Simulated result".

### 14.3 Copy

- Use the one product name from decision 0.5 everywhere.
- Primary action is the demo while no build exists. After the first release it becomes "Download for Windows", with the version and file size beside it.
- Say plainly that it is a prototype. Move the "in development" status near the hero, out of the footer.
- No claim without a measurement. "Lightweight" is backed by a measured idle RAM number and the method, or it stays out of the headline. Describe the architecture instead: "Built with Rust and Tauri, so there is no bundled browser engine."
- No time promises ("about a minute") until timed.
- Keep the privacy statement. The app collects no data. The site uses Vercel Web Analytics, and says so.

### 14.4 Layout and type

- One container (max-width about 960 px). Hero, demo, steps and footer share the same left edge.
- Spacing from the 4 / 8 / 12 / 16 / 20 / 24 / 32 scale, with larger steps between sections.
- Body text is at least 16 px, labels at least 12 px, all at 4.5:1 or better.
- Same radius scale as the app. No shadows, no gradients, no glow.
- Sentence case. No all-caps labels. No monospace except literal code.

### 14.5 Performance and page basics

None of these have been measured yet. Report LCP, INP and CLS only after a real measurement, and until then say "not measured".

- **Prerender the page** so the content is in the initial HTML. A client-rendered SPA shows crawlers and link previews almost nothing.
- **Hero image** is an `<img>` in the HTML with `fetchpriority="high"`, not lazy, not a CSS background.
- **Every** `<img>` **and embed has width and height or** `aspect-ratio`**.** Nothing shifts the layout while loading.
- **Scripts:** analytics loads with `defer` or `async`. No render-blocking third-party scripts.
- **Fonts:** self-hosted, `font-display: swap`, with a real fallback stack.
- **Head:** `<!doctype html>`, `<meta charset="utf-8">`, viewport tag, title, description, canonical, and Open Graph and Twitter card tags with a 1200 × 630 image. Add `SoftwareApplication` structured data only once a download exists.
- **Links:** external links that open a new tab use `rel="noopener noreferrer"`. HTTPS only.
- **Security:** no secrets in client code. Set a strict CSP in Vercel headers.
- **Quality floor:** responsive down to mobile, visible focus, `prefers-reduced-motion` respected, keyboard-only use works.
- **Robots and sitemap:** `robots.txt` and `sitemap.xml` in `public/`.

### 14.6 Copy pattern

Each statement on the site has two parts: the problem in the user's own terms, then what the app does about it. Two sentences, concrete nouns, no adjectives.

- Example (write in the present tense only for what ships; use "planned" for stubs): "Picking an encoder, resolution and bitrate usually means reading forum threads for your GPU. Arcalabs detects your hardware, picks them, shows what changed, and lets you undo it."
- The headline says what the app does. It does not claim more than the app does (no scale, speed or "hundreds of" claims without a measurement).
- Social proof is real or absent. No invented names, roles, quotes, star counts or user counts. A prototype with no users shows none.

### 14.7 After the first release

- **Download block:** one line per supported OS and architecture, linking to the release. Windows x64 only until another build exists. Show the version and file size.
- **Footer:** license (GPL-3.0), link to the releases page, link to a **Privacy** page, GitHub, and the analytics disclosure. The privacy page states what the app collects (nothing), what the site collects (page analytics), and that the stream key never leaves the device. A terms page is only added if the site ever asks for sign-ups or payment.
- **Analytics:** keep the lightest option that answers a real question. The app has none, so the site should not load a tag manager.
- **Content in the HTML:** the page text, hero image and download links are in the initial HTML, so they are readable without running scripts.

---

## 15. Implementation status (v0.1)

The screenshots show the app differs from this document in these places. Decide for each whether the app or the document should change.

| Area                | This document says                                    | The app currently does                                           |
| ------------------- | ----------------------------------------------------- | ---------------------------------------------------------------- |
| Settings container  | Full page with a **Done** button                      | Modal with a **Back** button                                     |
| Settings nav        | Stream, Video, Audio, Hotkeys, Advanced               | General, Stream, Video, Audio, Hotkeys                           |
| Go Live             | Inside the Stream panel, with the platform selector   | Standalone button in the control row                             |
| Platform selector   | Stream panel on the main window and onboarding step 1 | Inside Settings → Stream                                         |
| Optimize            | Automatic row in Video settings and in onboarding     | Button at the bottom of the settings sidebar, labeled "Optimize" |
| Go Live disabled    | Grey, with a visible reason                           | Dim accent fill, no reason                                       |
| Idle status bar     | App CPU and RAM, "—" for the rest                     | "CPU 4% · 60 fps · 0 kbps · 0 drop"                              |
| Preview empty state | "No capture yet" + source name                        | "Screen 1 · 1920×1080" only, small                               |
| Preview frame       | One frame                                             | Two nested frames                                                |
| Naming              | One spelling (decision 0.5)                           | "Arcalabs" and "ArcaLabs" both appear                            |

---

## 16. Changes from the previous draft

- Renamed `design.md` to `DESIGN.md` so tools that match the exact name find it.
- Added section 0 (open decisions), section 14 (website) and section 15 (implementation status).
- `text.muted` changed from `#6E6F78` to `#8E8F98` after measuring contrast. Added the measured contrast table.
- Added a disabled primary button state with a visible reason, and a stub state for the optimize result.
- Status bar now shows the app's own RAM, and shows "—" for stream metrics until live. No stand-in numbers.
- Live dot explicitly has no glow and no pulse.
- Removed the "about a minute" claim until it is timed.
- Added a minimum text size, stream-key field behavior, one wording for the keychain, and don'ts for decorative badges, eyebrows, emoji, AI icons, left stripes and arrows.
- Added section 1.1 (identity and direction), a `tokens.css` companion, and proposed reasons for the open decisions.
- Added website copy pattern (14.6) and post-release rules (14.7), and a rule against numbered all-caps feature labels.
- Removed the "Lucide required" wording. It is now "pending decision 0.3" because the set must be a written choice.
