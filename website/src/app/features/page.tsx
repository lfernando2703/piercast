import type { Metadata } from "next";
import { DownloadButton } from "@/components/DownloadButton";
import { PageShell } from "@/components/PageShell";

export const metadata: Metadata = {
  title: "Features",
  description:
    "Launch, monitor, expose, and control local apps from desktop, CLI, MCP, and mobile.",
};

const features = [
  {
    title: "Library & inspector",
    body: "Browse All, Running, Unhealthy, and Favorites. Inspect health sparklines, CPU and memory, usage, logs, expose mode, and depends-on order.",
  },
  {
    title: "Command palette",
    body: "⌘K / Ctrl+K fuzzy-finds apps and actions — Start, Open, Stop, Restart, Kill, Expose — without leaving the keyboard.",
  },
  {
    title: "Dependency-aware start",
    body: "Topological start order from depends_on. Cycles fail with dependency_cycle. Missing modes fail with mode_unavailable — Piercast never invents Start commands.",
  },
  {
    title: "Open that actually starts",
    body: "Open auto-starts last mode (or development), waits for healthy or times out, then opens open_url with {port} substituted.",
  },
  {
    title: "Tailscale expose",
    body: "Serve for tailnet-only HTTPS or Funnel for public. Only the app port is exposed — never the control plane.",
  },
  {
    title: "MCP + skill",
    body: "Agents register via piercast_upsert_app and the Piercast skill after scaffolding — piercast.yml in, Library updated.",
  },
] as const;

export default function FeaturesPage() {
  return (
    <PageShell
      title="Features"
      lead="Everything you need to run a fleet of local apps without a cloud account."
    >
      <ul className="space-y-10">
        {features.map((feature) => (
          <li key={feature.title}>
            <h2 className="font-display text-xl font-semibold tracking-[-0.02em] text-ink">
              {feature.title}
            </h2>
            <p className="mt-2">{feature.body}</p>
          </li>
        ))}
      </ul>
      <div className="pt-4">
        <DownloadButton showHint />
      </div>
    </PageShell>
  );
}
