export const brand = {
  name: "Piercast",
  tagline: "Launch anything. From anywhere.",
  siteUrl: process.env.NEXT_PUBLIC_SITE_URL ?? "https://piercast.io",
  colors: {
    primary: "#7B6CFF",
    ink: "#0B0B0F",
    fog: "#F5F5F7",
    success: "#30D158",
    warn: "#FF9F0A",
    danger: "#FF453A",
  },
  github: {
    org: "lfernando2703",
    repo: "piercast",
    url: "https://github.com/lfernando2703/piercast",
    releases: "https://github.com/lfernando2703/piercast/releases",
    latest: "https://github.com/lfernando2703/piercast/releases/latest",
  },
  docsUrl: "/docs",
} as const;

export type PlatformId = "macos" | "windows" | "linux" | "unknown";

export function detectPlatform(ua: string): PlatformId {
  const u = ua.toLowerCase();
  if (u.includes("mac")) return "macos";
  if (u.includes("win")) return "windows";
  if (u.includes("linux") || u.includes("x11")) return "linux";
  return "unknown";
}

export const platformLabels: Record<PlatformId, string> = {
  macos: "macOS",
  windows: "Windows",
  linux: "Linux",
  unknown: "your platform",
};

export function releaseUrlFor(_platform: PlatformId): string {
  return brand.github.latest;
}
