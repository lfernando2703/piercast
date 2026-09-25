import type { Metadata } from "next";
import { Bricolage_Grotesque, Inter } from "next/font/google";
import { SiteFooter } from "@/components/SiteFooter";
import { SiteHeader } from "@/components/SiteHeader";
import { brand } from "@/lib/brand";
import "./globals.css";

const inter = Inter({
  variable: "--font-inter",
  subsets: ["latin"],
  display: "swap",
});

const display = Bricolage_Grotesque({
  variable: "--font-display",
  subsets: ["latin"],
  display: "swap",
});

const site = brand.siteUrl;

export const metadata: Metadata = {
  metadataBase: new URL(site),
  title: {
    default: "Piercast — Launch anything. From anywhere.",
    template: "%s · Piercast",
  },
  description:
    "Piercast is the local-first desktop control plane for launching, monitoring, exposing, and controlling the apps you build.",
  openGraph: {
    title: "Piercast",
    description: brand.tagline,
    url: site,
    siteName: "Piercast",
    type: "website",
  },
  icons: {
    icon: "/mark-128.png",
    apple: "/mark-512.png",
  },
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className={`${inter.variable} ${display.variable} h-full`}>
      <body className="min-h-full font-sans antialiased">
        {/*
          THESIS: Piercast owns the pier — a calm fog shore where local apps ship from one deck; refuses purple-gradient SaaS hero stacks and metric strips.
          OWN-WORLD: Fog #F5F5F7 field, ink #0B0B0F type, periwinkle #7B6CFF signal only; Bricolage Grotesque display + Inter body; pier pylons as geometry, not cards.
          STORY: Visitor sees Piercast as the launch deck for vibe-coded local apps, believes it stays local-first, downloads.
          FIRST VIEWPORT: Full-bleed fog atmosphere with pier silhouette; brand wordmark as dominant display; tagline; single Download CTA; no cards/stats.
          FORM: Harbor deck at dawn — ordered list position 1 (pier/harbor ritual); seed key unattended-brief-lock.
          FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
        */}
        <div className="flex min-h-full flex-col">
          <SiteHeader />
          <main className="flex-1">{children}</main>
          <SiteFooter />
        </div>
      </body>
    </html>
  );
}
