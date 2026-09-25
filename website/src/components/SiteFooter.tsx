import Link from "next/link";
import { brand } from "@/lib/brand";
import { BrandMark } from "./BrandMark";

const footerLinks = [
  { href: "/features", label: "Features" },
  { href: "/download", label: "Download" },
  { href: "/docs", label: "Docs" },
  { href: "/open-source", label: "Open Source" },
  { href: "/changelog", label: "Changelog" },
  { href: "/privacy", label: "Privacy" },
  { href: "/terms", label: "Terms" },
] as const;

export function SiteFooter() {
  return (
    <footer className="border-t border-ink/8 bg-fog-deep/40">
      <div className="mx-auto flex max-w-6xl flex-col gap-8 px-5 py-12 sm:px-8">
        <div className="flex flex-col gap-6 sm:flex-row sm:items-start sm:justify-between">
          <div className="max-w-sm">
            <div className="mb-3 flex items-center gap-2.5">
              <BrandMark size={28} />
              <span className="font-display text-lg font-semibold tracking-[-0.02em]">
                Piercast
              </span>
            </div>
            <p className="text-sm leading-relaxed text-ink-muted">
              {brand.tagline} Local-first control for the apps you build and ship.
            </p>
          </div>
          <nav
            className="grid grid-cols-2 gap-x-10 gap-y-2 text-sm text-ink-muted sm:grid-cols-3"
            aria-label="Footer"
          >
            {footerLinks.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className="transition hover:text-ink"
              >
                {link.label}
              </Link>
            ))}
            <a
              href={brand.github.url}
              className="transition hover:text-ink"
              rel="noreferrer"
              target="_blank"
            >
              GitHub
            </a>
          </nav>
        </div>
        <p className="text-xs text-ink-soft">
          © {new Date().getFullYear()} Piercast. Apache-2.0.
        </p>
      </div>
    </footer>
  );
}
