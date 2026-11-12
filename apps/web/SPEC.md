# SPEC: Arca Labs v0.1 website refresh

## 1. Goal & User Story

People evaluating Arca Labs need an accurate view of what the Windows v0.1 prototype can do, what is planned through v1.0, and how the product and website handle data. The site presents that information with accessible legal pages and search-friendly routes without implying that v0.1 can stream live.

## 2. Inputs & Outputs

**Inputs**

- Existing Vite/React marketing site, workflow diagram, logo, and supplied desktop-app screenshots.
- Approved palette: Obsidian Black `#0B0B0D`, Icy White `#F2F4F8`, Radiant Violet `#8B5CF6`, Radiant Violet Hover `#A78BFA`, Surface Dark `#151519`, Border Dark `#29292F`, and Text Muted `#A1A1AA`.
- Product facts: v0.1 is a Windows prototype for YouTube, Kick, and Twitch that cannot go live. Desktop settings remain local, the stream key is stored in the OS keychain, and scrubbed crash logs are not automatically uploaded.
- Roadmap:
  - Q4 2026, v0.1: local prototype, settings, OS-keychain stream key, local scrubbed crash log, no telemetry.
  - Q1 2027, v0.2: screen/window capture, WASAPI audio capture, and live preview.
  - Q2 2027, v0.3: RTMP publishing, platform ingest presets, start/stop/status, and the first live game.
  - Q3 2027, v0.4: automatic encoder, resolution, frame-rate, and bitrate selection; manual overrides; on-device GPU detection.
  - Q4 2027, v1.0: Windows installer, auto-update, global hotkeys, and published RAM/CPU benchmarks.
- Motion request: keep the one-time Framer Motion reveal for the homepage hero and workflow sections. Animate roadmap milestones in document order with a subtle fade (opacity 0.85 to 1) and a 24px vertical shift to 0px over a 10%-of-viewport-height scroll interval positioned above the fixed consent popup when it is visible. Each reveal remains distinct and reverses naturally when scrolling back up. Keep text readable throughout, do not pin content or delay access, and show content immediately without motion under `prefers-reduced-motion: reduce`.
- No privacy contact or governing-law jurisdiction has been supplied. Do not invent either; owner/legal review is required before production publication.

**Outputs**

- Homepage hero with the supplied v0.1 screenshot beside the headline. Use the supplied logo as a blurred backdrop behind a translucent preview surface. Crop the screenshot to exclude simulated CPU/FPS values; identify the preview as a prototype.
- A Q4 2026–Q4 2027 roadmap and the existing workflow diagram, preserving its reduced-motion behavior.
- One-time Framer Motion reveals on the homepage hero and workflow heading/diagram. Each roadmap milestone gets a restrained opacity-and-vertical-shift reveal over a 10%-of-viewport-height scroll interval above the fixed consent popup when visible, so milestones reveal in document order without adjacent transitions overlapping at supported viewport sizes. The motion reverses as scroll direction reverses. Keep milestone text visible at a minimum opacity of 0.85; reduced-motion users see all content immediately. The workflow diagram keeps its existing behavior.
- Static Privacy, Terms, and custom 404 pages with shared visual styling. The 404 page links to the homepage, roadmap, and legal pages. The footer links to Privacy and Terms and displays `© 2026 Arca Labs`.
- Privacy content distinguishes hosted-site Vercel Web Analytics from the desktop app. The consent prompt is a compact, non-modal popup fixed near the viewport bottom, respects mobile safe-area insets, and does not shift page content. The analytics script loads only after explicit acceptance; Reject keeps analytics disabled. Store the choice locally, provide a footer control to change it, and default to disabled if local storage is unavailable. The consent choice is not stored in a cookie.
- Unique page titles, meta descriptions, canonical URLs, one clear H1 per page, descriptive alt text, a sitemap, an indexable robots.txt, and the existing favicon plus a mobile touch icon. The 404 page is not indexed or included in the sitemap.
- Clean public paths for Privacy and Terms. Preserve the existing `/flow.html` behavior. External links open in a new tab with appropriate `rel` attributes.
- Optimize supplied imagery without making text in the app screenshots difficult to read. Keep fonts and assets local.
- **Network calls:** hosted-site Vercel Web Analytics page-view requests occur only after explicit consent; this change adds no other network calls. The desktop app adds none.

## 3. Acceptance Criteria

- [ ] The homepage visibly uses the approved tokens in the 60/30/10 roles: dark ground, light content/type, and limited violet actions or emphasis.
- [ ] The supplied prototype screenshot appears beside the hero copy on wide screens and stacks cleanly on narrow screens; its displayed crop contains no simulated performance figures.
- [ ] The page labels v0.1 as a prototype that cannot go live and displays all five approved roadmap milestones with the correct quarter, version, and scope.
- [ ] Each roadmap milestone moves from opacity 0.85 and a 24px vertical offset to full opacity and its resting position over a 10%-of-viewport-height scroll interval; transitions happen one at a time in document order and reverse with scroll progress in either direction.
- [ ] When the consent popup is visible, it does not cover the roadmap milestone reveal interval at the supported mobile and desktop viewport sizes.
- [ ] Roadmap text remains visible and readable throughout the motion, with no pinning, delayed access, or animation of individual feature bullets.
- [ ] The homepage hero and workflow heading/diagram retain their one-time reveals; reduced-motion mode disables all reveals and displays content immediately.
- [ ] The existing workflow diagram remains available and usable with reduced motion enabled.
- [ ] Privacy, Terms, and custom 404 routes load directly. Unknown routes return the custom 404 response with working navigation links.
- [ ] The footer displays Privacy, Terms, and `© 2026 Arca Labs`; the supplied logo is used for browser and mobile touch icons.
- [ ] Each indexable page has a unique title, description, canonical URL, and exactly one H1. The 404 response is not indexed.
- [ ] `/robots.txt` permits indexing and references `/sitemap.xml`; the sitemap contains the homepage and legal pages, not the 404 page.
- [ ] Vercel Analytics sends no page-view request before consent or after rejection. Accepting enables it; the footer privacy control lets a visitor change the choice. If local storage is unavailable, analytics stays disabled.
- [ ] The consent prompt remains compact and fixed near the viewport bottom on desktop and mobile, does not block page interaction or shift content, and the footer control reopens it without scrolling the page.
- [ ] The privacy page accurately describes hosted-site analytics and local desktop-app handling without claiming that the desktop app sends data or that the site is legally compliant in every jurisdiction.
- [ ] No placeholder copy, dummy posts, default stock images, invented contact details, or invented governing-law terms remain.
- [ ] Internal navigation works; external links open in a new tab. Forms, payments, CRM, booking, and newsletter integrations are not added.
- [ ] `pnpm --dir apps/web build` and `pnpm --dir apps/web lint` pass. Verify the page at 360px, 768px, and 1440px widths in the available browser; report any browsers or physical devices that could not be tested.
- [ ] Production HTTPS, CDN/cache behavior, and live performance are reported only after checking the deployed site; local preview results are not presented as production verification.

## 4. Edge Cases & Error States

| Situation | Behavior |
|---|---|
| Visitor has not chosen an analytics preference | Show the compact bottom popup with Accept and Reject controls; do not load analytics. |
| Visitor rejects analytics | Keep analytics disabled and preserve the choice locally. |
| Visitor changes their choice | The footer privacy control reopens the choice; apply the new choice without adding another tracker. |
| Local storage is unavailable | Keep analytics disabled and leave the choice controls available. |
| Direct request to a legal page | Serve that page and its metadata without requiring prior homepage navigation. |
| Unknown route | Return the branded 404 page with links to useful destinations and a not-found status. |
| Reduced motion is enabled | Show all homepage content immediately without reveal motion and keep the workflow diagram usable without scroll-driven camera movement. |
| Consent popup is visible while the roadmap is scrolled | Keep milestone reveals above the popup so their motion remains visible. |
| Legal contact or jurisdiction remains unknown | Omit both rather than fabricate details; require owner/legal review before production publication. |
| Forms, payments, CRM, or booking are absent | Treat those checklist items as not applicable; do not add integrations. |
| Production host or device/browser is unavailable | State that its SSL, cache/CDN, performance, or device/browser check was not run. |

## 5. Out of Scope

- Changes to the desktop app, its data handling, or its release schedule.
- New analytics, advertising, external fonts, CDN assets, or other third-party integrations.
- Forms, payments, CRM, booking, newsletter capture, or social feeds.
- Inventing legal contact details, a governing-law jurisdiction, or claims of legal compliance.
- Deployment, DNS, hosting-plan, or Vercel project-setting changes.
- Changing existing public URLs beyond adding the approved pages; no redirect migration is required.
- Scroll-linked parallax, pinning, animation of legal pages, or animation of individual roadmap bullets.
