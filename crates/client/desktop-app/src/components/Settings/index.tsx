import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import './Settings.css';
import { Repository } from "../../models/Repository";

interface SettingsProps {
  path: string;
}

export default function Settings({ path }: SettingsProps) {
  const [username, setUsername] = useState<string>("");
  const [email, setEmail] = useState<string>("");
  const [origin, setOrigin] = useState<string>("");
  const [isSaving, setIsSaving] = useState<boolean>(false);

  useEffect(() => {
    if (path) {
      invoke<Repository>("open_repository", { path })
        .then((info) => {
          setUsername(info.user_name ?? "");
          setEmail(info.user_email ?? "");
          setOrigin(info.origin ?? "");
        })
        .catch((err: string) => console.error(err));
    }
  }, [path]);

  const handleSave = async (): Promise<void> => {
    setIsSaving(true);
    try {
      await invoke<Repository>("update_settings", {
        path,
        user_name: username,
        user_email: email,
        origin: origin,
      });
    } catch (err) {
      console.error(err);
    } finally {
      setIsSaving(false);
    }
  };

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
            />
            <p className="description">
              This name will be used when creating commits as well as logging
              into remote repositories.
            </p>
          </div>

          <div className="setting-item">
            <label>Email</label>
            <input
              type="email"
              value={email}
              onChange={(e) => setEmail(e.target.value)}
            />
            <p className="description">
              This email will be used when creating commits as well as logging
              into remote repositories.
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
            />
            <p className="description">
              The primary remote server address for this current workspace.
            </p>
          </div>
        </section>

        <button 
          className="save-button" 
          onClick={handleSave}
          disabled={isSaving || !path}
        >
          {isSaving ? "Saving..." : "Apply Changes"}
        </button>
      </div>
    </div>
  );
}