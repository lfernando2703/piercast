import type { Metadata } from "next";
import Link from "next/link";
import { PageShell } from "@/components/PageShell";
import { brand } from "@/lib/brand";

export const metadata: Metadata = {
  title: "Docs",
  description: "Piercast documentation — CLI, daemon, MCP, and piercast.yml.",
};

export default function DocsPage() {
  return (
    <PageShell
      title="Docs"
      lead="Guides live with the open-source repo while the dedicated docs surface comes online."
    >
      <p>
        Start with the repository README for install, daemon data dirs, control
        plane auth, and the <code className="text-ink">piercast.yml</code>{" "}
        schema.
      </p>
      <ul className="list-disc space-y-2 pl-5">
        <li>
          <a
            className="text-ink underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
            href={`${brand.github.url}#readme`}
            rel="noreferrer"
            target="_blank"
          >
            README on GitHub
          </a>
        </li>
        <li>
          <a
            className="text-ink underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
            href={`${brand.github.url}/tree/main/packages/schema`}
            rel="noreferrer"
            target="_blank"
          >
            Canonical schema
          </a>
        </li>
        <li>
          <a
            className="text-ink underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
            href={`${brand.github.url}/tree/main/skill/piercast`}
            rel="noreferrer"
            target="_blank"
          >
            Agent skill
          </a>
        </li>
        <li>
          <Link
            className="text-ink underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
            href="/changelog"
          >
            Changelog
          </Link>
        </li>
      </ul>
      <p className="text-sm text-ink-soft">
        Short link domain: piercast.sh (docs redirect planned).
      </p>
    </PageShell>
  );
}
