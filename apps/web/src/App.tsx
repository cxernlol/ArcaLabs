import { useEffect, useRef, type ReactNode } from "react";
import {
  motion,
  useReducedMotion,
  useScroll,
  useTransform,
} from "framer-motion";
import AnalyticsConsent from "./AnalyticsConsent.tsx";
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

const ROADMAP = [
  {
    quarter: "Q4 2026",
    dateTime: "2026-10",
    version: "v0.1",
    title: "Local prototype",
    status: "Current",
    features: [
      "Settings stay on your PC",
      "Stream key stored in the OS keychain",
      "Scrubbed crash logs stay local",
      "No desktop telemetry",
    ],
  },
  {
    quarter: "Q1 2027",
    dateTime: "2027-01",
    version: "v0.2",
    title: "Capture and preview",
    status: "Planned",
    features: [
      "Screen and window capture",
      "WASAPI audio capture",
      "Live preview",
    ],
  },
  {
    quarter: "Q2 2027",
    dateTime: "2027-04",
    version: "v0.3",
    title: "First live game",
    status: "Planned",
    features: [
      "RTMP publishing and ingest presets",
      "Start, stop, and stream status",
      "First game streamed live",
    ],
  },
  {
    quarter: "Q3 2027",
    dateTime: "2027-07",
    version: "v0.4",
    title: "Automatic stream setup",
    status: "Planned",
    features: [
      "On-device GPU detection",
      "Suggested encoder, resolution, frame rate, and bitrate",
      "Manual overrides",
    ],
  },
  {
    quarter: "Q4 2027",
    dateTime: "2027-10",
    version: "v1.0",
    title: "First stable release",
    status: "Planned",
    features: [
      "Windows installer and auto-update",
      "Global hotkeys",
      "Published RAM and CPU benchmarks",
    ],
  },
] as const;

const REVEAL_VARIANTS = {
  hidden: { opacity: 0.92, y: 12 },
  visible: { opacity: 1, y: 0 },
};

function AnimatedRoadmapItem({ children }: { children: ReactNode }) {
  const ref = useRef<HTMLLIElement>(null);
  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start 55%", "start 45%"],
  });
  const opacity = useTransform(scrollYProgress, [0, 1], [0.85, 1]);
  const y = useTransform(scrollYProgress, [0, 1], [24, 0]);

  return (
    <motion.li ref={ref} className="roadmap-item" style={{ opacity, y }}>
      {children}
    </motion.li>
  );
}

function RoadmapMilestone({
  milestone,
  revealEnabled,
}: {
  milestone: (typeof ROADMAP)[number];
  revealEnabled: boolean;
}) {
  const content = (
    <>
      <time className="roadmap-date" dateTime={milestone.dateTime}>
        {milestone.quarter}
      </time>
      <div className="roadmap-content">
        <div className="roadmap-title-row">
          <h3>{milestone.title}</h3>
          <span className="version-tag">{milestone.version}</span>
          <span className="milestone-tag">{milestone.status}</span>
        </div>
        <ul className="roadmap-features">
          {milestone.features.map((feature) => (
            <li key={feature}>{feature}</li>
          ))}
        </ul>
      </div>
    </>
  );

  return revealEnabled ? (
    <AnimatedRoadmapItem>{content}</AnimatedRoadmapItem>
  ) : (
    <li className="roadmap-item">{content}</li>
  );
}

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

function SiteHeader() {
  return (
    <header className="site-header">
      <a className="brand" href="/" aria-label="Arca Labs home">
        <img
          src="/logo.png"
          width={28}
          height={28}
          alt=""
          fetchPriority="high"
        />
        <span className="brand-name">Arca Labs</span>
      </a>
      <nav className="site-nav" aria-label="Page">
        <a href="#roadmap">Roadmap</a>
        <a href="#workflow">Workflow</a>
      </nav>
      <a
        className="header-github"
        href={GITHUB}
        target="_blank"
        rel="noopener noreferrer"
      >
        <GitHubMark />
        GitHub
      </a>
    </header>
  );
}

function SiteFooter() {
  return (
    <footer className="site-footer">
      <p className="footer-copy">© 2026 Arca Labs · GPL-3.0</p>
      <nav className="footer-nav" aria-label="Legal and privacy">
        <a href="/privacy/">Privacy</a>
        <a href="/terms/">Terms</a>
        <button
          className="privacy-settings-button"
          type="button"
          data-open-analytics-settings
          aria-controls="analytics-consent"
          aria-expanded="false"
        >
          Privacy settings
        </button>
      </nav>
    </footer>
  );
}

export default function App() {
  const frameRef = useRef<HTMLIFrameElement>(null);
  const revealEnabled = useReducedMotion() === false;

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
      <SiteHeader />
      <main id="main-content">
        <motion.section
          className="hero"
          aria-labelledby="hero-title"
          initial={revealEnabled ? "hidden" : false}
          whileInView={revealEnabled ? "visible" : undefined}
          viewport={{ once: true, amount: 0.16 }}
          variants={REVEAL_VARIANTS}
          transition={{ duration: 0.45, ease: "easeOut" }}
        >
          <div className="hero-copy">
            <p className="hero-kicker">Windows prototype · v0.1</p>
            <h1 id="hero-title">The stream key stays on your PC.</h1>
            <p className="lede">
              A streaming setup for YouTube, Kick, and Twitch, built around your
              hardware. This prototype keeps settings local and cannot go live
              yet.
            </p>
            <div className="hero-actions">
              <a
                className="btn btn-primary"
                href={GITHUB}
                target="_blank"
                rel="noopener noreferrer"
              >
                View on GitHub
              </a>
              <a className="btn btn-secondary" href="#roadmap">
                See the roadmap
              </a>
            </div>
          </div>

          <figure className="preview-stage">
            <img className="preview-mark" src="/logo.png" alt="" />
            <div className="preview-glass">
              <div className="preview-topline">
                <strong>Arca Labs · v0.1</strong>
                <span className="preview-status">Offline prototype</span>
              </div>
              <div className="preview-image-frame">
                <img
                  className="preview-image"
                  src="/prototype-home.png"
                  width={1172}
                  height={768}
                  fetchPriority="high"
                  alt="Arca Labs Windows prototype showing stream setup controls and an offline preview; live streaming is not available in v0.1."
                />
              </div>
              <figcaption className="preview-caption">
                Product preview. The prototype cannot start a live stream.
              </figcaption>
            </div>
          </figure>
        </motion.section>
        <section
          id="roadmap"
          className="roadmap"
          aria-labelledby="roadmap-title"
        >
          <div className="roadmap-inner">
            <div className="roadmap-heading">
              <p className="section-kicker">The plan</p>
              <h2 id="roadmap-title">From local prototype to first release.</h2>
              <p className="roadmap-intro">
                A planned path from v0.1 to v1.0. Dates and features may change
                as the prototype is tested.
              </p>
            </div>
            <ol className="roadmap-list">
              {ROADMAP.map((milestone) => (
                <RoadmapMilestone
                  key={milestone.version}
                  milestone={milestone}
                  revealEnabled={revealEnabled}
                />
              ))}
            </ol>
          </div>
        </section>
        <motion.section
          className="workflow-heading"
          aria-labelledby="workflow-title"
          initial={revealEnabled ? "hidden" : false}
          whileInView={revealEnabled ? "visible" : undefined}
          viewport={{ once: true, amount: 0.16 }}
          variants={REVEAL_VARIANTS}
          transition={{ duration: 0.45, ease: "easeOut" }}
        >
          <p className="section-kicker">How it works</p>
          <h2 id="workflow-title">A clear path from setup to stream.</h2>
          <p>
            Choose a platform, protect your stream key, review the setup, and
            keep control before anything goes live.
          </p>
        </motion.section>
        <section
          id="workflow"
          className="workflow"
          aria-label="Arca Labs streaming workflow"
        >
          <div className="workflow-sticky">
            <motion.iframe
              ref={frameRef}
              title="Arca Labs streaming workflow diagram"
              scrolling="no"
              src="/diagram.html?embed=1&theme=dark"
              initial={revealEnabled ? "hidden" : false}
              whileInView={revealEnabled ? "visible" : undefined}
              viewport={{ once: true, amount: 0.16 }}
              variants={REVEAL_VARIANTS}
              transition={{ duration: 0.45, ease: "easeOut" }}
            />
          </div>
        </section>
      </main>
      <SiteFooter />
      <AnalyticsConsent />
    </div>
  );
}
