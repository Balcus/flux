import { ChangeType } from "./ChangeType";

export interface StagedFile {
    path: string,
    change_type: ChangeType
}