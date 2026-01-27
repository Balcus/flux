import { Branch } from "./Branch";
import { Repository } from "./Repository";

export type RepositoryAction =
  | { type: 'LOADING_START' }
  | { type: 'LOADING_SUCCESS'; payload: Repository }
  | { type: 'LOADING_ERROR'; payload: string }
  | { type: 'CLOSE_REPOSITORY' }
  | { type: 'UPDATE_BRANCHES'; payload: Branch[] }
  | { type: 'UPDATE_INDEX'; payload: string[] }
  | { type: 'CLEAR_ERROR' };
