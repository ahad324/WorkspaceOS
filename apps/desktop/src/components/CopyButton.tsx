import { useState } from 'react';

interface CopyButtonProps {
  value: string;
}

export default function CopyButton({ value }: CopyButtonProps) {
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy: ', err);
    }
  };

  return (
    <button
      onClick={handleCopy}
      className="p-1.5 rounded-lg hover:bg-surface-secondary border border-border-subtle text-text-muted hover:text-text-primary transition-all duration-150 cursor-pointer flex items-center justify-center"
      title="Copy to clipboard"
    >
      {copied ? (
        <span className="material-symbols-rounded text-success-main text-sm animate-bounce">
          check
        </span>
      ) : (
        <span className="material-symbols-rounded text-sm">content_copy</span>
      )}
    </button>
  );
}
