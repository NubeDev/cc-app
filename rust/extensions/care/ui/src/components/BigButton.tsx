interface Props { onClick: () => void; label: string; variant?: "primary" | "ghost"; }
export function BigButton({ onClick, label, variant = "primary" }: Props) {
  const cls = variant === "primary"
    ? "bg-foreground text-background"
    : "border border-foreground/20";
  return (
    <button
      onClick={onClick}
      className={`min-h-[var(--care-touch-target)] w-full rounded-[var(--care-radius)] px-4 py-3 text-base ${cls}`}
    >
      {label}
    </button>
  );
}