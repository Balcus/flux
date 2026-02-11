import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import "./Settings.css";
import { useRepository } from "../../context/RepositoryContext";
import { toast } from "react-toastify";

export default function Settings() {
  const { repository, refreshRepository } = useRepository();
  const [username, setUsername] = useState<string>("");
  const [email, setEmail] = useState<string>("");
  const [origin, setOrigin] = useState<string>("");
  const [isSaving, setIsSaving] = useState<boolean>(false);

  useEffect(() => {
    if (repository) {
      setUsername(repository.user_name ?? "");
      setEmail(repository.user_email ?? "");
      setOrigin(repository.origin ?? "");
    }
  }, [repository]);

  const handleSave = async (): Promise<void> => {
    if (!repository) return;
    setIsSaving(true);

    try {
      await invoke("update_user_config", { userName: username, userEmail: email });
      await invoke("update_origin", { origin });
      await refreshRepository();
      
      toast.success("Settings saved successfully!");
    } catch (e) {
      const error = e instanceof Error ? e.message : String(e);
      
      toast.error(
        <div>
          <div style={{ fontWeight: '700', fontSize: '14px', marginBottom: '2px' }}>
            Failed to save settings
          </div>
          <div style={{ fontSize: '12px', opacity: 0.8, lineHeight: '1.4' }}>
            {error}
          </div>
        </div>
      );
    } finally {
      setIsSaving(false);
    }
  };

  if (!repository) return <div>No repository loaded</div>;

  return (
    <div className="settings-container">
      <div className="settings-hero">
        <h1>Settings</h1>
        <p>Manage repository configuration and user identity.</p>
      </div>
      <div className="settings-body">
        <section className="settings-section">
          <h2>User Credentials</h2>
          <div className="setting-item">
            <label>Username</label>
            <input
              type="text"
              value={username}
              onChange={(e) => setUsername(e.target.value)}
              placeholder="Enter your username"
            />
            <p className="description">
              This name will be used when creating commitsand authenticating with the reomote server.
            </p>
          </div>
          <div className="setting-item">
            <label>Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
              placeholder="Enter your email"
            />
            <p className="description">
              This email will be used when creating commits and authenticating with the reomote server.
            </p>
          </div>
        </section>

        <section className="settings-section">
          <h2>Repository Origin</h2>
          <div className="setting-item">
            <label>Remote URL</label>
            <input
              type="text"
              value={origin}
              onChange={(e) => setOrigin(e.target.value)}
              placeholder="https://github.com/user/repo.git"
            />
            <p className="description">
              The primary remote server address for this workspace.
            </p>
          </div>
        </section>

        <button className="save-button" onClick={handleSave} disabled={isSaving}>
          {isSaving ? "Saving..." : "Apply Changes"}
        </button>
      </div>
    </div>
  );
}