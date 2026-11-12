import { useEffect, useRef, useState } from "react";
import { inject } from "@vercel/analytics";
import "./AnalyticsConsent.css";

type ConsentChoice = "accepted" | "rejected";

const STORAGE_KEY = "arca-site-analytics-consent";
const OPEN_SETTINGS_EVENT = "arca:open-analytics-settings";

function readConsent(): {
  choice: ConsentChoice | null;
  storageAvailable: boolean;
} {
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    return {
      choice: stored === "accepted" || stored === "rejected" ? stored : null,
      storageAvailable: true,
    };
  } catch {
    return { choice: null, storageAvailable: false };
  }
}

export default function AnalyticsConsent() {
  const [initialConsent] = useState(readConsent);
  const [choice, setChoice] = useState(initialConsent.choice);
  const [isOpen, setIsOpen] = useState(initialConsent.choice === null);
  const [status, setStatus] = useState(
    initialConsent.storageAvailable
      ? ""
      : "This browser cannot access local storage. Analytics will stay off.",
  );
  const acceptRef = useRef<HTMLButtonElement>(null);
  const settingsTriggerRef = useRef<HTMLElement | null>(null);
  const returnFocusRef = useRef(false);

  useEffect(() => {
    if (choice === "accepted") inject({ debug: false });
  }, [choice]);

  useEffect(() => {
    const updateTriggers = () => {
      document
        .querySelectorAll<HTMLButtonElement>("[data-open-analytics-settings]")
        .forEach((button) => {
          button.setAttribute("aria-expanded", String(isOpen));
        });
      if (isOpen && settingsTriggerRef.current) {
        acceptRef.current?.focus({ preventScroll: true });
      }
    };
    updateTriggers();
  }, [isOpen]);

  useEffect(() => {
    const openSettings = () => {
      setIsOpen(true);
    };
    const handleClick = (event: MouseEvent) => {
      if (!(event.target instanceof Element)) return;
      const trigger = event.target.closest<HTMLElement>(
        "[data-open-analytics-settings]",
      );
      if (!trigger) return;
      settingsTriggerRef.current = trigger;
      window.dispatchEvent(new Event(OPEN_SETTINGS_EVENT));
    };
    document.addEventListener("click", handleClick);
    window.addEventListener(OPEN_SETTINGS_EVENT, openSettings);
    return () => {
      document.removeEventListener("click", handleClick);
      window.removeEventListener(OPEN_SETTINGS_EVENT, openSettings);
    };
  }, []);

  useEffect(() => {
    if (!isOpen && returnFocusRef.current) {
      settingsTriggerRef.current?.focus();
      returnFocusRef.current = false;
    }
  }, [isOpen]);

  const choose = (nextChoice: ConsentChoice) => {
    try {
      window.localStorage.setItem(STORAGE_KEY, nextChoice);
    } catch {
      setStatus(
        "Your choice could not be saved. Analytics remains off until storage is available.",
      );
      return;
    }
    setChoice(nextChoice);
    setStatus("");
    returnFocusRef.current = settingsTriggerRef.current !== null;
    setIsOpen(false);
  };

  return (
    <div className="analytics-consent-slot">
      <aside
        className="analytics-consent"
        id="analytics-consent"
        aria-labelledby="analytics-consent-title"
        hidden={!isOpen}
      >
        <div className="consent-copy">
          <p className="consent-title" id="analytics-consent-title">
            Optional site analytics
          </p>
          <p>
            Vercel Web Analytics measures page views and device details. It does
            not use third-party cookies.{" "}
            <a href="/privacy/">Read the privacy policy</a>.
          </p>
          {status && (
            <p className="consent-status" role="status">
              {status}
            </p>
          )}
        </div>
        <div className="consent-actions">
          <button
            ref={acceptRef}
            className="consent-button consent-accept"
            type="button"
            onClick={() => choose("accepted")}
          >
            Accept
          </button>
          <button
            className="consent-button consent-reject"
            type="button"
            onClick={() => choose("rejected")}
          >
            Reject
          </button>
        </div>
      </aside>
    </div>
  );
}
