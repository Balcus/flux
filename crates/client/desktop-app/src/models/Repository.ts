import { Branch } from "./Branch";
import { StatusInfo } from "./StatusInfo";

export interface Repository {
  path: string;
  branches: Branch[];
  head: string;
  index: string[];
  uncommited: string[];
  user_name: string | null;
  user_email: string | null;
  origin: string | null;
  status: StatusInfo;
}