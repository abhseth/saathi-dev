import React from "react";
import type { School } from "../types";

function CopyButton({ value }: { value: string }) {
  const [copied, setCopied] = React.useState(false);
  function handleCopy() {
    void navigator.clipboard.writeText(value).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }
  return (
    <button type="button" className="copy-btn" onClick={handleCopy} title={`Copy ${value}`}>
      {copied ? "✓" : "⎘"}
    </button>
  );
}

export function ContactLink({
  kind,
  value,
  withCopy = true,
}: {
  kind: "tel" | "mail";
  value: string | null | undefined;
  withCopy?: boolean;
}) {
  const trimmed = (value ?? "").trim();
  if (!trimmed) {
    return <span className="contact-link contact-link-empty">—</span>;
  }
  const href =
    kind === "tel"
      ? `tel:${trimmed.replace(/[^\d+]/g, "")}`
      : `mailto:${trimmed}`;
  return (
    <span className={`contact-link-wrap contact-link-${kind}`}>
      <a
        href={href}
        className="contact-link"
        onClick={(e) => e.stopPropagation()}
      >
        {trimmed}
      </a>
      {withCopy ? <CopyButton value={trimmed} /> : null}
    </span>
  );
}

export function SchoolContactsBar({ school }: { school: School | null }) {
  if (!school) return null;
  const contacts = [
    { role: "SPOC", name: school.school_spoc_name, mobile: school.school_spoc_mobile, email: school.school_spoc_email },
    { role: "Principal", name: school.principal_name, mobile: school.principal_mobile, email: school.principal_email },
    { role: "Center Head", name: school.center_head_name, mobile: school.center_head_mobile, email: school.center_head_email },
  ].filter((c) => c.name);
  if (contacts.length === 0) return null;
  return (
    <details className="school-contacts-bar">
      <summary>{school.name} — contacts</summary>
      <div className="school-contacts-list">
        {contacts.map((c) => (
          <div key={c.role} className="school-contact-row">
            <span className="contact-role">{c.role}</span>
            <span className="contact-name">{c.name}</span>
            {c.mobile ? (
              <span className="contact-field">
                <ContactLink kind="tel" value={c.mobile} />
              </span>
            ) : null}
            {c.email ? (
              <span className="contact-field">
                <ContactLink kind="mail" value={c.email} />
              </span>
            ) : null}
          </div>
        ))}
      </div>
    </details>
  );
}
