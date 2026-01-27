import {
  BranchIcon,
  FolderIcon,
  HistoryIcon,
  SettingsIcon,
} from "../assets/icons";

export interface MenuItem {
  id: string;
  label: string;
  children?: MenuItem[];
  icon?: string;
  className?: string;
}

export const MENU_ITEMS: MenuItem[] = [
  {
    id: "workspace",
    label: "Workspace",
    children: [
      { id: "history", label: "History", icon: HistoryIcon },
      { id: "settings", label: "Settings", icon: SettingsIcon },
    ],
    icon: FolderIcon,
  },
  {
    id: "branches",
    label: "Branches",
    children: [],
    icon: BranchIcon,
  },
];
