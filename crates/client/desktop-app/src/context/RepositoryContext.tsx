import {
  createContext,
  ReactNode,
  useContext,
  useEffect,
  useReducer,
} from "react";
import { RepositoryAction } from "../models/RepositoryAction";
import { RepositoryState } from "../models/RepositoryState";
import { invoke } from "@tauri-apps/api/core";
import { Repository } from "../models/Repository";
import { open } from "@tauri-apps/plugin-dialog";

interface RepositoryContextType extends RepositoryState {
  openRepository: () => Promise<void>;
  closeRepository: () => void;
  refreshRepository: () => Promise<void>;
  clearError: () => void;
}

function repositoryReducer(
  state: RepositoryState,
  action: RepositoryAction
): RepositoryState {
  switch (action.type) {
    case "LOADING_START":
      return {
        ...state,
        isLoading: true,
        error: null,
      };

    case "LOADING_SUCCESS":
      return {
        repository: action.payload,
        isLoading: false,
        error: null,
      };

    case "LOADING_ERROR":
      return {
        ...state,
        isLoading: false,
        error: action.payload,
      };

    case "CLOSE_REPOSITORY":
      return {
        repository: null,
        isLoading: false,
        error: null,
      };

    case "UPDATE_BRANCHES":
      if (!state.repository) return state;
      return {
        ...state,
        repository: {
          ...state.repository,
          branches: action.payload,
        },
      };

    case "UPDATE_INDEX":
      if (!state.repository) return state;
      return {
        ...state,
        repository: {
          ...state.repository,
          index: action.payload,
        },
      };

    case "CLEAR_ERROR":
      return {
        ...state,
        error: null,
      };

    default:
      return state;
  }
}

const RepositoryContext = createContext<RepositoryContextType | undefined>(
  undefined
);
const STORAGE_KEY = "flux_last_repository";

const initialState: RepositoryState = {
  repository: null,
  isLoading: true,
  error: null,
};

export function RepositoryProvider({ children }: { children: ReactNode }) {
  const [state, dispatch] = useReducer(repositoryReducer, initialState);

  useEffect(() => {
    const loadLastRepository = async () => {
      try {
        const lastRepoPath = localStorage.getItem(STORAGE_KEY);
        if (lastRepoPath) {
          await loadRepository(lastRepoPath);
        } else {
          dispatch({ type: "LOADING_ERROR", payload: "" });
        }
      } catch (err) {
        console.error("Failed to load last repository:", err);
        localStorage.removeItem(STORAGE_KEY);
        dispatch({
          type: "LOADING_ERROR",
          payload: "Failed to load previous repository",
        });
      }
    };

    loadLastRepository();
  }, []);

  const loadRepository = async (path: string) => {
    try {
      dispatch({ type: "LOADING_START" });
      const repo = await invoke<Repository>("open_repository", { path });
      dispatch({ type: "LOADING_SUCCESS", payload: repo });
      localStorage.setItem(STORAGE_KEY, path);
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      dispatch({ type: "LOADING_ERROR", payload: errorMessage });
      throw err;
    }
  };

  const openRepository = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Open Flux Repository",
      });

      if (selected) {
        await loadRepository(selected);
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : String(err);
      dispatch({ type: "LOADING_ERROR", payload: errorMessage });
    }
  };

  const closeRepository = () => {
    dispatch({ type: "CLOSE_REPOSITORY" });
    localStorage.removeItem(STORAGE_KEY);
  };

  const refreshRepository = async () => {
    if (!state.repository) return;

    try {
      await loadRepository(state.repository.path);
    } catch (err) {
      console.error("Failed to refresh repository:", err);
    }
  };

  const clearError = () => {
    dispatch({ type: "CLEAR_ERROR" });
  };

  return (
    <RepositoryContext.Provider
      value={{
        ...state,
        openRepository,
        closeRepository,
        refreshRepository,
        clearError,
      }}
    >
      {children}
    </RepositoryContext.Provider>
  );
}

export function useRepository() {
  const context = useContext(RepositoryContext);
  if (context === undefined) {
    throw new Error("useRepository must be used within a RepositoryProvider");
  }
  return context;
}
