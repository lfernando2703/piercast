import Link from "next/link";
import { DownloadButton } from "@/components/DownloadButton";
import { HeroAtmosphere } from "@/components/HeroAtmosphere";

export default function HomePage() {
  return (
    <>
      <section className="relative isolate min-h-[calc(100vh-4rem)] overflow-hidden">
        <HeroAtmosphere />
        <div className="relative z-10 mx-auto flex min-h-[calc(100vh-4rem)] max-w-6xl flex-col justify-center px-5 pb-28 pt-16 sm:px-8 sm:pb-36">
          <p className="animate-rise font-display text-[clamp(3.5rem,12vw,7.5rem)] font-semibold leading-[0.92] tracking-[-0.04em] text-ink">
            Piercast
          </p>
          <p className="animate-rise-delay-1 mt-6 max-w-xl text-xl leading-snug text-ink-muted sm:text-2xl">
            Launch anything. From anywhere.
          </p>
          <div className="animate-rise-delay-2 mt-10">
            <DownloadButton />
          </div>
        </div>
      </section>

      <section className="border-t border-ink/8 bg-fog">
        <div className="mx-auto grid max-w-6xl gap-12 px-5 py-20 sm:px-8 lg:grid-cols-[1.1fr_0.9fr] lg:gap-16">
          <div>
            <h2 className="font-display text-3xl font-semibold tracking-[-0.03em] sm:text-4xl">
              One deck for every local app.
            </h2>
            <p className="mt-4 max-w-prose text-base leading-relaxed text-ink-muted">
              Register vibe-coded projects once. Piercast starts dependencies in
              order, watches health, streams logs, and opens the right URL —
              from desktop, CLI, MCP, Raycast, or your phone.
            </p>
          </div>
          <div className="space-y-8 text-sm leading-relaxed text-ink-muted">
            <div>
              <h3 className="font-display text-lg font-semibold text-ink">
                Start with intent
              </h3>
              <p className="mt-2">
                Development or production modes you define — never invented
                commands, never surprise deploys on Open.
              </p>
            </div>
            <div>
              <h3 className="font-display text-lg font-semibold text-ink">
                Stay local-first
              </h3>
              <p className="mt-2">
                Control plane on loopback. Telemetry off by default. Expose only
                the app port you choose via Tailscale serve or funnel.
              </p>
            </div>
            <div>
              <h3 className="font-display text-lg font-semibold text-ink">
                Agent-ready
              </h3>
              <p className="mt-2">
                Embedded MCP tools and a skill so agents write{" "}
                <code className="text-ink">piercast.yml</code> and register
                without ceremony.
              </p>
            </div>
          </div>
        </div>
      </section>

      <section className="border-t border-ink/8 bg-ink text-fog">
        <div className="mx-auto flex max-w-6xl flex-col items-start gap-6 px-5 py-20 sm:px-8 sm:flex-row sm:items-end sm:justify-between">
          <div>
            <h2 className="font-display text-3xl font-semibold tracking-[-0.03em] sm:text-4xl">
              Ready when your apps are.
            </h2>
            <p className="mt-3 max-w-md text-fog/70">
              Native shells for macOS, Windows, and Linux — same daemon, same
              API.
            </p>
          </div>
          <div className="flex flex-wrap gap-3">
            <DownloadButton className="!bg-primary" />
            <Link
              href="/features"
              className="inline-flex items-center rounded-xl border border-fog/20 px-5 py-3 text-sm font-semibold text-fog transition hover:border-fog/40"
            >
              See features
            </Link>
          </div>
        </div>
      </section>
    </>
  );
}
