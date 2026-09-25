export function HeroAtmosphere() {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-0 overflow-hidden"
    >
      <div className="atmosphere-drift absolute -left-[20%] top-[-10%] h-[70vh] w-[70vw] rounded-full bg-[radial-gradient(circle_at_center,rgba(123,108,255,0.22),transparent_62%)]" />
      <div className="absolute right-[-10%] top-[8%] h-[55vh] w-[50vw] rounded-full bg-[radial-gradient(circle_at_center,rgba(11,11,15,0.08),transparent_65%)]" />
      <div className="absolute inset-x-0 bottom-0 h-[45%] bg-[linear-gradient(to_top,rgba(232,232,236,0.9),transparent)]" />
      <svg
        className="absolute bottom-0 left-1/2 h-[46%] w-[min(1100px,140%)] -translate-x-1/2 text-ink/80"
        viewBox="0 0 1100 420"
        fill="none"
      >
        <defs>
          <linearGradient id="water" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stopColor="#0B0B0F" stopOpacity="0.04" />
            <stop offset="100%" stopColor="#0B0B0F" stopOpacity="0.14" />
          </linearGradient>
        </defs>
        <rect x="0" y="280" width="1100" height="140" fill="url(#water)" />
        <g
          className="pier-draw"
          stroke="currentColor"
          strokeWidth="10"
          strokeLinecap="round"
          strokeLinejoin="round"
        >
          <path d="M180 300 V160" />
          <path d="M300 300 V120" />
          <path d="M420 300 V150" />
          <path d="M540 300 V170" />
          <path d="M660 300 V140" />
          <path d="M780 300 V180" />
          <path d="M900 300 V155" />
          <path d="M180 190 H900" />
        </g>
        <circle cx="900" cy="140" r="10" fill="#7B6CFF" />
        <path
          d="M140 318 C280 290, 420 340, 560 312 C700 284, 840 336, 980 308"
          stroke="#7B6CFF"
          strokeOpacity="0.35"
          strokeWidth="2"
          fill="none"
        />
      </svg>
    </div>
  );
}
