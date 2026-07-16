export default function WorkspaceLogo({ className = 'w-8 h-8' }: { className?: string }) {
  return (
    <svg
      className={`${className} text-accent-primary animate-pulse`}
      viewBox="0 0 32 32"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <circle
        cx="16"
        cy="16"
        r="14"
        stroke="currentColor"
        strokeWidth="2"
        strokeDasharray="5 3"
        className="origin-center animate-[spin_18s_linear_infinite]"
      />
      <circle
        cx="16"
        cy="16"
        r="9"
        stroke="currentColor"
        strokeWidth="2.5"
        className="origin-center animate-[spin_8s_linear_infinite_reverse] opacity-80"
        strokeDasharray="14 4"
      />
      <circle
        cx="16"
        cy="16"
        r="4.5"
        fill="currentColor"
        className="animate-ping"
        style={{ animationDuration: '3s' }}
      />
      <circle cx="16" cy="16" r="3.5" fill="currentColor" />
    </svg>
  );
}
