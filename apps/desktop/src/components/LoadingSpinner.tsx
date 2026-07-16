interface LoadingSpinnerProps {
  text?: string;
}

export default function LoadingSpinner({ text = 'Loading...' }: LoadingSpinnerProps) {
  return (
    <div className="flex flex-col items-center justify-center space-y-3 py-10">
      <div className="w-6 h-6 border-2 border-accent-primary border-t-transparent rounded-full animate-spin"></div>
      <span className="text-text-muted text-[11px] font-medium tracking-wide animate-pulse">
        {text}
      </span>
    </div>
  );
}
