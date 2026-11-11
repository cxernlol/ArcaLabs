import { OptimizeDemo } from "./components/OptimizeDemo";
import "./App.css";

const GITHUB = "https://github.com/cxernlol/ArcaLabs";

export default function App() {
  return (
    <div className="page">
      <header className="site-header">
        <a className="brand" href="/">
          <img
            src="/logo.png"
            width={44}
            height={44}
            alt="Arca Labs"
            fetchPriority="high"
          />
          <span className="brand-name">Arca Labs</span>
        </a>
        <nav className="site-nav" aria-label="Page">
          <a href="#optimize-demo">Try optimize</a>
          <a href={GITHUB} rel="noopener noreferrer">
            GitHub
          </a>
        </nav>
      </header>

      <main id="main-content">
        <section className="hero" aria-labelledby="hero-title">
          <div className="hero-copy">
            <p className="hero-kicker">Arca Labs · Windows streaming</p>
            <h1 id="hero-title">
              Go live without learning encoders
            </h1>
            <p className="hero-lede">
              Arcalabs is a Windows streaming app for YouTube, Kick, and Twitch.
              It picks resolution, frame rate, and bitrate for your PC. You review
              the changes, undo if you want, then hit Go Live.
            </p>
            <p className="hero-note">
              Built with Rust and Tauri on WebView2, so idle RAM stays with capture
              and encode, not a second Chromium install. The desktop app still
              collects no telemetry.
            </p>
            <a className="hero-cta" href="#optimize-demo">
              Try the optimize flow
            </a>
          </div>
          <div className="hero-visual" aria-hidden>
            <div className="hero-wedge" />
            <img
              className="hero-mark"
              src="/logo.png"
              width={280}
              height={280}
              alt=""
              fetchPriority="high"
            />
          </div>
        </section>

        <OptimizeDemo />

        <section className="first-run" aria-labelledby="first-run-title">
          <h2 id="first-run-title">First time you open the app</h2>
          <p className="first-run-intro">
            The onboarding in the desktop build matches what you see below: about
            a minute, three concrete steps, then you are ready to stream.
          </p>
          <ul className="first-run-list">
            <li>
              <span className="first-run-label">Where you stream</span>
              YouTube, Kick, or Twitch. One platform at a time while live.
            </li>
            <li>
              <span className="first-run-label">Stream key</span>
              Paste once. Stored in the Windows credential locker, not a config file.
            </li>
            <li>
              <span className="first-run-label">Optimize</span>
              Run it on your hardware, read the diff, finish setup, go live.
            </li>
          </ul>
        </section>
      </main>

      <footer className="site-footer">
        <p className="footer-copy">
          <span>Windows-first desktop app in development.</span>
          <span className="footer-privacy">
            This page uses Vercel Web Analytics. The Arcalabs app does not.
          </span>
        </p>
        <a href={GITHUB} rel="noopener noreferrer">
          ArcaLabs on GitHub
        </a>
      </footer>
    </div>
  );
}
