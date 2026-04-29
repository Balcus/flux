import "./HeadChip.css";

interface HeadChipProps {
  name: string;
  color: string;
}

export function HeadChip({ name, color }: HeadChipProps) {
  return (
    <span className="branch-pill" style={{ backgroundColor: color }}>
      {name}
    </span>
  );
}
