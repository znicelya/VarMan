export type Scope = "user" | "system";

export interface EnvVar {
  name: string;
  value: string;
  scope: Scope;
  isExpandable: boolean;
  rawValue: string | null;
  source: string | null;
}

export interface PathEntry {
  path: string;
  exists: boolean;
  isDuplicate: boolean;
}

export type Change =
  | { type: "add"; value: EnvVar }
  | { type: "modify"; value: EnvVar }
  | { type: "remove"; value: EnvVar };

export interface EnvDiff {
  added: EnvVar[];
  modified: [EnvVar, EnvVar][];
  removed: EnvVar[];
}

export interface BackupInfo {
  path: string;
  scope: Scope;
  createdAt: string;
}

export interface EnvError {
  code: string;
  message: string;
}
