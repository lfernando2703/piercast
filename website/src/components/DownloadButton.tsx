"use client";

import { useEffect, useState } from "react";
import {
  detectPlatform,
  platformLabels,
  releaseUrlFor,
  type PlatformId,
} from "@/lib/brand";

type Props = {
  className?: string;
  label?: string;
  showHint?: boolean;
};

export function DownloadButton({
  className = "",
  label = "Download",
  showHint = false,
}: Props) {
  const [platform, setPlatform] = useState<PlatformId>("unknown");

  useEffect(() => {
    setPlatform(detectPlatform(navigator.userAgent));
  }, []);

  const href = releaseUrlFor(platform);
  const platformLabel = platformLabels[platform];

  return (
    <div className="inline-flex flex-col items-start gap-2">
      <a
        href={href}
        className={`inline-flex items-center justify-center rounded-xl bg-primary px-5 py-3 text-sm font-semibold text-fog transition hover:brightness-110 ${className}`}
        rel="noreferrer"
        target="_blank"
      >
        {label}
        {platform !== "unknown" ? ` for ${platformLabel}` : ""}
      </a>
      {showHint ? (
        <p className="text-xs text-ink-soft">
          Opens GitHub Releases · {platformLabel}
        </p>
      ) : null}
    </div>
  );
}

export function PlatformDownloadList() {
  const [platform, setPlatform] = useState<PlatformId>("unknown");

  useEffect(() => {
    setPlatform(detectPlatform(navigator.userAgent));
  }, []);

  const items: { id: PlatformId; title: string; detail: string }[] = [
    {
      id: "macos",
      title: "macOS",
      detail: "Universal .dmg · Homebrew cask coming soon",
    },
    {
      id: "windows",
      title: "Windows",
      detail: "MSI and MSIX on GitHub Releases",
    },
    {
      id: "linux",
      title: "Linux",
      detail: ".deb, .rpm, and AppImage",
    },
  ];

  return (
    <ul className="mt-10 divide-y divide-ink/8 border-y border-ink/8">
      {items.map((item) => {
        const preferred = platform === item.id;
        return (
          <li
            key={item.id}
            className={`flex flex-col gap-3 py-6 sm:flex-row sm:items-center sm:justify-between ${
              preferred ? "bg-primary/5" : ""
            }`}
          >
            <div>
              <p className="font-display text-xl font-semibold tracking-[-0.02em]">
                {item.title}
                {preferred ? (
                  <span className="ml-2 text-sm font-medium text-primary">
                    Detected
                  </span>
                ) : null}
              </p>
              <p className="mt-1 text-sm text-ink-muted">{item.detail}</p>
            </div>
            <a
              href={releaseUrlFor(item.id)}
              className={`inline-flex rounded-xl px-4 py-2.5 text-sm font-semibold transition ${
                preferred
                  ? "bg-primary text-fog hover:brightness-110"
                  : "bg-ink text-fog hover:bg-ink/90"
              }`}
              rel="noreferrer"
              target="_blank"
            >
              Get {item.title}
            </a>
          </li>
        );
      })}
    </ul>
  );
}
