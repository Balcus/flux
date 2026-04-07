export interface Commit {
  id: string;
  message: string;
  author: string;
  parent: string | null;
  branch: string;
}
