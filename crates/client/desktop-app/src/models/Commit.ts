import { Tree } from "./Tree";

export interface Commit {
    hash: string,
    authorName: string,
    timestamp: Date,
    tree: Tree;
}