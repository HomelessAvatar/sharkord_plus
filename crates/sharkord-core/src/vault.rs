use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};

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

/// Server profile representation matching Tauri IPC and client schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServerProfile {
    pub id: String,
    pub name: String,
    pub server_url: String,
    pub token: String,
    #[serde(default)]
    pub user: Option<crate::types::User>,
}

impl From<&SavedServerSession> for ServerProfile {
    fn from(s: &SavedServerSession) -> Self {
        ServerProfile {
            id: s.server_id.clone(),
            name: s.server_name.clone(),
            server_url: s.server_url.clone(),
            token: s.token.clone(),
            user: if s.user_id > 0 || !s.username.is_empty() {
                Some(crate::types::User {
                    id: s.user_id,
                    name: s.username.clone(),
                    identity: Some(s.display_name.clone()),
                    ..Default::default()
                })
            } else {
                None
            },
        }
    }
}

impl From<&ServerProfile> for SavedServerSession {
    fn from(p: &ServerProfile) -> Self {
        SavedServerSession {
            server_id: p.id.clone(),
            server_name: p.name.clone(),
            server_url: p.server_url.clone(),
            token: p.token.clone(),
            user_id: p.user.as_ref().map(|u| u.id).unwrap_or(0),
            username: p.user.as_ref().map(|u| u.name.clone()).unwrap_or_default(),
            display_name: p
                .user
                .as_ref()
                .and_then(|u| u.identity.clone())
                .unwrap_or_else(|| p.name.clone()),
            avatar_url: None,
            last_accessed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
        }
    }
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

fn default_true() -> bool {
    true
}

/// Desktop-specific persistent client settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesktopSettings {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_true")]
    pub close_to_tray: bool,
    #[serde(default)]
    pub start_to_tray: bool,
    #[serde(default = "default_true")]
    pub dynamic_tray_icon: bool,
    #[serde(default = "default_true")]
    pub gpu_encoding: bool,
}

impl Default for DesktopSettings {
    fn default() -> Self {
        Self {
            auto_start: false,
            close_to_tray: true,
            start_to_tray: false,
            dynamic_tray_icon: true,
            gpu_encoding: true,
        }
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
    #[serde(default)]
    pub profiles: Vec<ServerProfile>,
    #[serde(default)]
    pub desktop_settings: DesktopSettings,
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
            profiles: Vec::new(),
            desktop_settings: DesktopSettings::default(),
        }
    }

    /// Alias for `new` allowing fluent construction with an explicit path.
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self::new(path)
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
                return PathBuf::from(config_home)
                    .join("sharkord")
                    .join("vault.json");
            }
            if let Ok(home) = std::env::var("HOME") {
                return PathBuf::from(home)
                    .join(".config")
                    .join("sharkord")
                    .join("vault.json");
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
        if vault.profiles.is_empty() && !vault.servers.is_empty() {
            vault.profiles = vault.servers.iter().map(ServerProfile::from).collect();
        } else if vault.servers.is_empty() && !vault.profiles.is_empty() {
            vault.servers = vault.profiles.iter().map(SavedServerSession::from).collect();
        }
        Ok(vault)
    }

    /// Reload the session vault from its configured file path.
    pub fn load(&mut self) -> Result<(), SharkordError> {
        let loaded = Self::load_from_path(&self.path)?;
        self.active_server_id = loaded.active_server_id;
        self.servers = loaded.servers;
        self.profiles = loaded.profiles;
        self.desktop_settings = loaded.desktop_settings;
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

        let parent_dir = self.path.parent().unwrap_or_else(|| Path::new("."));

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
        if let Some(existing) = self.servers.iter_mut().find(|s| s.server_id == server_id) {
            *existing = session.clone();
        } else {
            self.servers.push(session.clone());
        }

        let profile = ServerProfile::from(&session);
        if let Some(existing_p) = self.profiles.iter_mut().find(|p| p.id == server_id) {
            *existing_p = profile;
        } else {
            self.profiles.push(profile);
        }

        if set_active {
            self.active_server_id = Some(server_id);
        }

        self.save()
    }

    /// Saves or updates a server profile, and makes it active if no active server is set.
    pub fn save_profile(&mut self, profile: ServerProfile) -> Result<(), SharkordError> {
        let id = profile.id.clone();
        if let Some(existing) = self.profiles.iter_mut().find(|p| p.id == id) {
            *existing = profile.clone();
        } else {
            self.profiles.push(profile.clone());
        }

        let session = SavedServerSession::from(&profile);
        if let Some(existing_s) = self.servers.iter_mut().find(|s| s.server_id == id) {
            *existing_s = session;
        } else {
            self.servers.push(session);
        }

        if self.active_server_id.is_none() {
            self.active_server_id = Some(id);
        }

        self.save()
    }

    /// Finds a saved server profile by id.
    pub fn get_profile(&self, id: &str) -> Option<&ServerProfile> {
        self.profiles.iter().find(|p| p.id == id)
    }

    /// Returns the active server profile, if set and present.
    pub fn get_active_profile(&self) -> Option<&ServerProfile> {
        let active_id = self.active_server_id.as_deref()?;
        self.profiles.iter().find(|p| p.id == active_id)
    }

    /// Removes a server profile by id.
    pub fn remove_profile(&mut self, id: &str) -> Result<Option<ServerProfile>, SharkordError> {
        let pos = self.profiles.iter().position(|p| p.id == id);
        let removed = match pos {
            Some(idx) => Some(self.profiles.remove(idx)),
            None => None,
        };

        let pos_s = self.servers.iter().position(|s| s.server_id == id);
        if let Some(idx_s) = pos_s {
            self.servers.remove(idx_s);
        }

        if self.active_server_id.as_deref() == Some(id) {
            self.active_server_id = self.profiles.first().map(|p| p.id.clone());
        }

        if removed.is_some() {
            self.save()?;
        }

        Ok(removed)
    }

    /// Returns all saved server profiles.
    pub fn list_profiles(&self) -> Vec<ServerProfile> {
        self.profiles.clone()
    }

    /// Removes a server session by `server_id`.
    /// If the removed server was active, selects the next available server or `None`.
    pub fn remove(&mut self, server_id: &str) -> Result<Option<SavedServerSession>, SharkordError> {
        let pos = self.servers.iter().position(|s| s.server_id == server_id);
        let removed = match pos {
            Some(idx) => Some(self.servers.remove(idx)),
            None => None,
        };

        let pos_p = self.profiles.iter().position(|p| p.id == server_id);
        if let Some(idx_p) = pos_p {
            self.profiles.remove(idx_p);
        }

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

    /// Returns a reference to the desktop settings.
    pub fn get_desktop_settings(&self) -> &DesktopSettings {
        &self.desktop_settings
    }

    /// Updates the desktop settings and persists the vault.
    pub fn update_desktop_settings(
        &mut self,
        settings: DesktopSettings,
    ) -> Result<(), SharkordError> {
        self.desktop_settings = settings;
        self.save()
    }

    /// Clears all server sessions and resets the active server pointer.
    pub fn clear(&mut self) -> Result<(), SharkordError> {
        self.servers.clear();
        self.profiles.clear();
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
