use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use crate::error::SharkordError;

/// Persistent session information for a connected Sharkord server.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SavedServerSession {
    pub server_id: String,
    pub server_name: String,
    pub server_url: String,
    pub token: String,
    pub user_id: u64,
    pub username: String,
    pub display_name: String,
    #[serde(default)]
    pub avatar_url: Option<String>,
    pub last_accessed_at: u64,
}

/// Conversion trait to allow flexible server identifiers in `set_active`.
pub trait IntoActiveServerId {
    fn into_active_id(self) -> Option<String>;
}

impl IntoActiveServerId for Option<String> {
    fn into_active_id(self) -> Option<String> {
        self
    }
}

impl IntoActiveServerId for Option<&str> {
    fn into_active_id(self) -> Option<String> {
        self.map(|s| s.to_string())
    }
}

impl IntoActiveServerId for String {
    fn into_active_id(self) -> Option<String> {
        Some(self)
    }
}

impl IntoActiveServerId for &str {
    fn into_active_id(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl IntoActiveServerId for &String {
    fn into_active_id(self) -> Option<String> {
        Some(self.clone())
    }
}

/// Multi-server session storage manager with atomic disk persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionVault {
    #[serde(skip)]
    path: PathBuf,
    pub active_server_id: Option<String>,
    #[serde(default)]
    pub servers: Vec<SavedServerSession>,
}

impl Default for SessionVault {
    fn default() -> Self {
        Self::new(Self::default_path())
    }
}

impl SessionVault {
    /// Create a new session vault targeting the given file path.
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            active_server_id: None,
            servers: Vec::new(),
        }
    }

    /// Default configuration path for the current operating system.
    pub fn default_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            if let Ok(appdata) = std::env::var("APPDATA") {
                return PathBuf::from(appdata).join("sharkord").join("vault.json");
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home)
                    .join("Library")
                    .join("Application Support")
                    .join("sharkord")
                    .join("vault.json");
            }
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            if let Ok(config_home) = std::env::var("XDG_CONFIG_HOME") {
                return PathBuf::from(config_home).join("sharkord").join("vault.json");
            }
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home).join(".config").join("sharkord").join("vault.json");
            }
        }
        PathBuf::from("sharkord_vault.json")
    }

    /// Returns the path to the vault storage file.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Sets the path to the vault storage file.
    pub fn set_path(&mut self, path: impl Into<PathBuf>) {
        self.path = path.into();
    }

    /// Load and deserialize the session vault from the specified file path.
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, SharkordError> {
        let path_ref = path.as_ref();
        let content = std::fs::read_to_string(path_ref)?;
        let mut vault: SessionVault = serde_json::from_str(&content)?;
        vault.path = path_ref.to_path_buf();
        Ok(vault)
    }

    /// Reload the session vault from its configured file path.
    pub fn load(&mut self) -> Result<(), SharkordError> {
        let loaded = Self::load_from_path(&self.path)?;
        self.active_server_id = loaded.active_server_id;
        self.servers = loaded.servers;
        Ok(())
    }

    /// Save the session vault state atomically to disk.
    pub fn save(&self) -> Result<(), SharkordError> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let json = serde_json::to_string_pretty(self)?;

        let parent_dir = self
            .path
            .parent()
            .unwrap_or_else(|| Path::new("."));

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);

        let tmp_path = parent_dir.join(format!(
            ".vault_{}_{}_{}.tmp",
            std::process::id(),
            nonce,
            rand_fast()
        ));

        // Write and sync temporary file
        let write_res = (|| -> Result<(), SharkordError> {
            let mut file = std::fs::File::create(&tmp_path)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(err) = write_res {
            let _ = std::fs::remove_file(&tmp_path);
            return Err(err);
        }

        // Atomically rename temporary file to destination path
        if let Err(err) = std::fs::rename(&tmp_path, &self.path) {
            // Windows fallback when destination file exists
            if self.path.exists() {
                let _ = std::fs::remove_file(&self.path);
                if let Err(retry_err) = std::fs::rename(&tmp_path, &self.path) {
                    let _ = std::fs::remove_file(&tmp_path);
                    return Err(SharkordError::Io(retry_err));
                }
            } else {
                let _ = std::fs::remove_file(&tmp_path);
                return Err(SharkordError::Io(err));
            }
        }

        Ok(())
    }

    /// Adds a new server session or updates an existing one by `server_id`.
    /// Optionally marks the server as the active session.
    pub fn add_or_update(
        &mut self,
        session: SavedServerSession,
        set_active: bool,
    ) -> Result<(), SharkordError> {
        let server_id = session.server_id.clone();
        if let Some(existing) = self
            .servers
            .iter_mut()
            .find(|s| s.server_id == server_id)
        {
            *existing = session;
        } else {
            self.servers.push(session);
        }

        if set_active {
            self.active_server_id = Some(server_id);
        }

        self.save()
    }

    /// Removes a server session by `server_id`.
    /// If the removed server was active, selects the next available server or `None`.
    pub fn remove(&mut self, server_id: &str) -> Result<Option<SavedServerSession>, SharkordError> {
        let pos = self.servers.iter().position(|s| s.server_id == server_id);
        let removed = match pos {
            Some(idx) => Some(self.servers.remove(idx)),
            None => None,
        };

        if self.active_server_id.as_deref() == Some(server_id) {
            self.active_server_id = self.servers.first().map(|s| s.server_id.clone());
        }

        if removed.is_some() {
            self.save()?;
        }

        Ok(removed)
    }

    /// Returns a reference to the active server session, if set and present.
    pub fn get_active(&self) -> Option<&SavedServerSession> {
        let active_id = self.active_server_id.as_deref()?;
        self.servers.iter().find(|s| s.server_id == active_id)
    }

    /// Returns a mutable reference to the active server session, if set and present.
    pub fn get_active_mut(&mut self) -> Option<&mut SavedServerSession> {
        let active_id = self.active_server_id.as_deref()?;
        self.servers.iter_mut().find(|s| s.server_id == active_id)
    }

    /// Sets the active server identifier, or clears it if `None`.
    pub fn set_active(&mut self, server_id: impl IntoActiveServerId) -> Result<(), SharkordError> {
        let id_opt = server_id.into_active_id();
        if let Some(ref id) = id_opt {
            if !self.servers.iter().any(|s| &s.server_id == id) {
                return Err(SharkordError::NotFound(format!(
                    "Server '{id}' not found in session vault"
                )));
            }
        }
        self.active_server_id = id_opt;
        self.save()
    }

    /// Clears the active server selection.
    pub fn clear_active(&mut self) -> Result<(), SharkordError> {
        self.active_server_id = None;
        self.save()
    }

    /// Finds a saved server session by its `server_id`.
    pub fn get_server(&self, server_id: &str) -> Option<&SavedServerSession> {
        self.servers.iter().find(|s| s.server_id == server_id)
    }

    /// Finds a mutable saved server session by its `server_id`.
    pub fn get_server_mut(&mut self, server_id: &str) -> Option<&mut SavedServerSession> {
        self.servers.iter_mut().find(|s| s.server_id == server_id)
    }

    /// Clears all server sessions and resets the active server pointer.
    pub fn clear(&mut self) -> Result<(), SharkordError> {
        self.servers.clear();
        self.active_server_id = None;
        self.save()
    }
}

/// Helper to generate pseudo-random digits without heavy dependencies.
fn rand_fast() -> u32 {
    use std::sync::atomic::{AtomicU32, Ordering};
    static COUNTER: AtomicU32 = AtomicU32::new(1);
    COUNTER.fetch_add(1, Ordering::Relaxed)
}
