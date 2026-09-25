import type { Metadata } from "next";
import { PageShell } from "@/components/PageShell";
import { brand } from "@/lib/brand";

export const metadata: Metadata = {
  title: "Open Source",
  description:
    "Piercast is open source under Apache-2.0 — daemon, CLI, MCP, and shells.",
};

export default function OpenSourcePage() {
  return (
    <PageShell
      title="Open Source"
      lead="Piercast is built in the open under the Apache-2.0 license."
    >
      <p>
        The shared Rust daemon, CLI, MCP server, schema, skill, and desktop
        shells live in one monorepo. Contributions welcome once the foundation
        lands.
      </p>
      <p>
        <a
          className="inline-flex rounded-xl bg-ink px-4 py-2.5 text-sm font-semibold text-fog transition hover:bg-ink/90"
          href={brand.github.url}
          rel="noreferrer"
          target="_blank"
        >
          View on GitHub
        </a>
      </p>
      <ul className="list-disc space-y-2 pl-5">
        <li>License: Apache-2.0</li>
        <li>
          Current remote:{" "}
          <a
            className="text-ink underline decoration-primary/40 underline-offset-4"
            href={brand.github.url}
            rel="noreferrer"
            target="_blank"
          >
            github.com/{brand.github.org}/{brand.github.repo}
          </a>
        </li>
        <li>
          Code of conduct and security policy ship in-repo — report
          vulnerabilities privately via SECURITY.md.
        </li>
      </ul>
    </PageShell>
  );
}
