import { AddedIcon, ChangedIcon, DeletedIcon } from "../../../assets/icons";
import { ChangeType } from "../../../models/ChangeType";
import "./FileRow.css";

const ICON_SRC: Record<ChangeType, string> = {
  Added: AddedIcon,
  Modified: ChangedIcon,
  Deleted: DeletedIcon,
};

export default function FileRow({
  path,
  type,
  selected,
  onSelect,
  onAction,
  actionLabel,
}: {
  path: string;
  type?: ChangeType;
  selected: boolean;
  onSelect: () => void;
  onAction: () => void;
  actionLabel: string;
}) {
  const resolvedType: ChangeType = type ?? "Added";
  const iconClass = `file-row-icon ${resolvedType.toLowerCase()}${selected ? " icon-selected" : ""}`;

  return (
    <div className={`file-row${selected ? " active" : ""}`} onClick={onSelect}>
      <img
        src={ICON_SRC[resolvedType]}
        width={14}
        height={14}
        className={iconClass}
      />
      <span className="file-row-name">{path}</span>
      <button
        className="file-row-action"
        onClick={(e) => {
          e.stopPropagation();
          onAction();
        }}
      >
        {actionLabel}
      </button>
    </div>
  );
}
