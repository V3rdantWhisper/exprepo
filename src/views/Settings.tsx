import { Show, createSignal, type Component } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { FolderOpen } from "lucide-solid";
import * as api from "../lib/api";
import { state, refreshConfig, refreshCves, refreshStatus, reportError, toast } from "../lib/store";

const Settings: Component = () => {
  const [repoPath, setRepoPath] = createSignal(state.config?.repo_path ?? "");
  const [remote, setRemote] = createSignal(state.config?.remote_url ?? "");
  const [cloneUrl, setCloneUrl] = createSignal("");
  const [token, setToken] = createSignal("");

  const browse = async () => {
    const picked = await open({ directory: true, multiple: false });
    if (typeof picked === "string") setRepoPath(picked);
  };

  const wrap = (fn: () => Promise<unknown>, ok: string) => async () => {
    try {
      await fn();
      toast(ok);
    } catch (e) {
      reportError(e);
    }
  };

  const setPath = wrap(async () => {
    await api.setRepoPath(repoPath());
    await refreshConfig();
    await refreshStatus();
    await refreshCves();
  }, "Repo path updated");

  const initRepo = wrap(async () => {
    await api.repoInit();
    await refreshStatus();
    await refreshCves();
  }, "Repository initialized");

  const doClone = wrap(async () => {
    await api.repoClone(cloneUrl());
    await refreshConfig();
    await refreshStatus();
    await refreshCves();
  }, "Repository cloned");

  const setRemoteUrl = wrap(async () => {
    await api.repoSetRemote(remote());
    await refreshStatus();
  }, "Remote updated");

  const saveToken = wrap(async () => {
    await api.secretSetToken(token());
    setToken("");
    await refreshConfig();
  }, "Token saved to keyring");

  const removeToken = wrap(async () => {
    await api.secretDeleteToken();
    await refreshConfig();
  }, "Token removed");

  return (
    <div class="form-doc">
      <h2>Settings</h2>

      <section class="settings-group">
        <h3>Repository</h3>
        <label class="field">
          <span>Local repo path</span>
          <div class="field-row">
            <input value={repoPath()} onInput={(e) => setRepoPath(e.currentTarget.value)} />
            <button class="btn" onClick={browse}>
              <FolderOpen size={14} />
            </button>
          </div>
        </label>
        <div class="form-actions">
          <button class="btn" onClick={setPath}>Set path</button>
          <button class="btn" onClick={initRepo}>Init repo here</button>
        </div>
      </section>

      <section class="settings-group">
        <h3>GitHub sync</h3>
        <label class="field">
          <span>Remote URL (https)</span>
          <input
            value={remote()}
            placeholder="https://github.com/user/repo.git"
            onInput={(e) => setRemote(e.currentTarget.value)}
          />
        </label>
        <div class="form-actions">
          <button class="btn" onClick={setRemoteUrl}>Set remote</button>
        </div>

        <label class="field">
          <span>Clone into repo path</span>
          <input
            value={cloneUrl()}
            placeholder="https://github.com/user/repo.git"
            onInput={(e) => setCloneUrl(e.currentTarget.value)}
          />
        </label>
        <div class="form-actions">
          <button class="btn" onClick={doClone}>Clone</button>
        </div>

        <label class="field">
          <span>
            GitHub token (PAT){" "}
            <Show when={state.hasToken} fallback={<em class="muted">not set</em>}>
              <em class="ok">stored</em>
            </Show>
          </span>
          <input
            type="password"
            value={token()}
            placeholder="ghp_..."
            onInput={(e) => setToken(e.currentTarget.value)}
          />
        </label>
        <div class="form-actions">
          <button class="btn btn-primary" onClick={saveToken}>Save token</button>
          <button class="btn btn-danger" onClick={removeToken}>Remove token</button>
        </div>
        <p class="form-hint">
          The token is stored in your OS keyring and used for push/pull. Use a fine-grained PAT
          with <code>contents: read/write</code> on the target repo.
        </p>
      </section>

      <section class="settings-group">
        <h3>Identity</h3>
        <p class="form-hint">
          Commits authored as <code>{state.config?.git_name}</code> &lt;{state.config?.git_email}&gt;
          on branch <code>{state.config?.default_branch}</code>.
        </p>
      </section>
    </div>
  );
};

export default Settings;
