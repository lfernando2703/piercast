import type { Metadata } from "next";
import { PageShell } from "@/components/PageShell";

export const metadata: Metadata = {
  title: "Privacy",
  description:
    "Piercast is local-first. Telemetry is off by default. No cloud account required.",
};

export default function PrivacyPage() {
  return (
    <PageShell
      title="Privacy"
      lead="Local-first by design. Your apps and registry stay on your machine."
    >
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        What we collect
      </h2>
      <p>
        By default, Piercast collects nothing. There is no cloud account, no
        mandatory analytics, and no automatic upload of logs, metrics, or
        registry contents.
      </p>
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Telemetry
      </h2>
      <p>
        Telemetry is <strong className="font-semibold text-ink">off</strong> by
        default. If a future optional crash reporter ships, it will be an
        explicit Settings opt-in and will never include source code or secrets
        from your apps.
      </p>
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Pairing &amp; network
      </h2>
      <p>
        Mobile pairing uses a short-lived bootstrap token you scan locally.
        Device tokens stay between your devices. When Mobile Access is enabled,
        the control API may also bind your Tailscale IP — never{" "}
        <code className="text-ink">0.0.0.0</code>. Funnel/serve expose only the
        target app port you choose, never the daemon API.
      </p>
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Website
      </h2>
      <p>
        This marketing site may use standard hosting logs (IP, user agent) from
        the CDN/host. We do not sell personal data. Contact: privacy@piercast.io
        once mail is provisioned.
      </p>
      <p className="text-sm text-ink-soft">Last updated: 2026-09-25</p>
    </PageShell>
  );
}
