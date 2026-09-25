"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { BrandLockup } from "./BrandMark";

const links = [
  { href: "/features", label: "Features" },
  { href: "/download", label: "Download" },
  { href: "/docs", label: "Docs" },
  { href: "/open-source", label: "Open Source" },
  { href: "/changelog", label: "Changelog" },
] as const;

export function SiteHeader() {
  const pathname = usePathname();

  return (
    <header className="relative z-20 border-b border-ink/6 bg-fog/80 backdrop-blur-md">
      <div className="mx-auto flex h-16 max-w-6xl items-center justify-between px-5 sm:px-8">
        <BrandLockup />
        <nav className="hidden items-center gap-7 md:flex" aria-label="Primary">
          {links.map((link) => {
            const active =
              pathname === link.href || pathname.startsWith(`${link.href}/`);
            return (
              <Link
                key={link.href}
                href={link.href}
                className={`text-sm transition-colors ${active ? "text-ink" : "text-ink-soft hover:text-ink"}`}
              >
                {link.label}
              </Link>
            );
          })}
        </nav>
        <Link
          href="/download"
          className="rounded-xl bg-ink px-3.5 py-2 text-sm font-medium text-fog transition hover:bg-ink/90"
        >
          Download
        </Link>
      </div>
      <nav
        className="flex gap-5 overflow-x-auto border-t border-ink/6 px-5 py-2.5 text-sm md:hidden"
        aria-label="Mobile"
      >
        {links.map((link) => {
          const active =
            pathname === link.href || pathname.startsWith(`${link.href}/`);
          return (
            <Link
              key={link.href}
              href={link.href}
              className={`whitespace-nowrap ${active ? "text-ink" : "text-ink-soft"}`}
            >
              {link.label}
            </Link>
          );
        })}
      </nav>
    </header>
  );
}
