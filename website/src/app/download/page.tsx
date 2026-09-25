import type { Metadata } from "next";
import { PlatformDownloadList } from "@/components/DownloadButton";
import { brand } from "@/lib/brand";

export const metadata: Metadata = {
  title: "Download",
  description:
    "Download Piercast for macOS, Windows, or Linux from GitHub Releases.",
};

export default function DownloadPage() {
  return (
    <div className="mx-auto max-w-3xl px-5 py-16 sm:px-8 sm:py-20">
      <h1 className="font-display text-4xl font-semibold tracking-[-0.03em] text-ink sm:text-5xl">
        Download
      </h1>
      <p className="mt-4 text-lg leading-relaxed text-ink-muted">
        Installers ship on GitHub Releases. We detect your platform and highlight
        the matching build — assets live at{" "}
        <a
          className="text-ink underline decoration-primary/40 underline-offset-4 hover:decoration-primary"
          href={brand.github.releases}
          rel="noreferrer"
          target="_blank"
        >
          {brand.github.org}/{brand.github.repo}
        </a>
        .
      </p>
      <PlatformDownloadList />
      <p className="mt-8 text-sm text-ink-soft">
        Prefer CLI-only? Grab <code className="text-ink">piercast</code> and{" "}
        <code className="text-ink">piercastd</code> from the same release, or
        follow docs after install.
      </p>
    </div>
  );
}
