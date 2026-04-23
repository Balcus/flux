import "../../../App.css";
import "./FileRow.css";

export default function FileRow({
  path,
  type,
  selected,
  onSelect,
  onAction,
  actionLabel,
}: {
  path: string;
  type?: string;
  selected: boolean;
  onSelect: () => void;
  onAction: () => void;
  actionLabel: string;
}) {
  const badgeClass = (t: string) =>
    t === "Added" ? "badge-add" : t === "Deleted" ? "badge-del" : "badge-mod";

  const badgeLabel = (t: string) =>
    t === "Added" ? "A" : t === "Deleted" ? "D" : "M";

  return (
    <div className={`file-row${selected ? " active" : ""}`} onClick={onSelect}>
      <span
        className={`file-row-badge ${type ? badgeClass(type) : "badge-add"}`}
      >
        {type ? badgeLabel(type) : "A"}
      </span>
      <span className="file-row-name">{path}</span>
      <button
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
