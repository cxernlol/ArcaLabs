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
            width={36}
            height={36}
            alt=""
            fetchPriority="high"
          />
          <span className="brand-name">Arca Labs</span>
        </a>
        <nav className="site-nav" aria-label="Page">
          <a href="#optimize-demo">Optimize</a>
          <a href={GITHUB} rel="noopener noreferrer">
            GitHub
          </a>
        </nav>
      </header>

      <p className="proto-banner" role="status">
        Prototype — no Go Live yet. Run the desktop shell from source; this page
        shows the optimize flow only.
      </p>

      <main id="main-content">
        <section className="hero-lead" aria-labelledby="hero-title">
          <h1 id="hero-title">Arca Labs</h1>
          <p className="hero-lede">
            Windows streaming for YouTube, Kick, and Twitch. One tap picks
            encoder settings for your PC — you review the diff, then go live
            when the app can.
          </p>
        </section>

        <OptimizeDemo />

        <section className="first-run" aria-labelledby="first-run-title">
          <h2 id="first-run-title">First time you open the app</h2>
          <ol className="first-run-list">
            <li>
              <span className="first-run-step">1</span>
              <div>
                <span className="first-run-label">Where you stream</span>
                <p>YouTube, Kick, or Twitch. One platform at a time while live.</p>
              </div>
            </li>
            <li>
              <span className="first-run-step">2</span>
              <div>
                <span className="first-run-label">Stream key</span>
                <p>
                  Paste once. Stored in the Windows credential locker, not a
                  config file.
                </p>
              </div>
            </li>
            <li>
              <span className="first-run-step">3</span>
              <div>
                <span className="first-run-label">Optimize</span>
                <p>
                  Run it on your hardware, read the diff, finish setup. Go Live
                  lands in a later build.
                </p>
              </div>
            </li>
          </ol>
        </section>

        <section className="decisions" aria-labelledby="decisions-title">
          <h2 id="decisions-title">Decisions we actually made</h2>
          <ul className="decisions-list">
            <li>
              Brand accent is logo teal only — not a second violet for the site.
            </li>
            <li>
              The optimize Before/After is the hero; no decorative logo art.
            </li>
            <li>
              Stream keys stay in the OS keychain. Crash logs stay local and
              scrubbed; nothing uploads.
            </li>
            <li>
              Desktop ships as a source prototype (v0.1). No installer in this
              release.
            </li>
          </ul>
        </section>
      </main>

      <footer className="site-footer">
        <p className="footer-copy">
          <span>Windows-first · v0.1 prototype · GPL-3.0</span>
          <span className="footer-privacy">
            This page uses Vercel Web Analytics. The desktop app does not.
          </span>
        </p>
        <a href={GITHUB} rel="noopener noreferrer">
          ArcaLabs on GitHub
        </a>
      </footer>
    </div>
  );
}
