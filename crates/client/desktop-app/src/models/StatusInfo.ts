import { StagedFile } from "./StagedFile";

export interface StatusInfo {
    untracked: string[],
    staged: StagedFile[],
}