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
  link?: string;
}

export const MENU_ITEMS: MenuItem[] = [
  {
    id: "workspace",
    label: "Workspace",
    children: [
      { id: "history", label: "History", icon: HistoryIcon, link: "/history" },
      { id: "settings", label: "Settings", icon: SettingsIcon, link: "/settings" },
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
