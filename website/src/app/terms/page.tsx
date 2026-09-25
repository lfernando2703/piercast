import type { Metadata } from "next";
import { PageShell } from "@/components/PageShell";
import { brand } from "@/lib/brand";

export const metadata: Metadata = {
  title: "Terms",
  description: "Terms of use for Piercast software and this website.",
};

export default function TermsPage() {
  return (
    <PageShell
      title="Terms"
      lead="Plain-language terms for using Piercast and piercast.io."
    >
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Software license
      </h2>
      <p>
        Piercast source and binaries are provided under the Apache License 2.0.
        See the LICENSE file in the{" "}
        <a
          className="text-ink underline decoration-primary/40 underline-offset-4"
          href={brand.github.url}
          rel="noreferrer"
          target="_blank"
        >
          repository
        </a>
        .
      </p>
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Local-first product
      </h2>
      <p>
        Piercast runs on your devices. You are responsible for the apps you
        register, the commands you configure, and any network exposure you
        enable (including Tailscale Funnel). Piercast does not require a cloud
        account; telemetry remains off unless you opt in.
      </p>
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Website
      </h2>
      <p>
        Content on piercast.io is provided as-is for information and download
        links. Download links currently point at GitHub Releases for{" "}
        <code className="text-ink">
          {brand.github.org}/{brand.github.repo}
        </code>
        .
      </p>
      <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
        Disclaimer
      </h2>
      <p>
        Software is provided without warranty of any kind. To the extent
        permitted by law, authors are not liable for damages arising from use
        or inability to use Piercast.
      </p>
      <p className="text-sm text-ink-soft">Last updated: 2026-09-25</p>
    </PageShell>
  );
}
