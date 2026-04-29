import {
  BranchIcon,
  CommitIcon,
  FileStatus,
  FolderIcon,
  HistoryIcon,
  PushIcon,
  SettingsIcon,
} from "../assets/icons";

export const LANE_CONFIG = {
  LANE_W: 16,
  ROW_H: 32,
  DOT_R: 4,
  LANE_COLORS: [
    "#4c8ef7",
    "#e06c75",
    "#98c379",
    "#e5c07b",
    "#c678dd",
    "#56b6c2",
    "#d19a66",
    "#61afef",
    "#f28b6a",
    "#7ec8a4",
  ],
};

export interface MenuItem {
  id: string;
  label: string;
  children?: MenuItem[];
  icon?: string;
  className?: string;
  link?: string;
  contextMenuId?: string;
}

export const BRANCH_CONTEXT_MENU_ID = "branch-context-menu";

export const MENU_ITEMS: MenuItem[] = [
  {
    id: "workspace",
    label: "Workspace",
    children: [
      { id: "history", label: "History", icon: HistoryIcon, link: "/history" },
      {
        id: "file status",
        label: "File Status",
        icon: FileStatus,
        link: "/file-status",
      },
      {
        id: "settings",
        label: "Settings",
        icon: SettingsIcon,
        link: "/settings",
      },
    ],
    icon: FolderIcon,
  },
  {
    id: "branches",
    label: "Branches",
    children: [],
    icon: BranchIcon,
    contextMenuId: BRANCH_CONTEXT_MENU_ID,
  },
];

export interface ActionBarItem {
  id: string;
  icon: string;
  name: string;
}

export const ACTION_BAR_ITEMS: ActionBarItem[] = [
  {
    id: "commit",
    icon: CommitIcon,
    name: "Commit",
  },
  {
    id: "push",
    icon: PushIcon,
    name: "Push",
  },
];
