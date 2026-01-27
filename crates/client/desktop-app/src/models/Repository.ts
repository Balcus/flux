import { Branch } from "./Branch";

export interface Repository {
    path: string,
    branches: Branch[],
    head: string,
    index: string[],
    uncommited: string[]
}