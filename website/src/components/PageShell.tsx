import type { ReactNode } from "react";

export function PageShell({
  title,
  lead,
  children,
}: {
  title: string;
  lead?: string;
  children: ReactNode;
}) {
  return (
    <div className="mx-auto max-w-3xl px-5 py-16 sm:px-8 sm:py-20">
      <h1 className="font-display text-4xl font-semibold tracking-[-0.03em] text-ink sm:text-5xl">
        {title}
      </h1>
      {lead ? (
        <p className="mt-4 text-lg leading-relaxed text-ink-muted">{lead}</p>
      ) : null}
      <div className="mt-10 space-y-6 text-base leading-relaxed text-ink-muted">
        {children}
      </div>
    </div>
  );
}
