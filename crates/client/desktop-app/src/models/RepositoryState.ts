import { Repository } from "./Repository";

export interface RepositoryState {
    repository: Repository | null;
    isLoading: boolean;
    error: string | null;
}