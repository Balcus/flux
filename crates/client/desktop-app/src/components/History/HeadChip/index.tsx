import { BranchIcon } from "../../../assets/icons";

import "./HeadChip.css";

interface HeadChipProps {
  name: string;
  color: string;
}

export function HeadChip({ name, color }: HeadChipProps) {
  return (
    <span className="head-chip">
      <img src={BranchIcon} />
      <span style={{ color: `${color}` }}>{name}</span>
    </span>
  );
}
