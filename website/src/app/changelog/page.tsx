import type { Metadata } from "next";
import { PageShell } from "@/components/PageShell";
import { brand } from "@/lib/brand";

export const metadata: Metadata = {
  title: "Changelog",
  description: "Release notes for Piercast.",
};

const entries = [
  {
    version: "0.1.0",
    date: "Unreleased",
    notes: [
      "Initial monorepo: piercastd, CLI, schema, MCP tools, and agent skill.",
      "Marketing site at piercast.io with download links to GitHub Releases.",
      "Native desktop and mobile shells in progress.",
    ],
  },
] as const;

export default function ChangelogPage() {
  return (
    <PageShell
      title="Changelog"
      lead="What shipped, what is shipping. Full release assets on GitHub."
    >
      {entries.map((entry) => (
        <article key={entry.version} className="border-t border-ink/8 pt-8 first:border-t-0 first:pt-0">
          <header className="flex flex-wrap items-baseline gap-3">
            <h2 className="font-display text-2xl font-semibold tracking-[-0.02em] text-ink">
              {entry.version}
            </h2>
            <time className="text-sm text-ink-soft">{entry.date}</time>
          </header>
          <ul className="mt-4 list-disc space-y-2 pl-5">
            {entry.notes.map((note) => (
              <li key={note}>{note}</li>
            ))}
          </ul>
        </article>
      ))}
      <p>
        <a
          className="text-ink underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
          href={brand.github.releases}
          rel="noreferrer"
          target="_blank"
        >
          All GitHub Releases →
        </a>
      </p>
    </PageShell>
  );
}
