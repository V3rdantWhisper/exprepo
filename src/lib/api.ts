import { invoke } from "@tauri-apps/api/core";

// ---- Types mirroring the Rust models (serde snake_case) ----

export interface AppConfig {
  repo_path: string;
  remote_url: string | null;
  default_branch: string;
  git_name: string;
  git_email: string;
}

export interface CveMeta {
  title: string;
  description: string;
  status: string;
  severity: string;
  tags: string[];
  references: string[];
}

export interface ExpMeta {
  name: string;
  status: string;
  target_env: string;
  notes: string;
}

export interface Exp {
  id: string;
  cve_id: string;
  meta: ExpMeta;
  wps: string[];
  sources: string[];
}

export interface Cve {
  id: string;
  meta: CveMeta;
  exps: Exp[];
}

export interface StatusEntry {
  path: string;
  state: string;
  staged: boolean;
}

export interface RepoStatus {
  branch: string | null;
  remote_url: string | null;
  entries: StatusEntry[];
  ahead: number;
  behind: number;
}

export interface Environment {
  id: string;
  name: string;
  arch: string;
  qemu_binary: string;
  kernel_image: string | null;
  rootfs: string | null;
  append: string | null;
  extra_args: string;
  working_dir: string | null;
  build_guide: string | null;
  created_at: string;
}

export function emptyCveMeta(): CveMeta {
  return { title: "", description: "", status: "todo", severity: "", tags: [], references: [] };
}

export function emptyExpMeta(): ExpMeta {
  return { name: "", status: "todo", target_env: "", notes: "" };
}

export function emptyEnvironment(): Environment {
  return {
    id: "",
    name: "",
    arch: "x86_64",
    qemu_binary: "",
    kernel_image: null,
    rootfs: null,
    append: null,
    extra_args: "-nographic -m 1G",
    working_dir: null,
    build_guide: "",
    created_at: "",
  };
}

// ---- Config / repo ----

export const getConfig = () => invoke<AppConfig>("get_config");
export const setRepoPath = (path: string) => invoke<AppConfig>("set_repo_path", { path });
export const repoInit = () => invoke<void>("repo_init");
export const repoClone = (url: string) => invoke<void>("repo_clone", { url });
export const repoStatus = () => invoke<RepoStatus>("repo_status");
export const repoCommit = (message: string) => invoke<string>("repo_commit", { message });
export const repoPush = () => invoke<void>("repo_push");
export const repoPull = () => invoke<void>("repo_pull");
export const repoGetRemote = () => invoke<string | null>("repo_get_remote");
export const repoSetRemote = (url: string) => invoke<void>("repo_set_remote", { url });

// ---- CVE / exp / wp ----

export const cveList = () => invoke<Cve[]>("cve_list");
export const cveGet = (cveId: string) => invoke<Cve>("cve_get", { cveId });
export const cveCreate = (cveId: string, meta: CveMeta) =>
  invoke<void>("cve_create", { cveId, meta });
export const cveUpdateMeta = (cveId: string, meta: CveMeta) =>
  invoke<void>("cve_update_meta", { cveId, meta });
export const expCreate = (cveId: string, expId: string, meta: ExpMeta) =>
  invoke<void>("exp_create", { cveId, expId, meta });
export const expUpdateMeta = (cveId: string, expId: string, meta: ExpMeta) =>
  invoke<void>("exp_update_meta", { cveId, expId, meta });
export const wpCreate = (cveId: string, expId: string, filename: string) =>
  invoke<string>("wp_create", { cveId, expId, filename });
export const fileRead = (path: string) => invoke<string>("file_read", { path });
export const fileWrite = (path: string, content: string) =>
  invoke<void>("file_write", { path, content });

// ---- Environments ----

export const envList = () => invoke<Environment[]>("env_list");
export const envRunning = () => invoke<string[]>("env_running");
export const envSave = (env: Environment) => invoke<Environment>("env_save", { env });
export const envDelete = (id: string) => invoke<void>("env_delete", { id });
export const envLaunch = (id: string) => invoke<void>("env_launch", { id });
export const envStop = (id: string) => invoke<void>("env_stop", { id });
export const envExportGuide = (id: string) => invoke<string>("env_export_guide", { id });

// ---- Secrets ----

export const secretSetToken = (token: string) => invoke<void>("secret_set_token", { token });
export const secretHasToken = () => invoke<boolean>("secret_has_token");
export const secretDeleteToken = () => invoke<void>("secret_delete_token");
