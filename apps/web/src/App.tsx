import { OptimizeDemo } from "./components/OptimizeDemo";
import "./App.css";

const GITHUB = "https://github.com/cxernlol/ArcaLabs";

export default function App() {
  return (
    <div className="page">
      <header className="site-header">
        <a className="brand" href="/">
          <img src="/logo.png" width={44} height={44} alt="Arca Labs" />
          <strong>Arca Labs</strong>
        </a>
        <a className="header-link" href={GITHUB} rel="noopener noreferrer">
          Source on GitHub
        </a>
      </header>

      <section className="hero">
        <div className="hero-copy">
          <h1>Stream without learning encoders</h1>
          <p>
            Arcalabs is lightweight streaming for YouTube, Kick, and Twitch. It
            picks resolution, frame rate, and bitrate for your PC — you confirm or
            undo, then go live.
          </p>
          <ul className="hero-stats">
            <li>
              <strong>Rust + Tauri</strong>
              System webview, not a bundled Chromium stack
            </li>
            <li>
              <strong>~1 min</strong>
              First-run setup to live
            </li>
            <li>
              <strong>Zero telemetry</strong>
              No analytics on this site or in the app
            </li>
          </ul>
        </div>
        <div className="hero-visual">
          <img
            className="hero-mark"
            src="/logo.png"
            width={320}
            height={320}
            alt=""
            aria-hidden
          />
        </div>
      </section>

      <section className="pillars" aria-label="Why Arcalabs">
        <article className="pillar">
          <h3>Light at idle</h3>
          <p>
            Electron ships a full browser. Tauri uses WebView2 on Windows so your
            RAM goes to capture and encode, not an idle Chromium process.
          </p>
        </article>
        <article className="pillar">
          <h3>Automatic first</h3>
          <p>
            Manual encoder fields stay one level down. You see what changed, not a
            wall of jargon.
          </p>
        </article>
        <article className="pillar">
          <h3>Privacy default</h3>
          <p>
            Hardware detection stays on your machine. The only outbound connection
            when you stream is to the platform you chose.
          </p>
        </article>
      </section>

      <OptimizeDemo />

      <section className="steps">
        <h2>Three steps. About a minute.</h2>
        <ol>
          <li>
            <span>Step 1</span>
            <p>Choose YouTube, Kick, or Twitch.</p>
          </li>
          <li>
            <span>Step 2</span>
            <p>Paste your stream key. Stored in the OS keychain, not plain text.</p>
          </li>
          <li>
            <span>Step 3</span>
            <p>Optimize for this PC, review the changes, finish setup, Go Live.</p>
          </li>
        </ol>
      </section>

      <section className="contrast" aria-labelledby="electron-title">
        <div className="contrast-inner">
          <h2 id="electron-title">Why not Electron?</h2>
          <p>
            Electron is excellent when you need its ecosystem. For a streamer tool
            that should stay quiet in the tray and small on RAM, shipping Chromium
            in every install works against you. Arcalabs keeps the UI thin and puts
            weight in Rust where capture and encode belong.
          </p>
        </div>
      </section>

      <footer className="site-footer">
        <span>Arca Labs · Windows-first desktop app in development</span>
        <a href={GITHUB}>GitHub</a>
      </footer>
    </div>
  );
}
