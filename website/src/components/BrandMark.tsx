import Link from "next/link";

export function BrandMark({
  size = 36,
  className = "",
}: {
  size?: number;
  className?: string;
}) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 128 128"
      role="img"
      aria-label="Piercast"
      className={className}
    >
      <rect width="128" height="128" rx="28" fill="#0B0B0F" />
      <g
        fill="none"
        stroke="#7B6CFF"
        strokeWidth="8"
        strokeLinecap="round"
        strokeLinejoin="round"
      >
        <path d="M28 92 V44" />
        <path d="M52 92 V36" />
        <path d="M76 92 V44" />
        <path d="M100 92 V52" />
        <path d="M28 52 H100" />
      </g>
      <circle cx="100" cy="40" r="6" fill="#7B6CFF" />
    </svg>
  );
}

export function BrandLockup({
  className = "",
}: {
  className?: string;
}) {
  return (
    <Link
      href="/"
      className={`inline-flex items-center gap-2.5 no-underline ${className}`}
      aria-label="Piercast home"
    >
      <BrandMark size={28} />
      <span className="font-display text-[1.05rem] font-semibold tracking-[-0.02em] text-ink">
        Piercast
      </span>
    </Link>
  );
}
