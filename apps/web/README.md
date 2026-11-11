# Arca Labs — marketing site

Static landing page with a client-side **Optimize for this PC** demo. No analytics, no API routes, no forms that send data off-device.

## Local

```bash
pnpm install
pnpm dev
```

## Deploy on Vercel

1. [Import](https://vercel.com/new) the `cxernlol/ArcaLabs` repository.
2. Set **Root Directory** to `apps/web` (required — the repo is a monorepo).
3. Vercel should detect **Vite**; `vercel.json` sets install/build/output.
4. Deploy. Production track is usually `main` after merge; branch deploys get preview URLs.

| Setting | Value |
|--------|--------|
| Root Directory | `apps/web` |
| Install Command | `pnpm install` |
| Build Command | `pnpm build` |
| Output Directory | `dist` |

**Privacy:** Do not add Vercel Analytics or Speed Insights unless you add an explicit opt-in spec — matches `AGENTS.md`.

After the first deploy, open the preview URL and run through **Optimize → Undo** to confirm the demo and that `/logo.png` loads.
