import { Commit } from "./Commit";

export interface Branch {
    name: string;
    commits: Commit[];
    is_current: boolean;
}