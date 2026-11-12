import { useEffect, useRef } from "react";
import "./App.css";

const FOCUS_STEPS = [
  "streamer",
  "platform",
  "key",
  "optimize",
  "review",
  "golive",
  "",
] as const;

const GITHUB = "https://github.com/cxernlol/ArcaLabs";

function GitHubMark() {
  return (
    <svg width="16" height="16" viewBox="0 0 16 16" aria-hidden="true">
      <path
        fill="currentColor"
        d="M8 0C3.58 0 0 3.58 0 8a8 8 0 0 0 5.47 7.59c.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82A7.7 7.7 0 0 1 8 3.87a7.7 7.7 0 0 1 2 .27c1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.01 8.01 0 0 0 16 8c0-4.42-3.58-8-8-8Z"
      />
    </svg>
  );
}

export default function App() {
  const frameRef = useRef<HTMLIFrameElement>(null);

  useEffect(() => {
    const section = document.getElementById("workflow");
    const frame = frameRef.current;
    if (!section || !frame) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;

    let lastIndex = -1;
    let scheduledFrame: number | null = null;
    const post = (progress: number) => {
      const scaledProgress = progress * FOCUS_STEPS.length;
      const index = Math.min(
        FOCUS_STEPS.length - 1,
        Math.floor(scaledProgress),
      );
      frame.contentWindow?.postMessage(
        {
          type: "arca-scroll",
          fromId: FOCUS_STEPS[index - 1] ?? "",
          id: FOCUS_STEPS[index],
          progress: scaledProgress - index,
        },
        window.location.origin,
      );
    };
    const send = () => {
      const rect = section.getBoundingClientRect();
      const stuck = rect.top <= 1 && rect.bottom >= window.innerHeight - 1;
      if (!stuck) {
        if (lastIndex !== -1) {
          post(1);
          lastIndex = -1;
        }
        return;
      }
      const total = section.offsetHeight - window.innerHeight;
      const scrolled = Math.min(Math.max(-rect.top, 0), Math.max(total, 0));
      const progress = total > 0 ? scrolled / total : 0;
      lastIndex = Math.min(
        FOCUS_STEPS.length - 1,
        Math.floor(progress * FOCUS_STEPS.length),
      );
      post(progress);
    };
    const scheduleSend = () => {
      if (scheduledFrame !== null) return;
      scheduledFrame = window.requestAnimationFrame(() => {
        scheduledFrame = null;
        send();
      });
    };

    send();
    frame.addEventListener("load", send);
    window.addEventListener("scroll", scheduleSend, { passive: true });
    window.addEventListener("resize", scheduleSend);
    return () => {
      if (scheduledFrame !== null) {
        window.cancelAnimationFrame(scheduledFrame);
      }
      frame.removeEventListener("load", send);
      window.removeEventListener("scroll", scheduleSend);
      window.removeEventListener("resize", scheduleSend);
    };
  }, []);

  return (
    <div className="page">
      <header className="site-header">
        <a className="brand" href="/">
          <img src="/logo.png" width={28} height={28} alt="" fetchPriority="high" />
          <span className="brand-name">Arca Labs</span>
        </a>
        <nav className="site-nav" aria-label="Page">
          <a href="#workflow">Workflow</a>
        </nav>
        <a className="header-github" href={GITHUB} rel="noopener noreferrer">
          <GitHubMark />
          GitHub
        </a>
      </header>

      <main id="main-content">
        <section className="hero">
          <h1>The stream key stays on your PC.</h1>
          <p className="lede">
            Windows prototype for YouTube, Kick, and Twitch. Encoder settings
            come from your hardware. This build cannot go live.
          </p>
          <div className="hero-actions">
            <a className="btn btn-primary" href={GITHUB} rel="noopener noreferrer">
              GitHub
            </a>
          </div>
        </section>
        <section id="workflow" className="workflow" aria-label="Workflow">
          <div className="workflow-sticky">
            <iframe
              ref={frameRef}
              title="Arca Labs workflow"
              scrolling="no"
              src="/diagram.html?embed=1&theme=dark"
            />
          </div>
        </section>
      </main>

      <footer className="site-footer">
        <p>
          v0.1 prototype. GPL-3.0. This page uses Vercel Web Analytics. The
          desktop app does not.
        </p>
      </footer>
    </div>
  );
}
