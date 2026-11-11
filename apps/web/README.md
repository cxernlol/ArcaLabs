# Arca Labs — marketing site

Static landing page with a client-side **Optimize for this PC** demo. No API routes and no forms that send data off-device.

**Analytics:** This site uses [Vercel Web Analytics](https://vercel.com/docs/analytics) (`@vercel/analytics/react`) for aggregate page views on the hosted deployment only. The **Arcalabs desktop app does not use analytics or telemetry.**

**Design:** [`DESIGN.md`](DESIGN.md) (direction). UI changes follow [antislop](https://github.com/miqdadbadjuber/anti-slop/tree/main/skills/antislop) as the filter.

## Local

```bash
pnpm install
pnpm dev
```

`pnpm dev` does not load production analytics unless you enable Web Analytics for the project on Vercel.

## Deploy on Vercel

1. [Import](https://vercel.com/new) the `cxernlol/ArcaLabs` repository.
2. Set **Root Directory** to `apps/web` (required — the repo is a monorepo).
3. Vercel should detect **Vite**; `vercel.json` sets install/build/output.
4. Enable **Web Analytics** for the project in the Vercel dashboard if you want metrics in production.

| Setting | Value |
|--------|--------|
| Root Directory | `apps/web` |
| Install Command | `pnpm install` |
| Build Command | `pnpm build` |
| Output Directory | `dist` |

After deploy, run **Optimize → Undo** on the live URL and confirm `/logo.png` loads.
