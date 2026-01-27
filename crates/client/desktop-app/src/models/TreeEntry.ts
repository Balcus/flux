export interface TreeEntry {
    hash: string;
    mode: string;
    type: "blob" | "tree";
}