import { StagedFile } from "./StagedFile";

export interface StatusInfo {
  staged: StagedFile[];
  unstaged: StagedFile[];
  untracked: string[];
}
