export interface GraphNode {
  id: string;
  short_id: string;
  message: string;
  author: string;
  branches: string[];
  parents: string[];
  is_merge: boolean;
}
