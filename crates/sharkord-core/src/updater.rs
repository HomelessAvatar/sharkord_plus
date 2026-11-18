use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::SharkordError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReleaseAsset {
    pub name: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub browser_download_url: String,
    #[serde(default)]
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ReleaseInfo {
    #[serde(default)]
    pub tag_name: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub body: String,
    #[serde(default)]
    pub published_at: String,
    #[serde(default)]
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum UpdateStatus {
    #[default]
    Idle,
    Checking,
    UpdateAvailable(ReleaseInfo),
    UpToDate,
    Downloading {
        percent: f32,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    ReadyToInstall {
        installer_path: PathBuf,
        version: String,
    },
    Error(String),
}

impl UpdateStatus {
    pub fn is_update_available(&self) -> bool {
        matches!(self, UpdateStatus::UpdateAvailable(_))
    }

    pub fn release_info(&self) -> Option<&ReleaseInfo> {
        match self {
            UpdateStatus::UpdateAvailable(info) => Some(info),
            _ => None,
        }
    }
}

pub fn parse_semver(version_str: &str) -> Option<(u64, u64, u64)> {
    let trimmed = version_str.trim();
    let without_prefix = if let Some(idx) = trimmed.find('v') {
        &trimmed[idx + 1..]
    } else if let Some(idx) = trimmed.find('V') {
        &trimmed[idx + 1..]
    } else {
        trimmed
    };

    let core_version = without_prefix.split('-').next()?.split('+').next()?;

    let parts: Vec<&str> = core_version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major = parts[0].parse::<u64>().ok()?;
    let minor = parts[1].parse::<u64>().ok()?;
    let patch = parts[2].parse::<u64>().ok()?;

    Some((major, minor, patch))
}

pub fn is_newer_version(current: &str, remote: &str) -> bool {
    match (parse_semver(current), parse_semver(remote)) {
        (Some(c), Some(r)) => r > c,
        _ => false,
    }
}

fn matches_os(name_lower: &str, os_lower: &str) -> bool {
    if os_lower.contains("mac")
        || os_lower.contains("darwin")
        || os_lower.contains("osx")
        || os_lower.contains("apple")
    {
        name_lower.contains("macos")
            || name_lower.contains("darwin")
            || name_lower.contains("mac")
            || name_lower.ends_with(".dmg")
    } else if os_lower == "windows" || os_lower.starts_with("win") {
        name_lower.contains("windows")
            || (name_lower.contains("win") && !name_lower.contains("darwin"))
            || name_lower.ends_with(".msi")
    } else if os_lower.contains("linux") {
        name_lower.contains("linux")
            || name_lower.ends_with(".appimage")
            || name_lower.ends_with(".deb")
    } else {
        name_lower.contains(os_lower)
    }
}

fn is_arch_incompatible(name_lower: &str, arch_lower: &str) -> bool {
    if arch_lower == "x86_64" || arch_lower == "amd64" || arch_lower == "x64" {
        name_lower.contains("aarch64")
            || name_lower.contains("arm64")
            || name_lower.contains("armv7")
    } else if arch_lower == "aarch64" || arch_lower == "arm64" {
        name_lower.contains("x86_64") || name_lower.contains("amd64") || name_lower.contains("x64")
    } else {
        (name_lower.contains("x86_64")
            || name_lower.contains("amd64")
            || name_lower.contains("x64")
            || name_lower.contains("aarch64")
            || name_lower.contains("arm64"))
            && !name_lower.contains(arch_lower)
    }
}

fn matches_arch_explicit(name_lower: &str, arch_lower: &str) -> bool {
    if arch_lower == "x86_64" || arch_lower == "amd64" || arch_lower == "x64" {
        name_lower.contains("x86_64") || name_lower.contains("amd64") || name_lower.contains("x64")
    } else if arch_lower == "aarch64" || arch_lower == "arm64" {
        name_lower.contains("aarch64") || name_lower.contains("arm64")
    } else {
        name_lower.contains(arch_lower)
    }
}

pub fn find_platform_asset<'a>(
    release: &'a ReleaseInfo,
    os: &str,
    arch: &str,
) -> Option<&'a ReleaseAsset> {
    let os_lower = os.to_lowercase();
    let arch_lower = arch.to_lowercase();

    let preferred_extensions: &[&str] = if os_lower.contains("mac")
        || os_lower.contains("darwin")
        || os_lower.contains("osx")
        || os_lower.contains("apple")
    {
        &[".dmg", ".zip", ".tar.gz"]
    } else if os_lower == "windows" || os_lower.starts_with("win") {
        &[".msi", ".zip"]
    } else if os_lower.contains("linux") {
        &[".appimage", ".deb", ".tar.gz"]
    } else {
        &[".zip", ".tar.gz"]
    };

    let candidates: Vec<&'a ReleaseAsset> = release
        .assets
        .iter()
        .filter(|asset| {
            let name_lower = asset.name.to_lowercase();
            matches_os(&name_lower, &os_lower) && !is_arch_incompatible(&name_lower, &arch_lower)
        })
        .collect();

    for ext in preferred_extensions {
        let matching_ext_candidates: Vec<&&'a ReleaseAsset> = candidates
            .iter()
            .filter(|asset| asset.name.to_lowercase().ends_with(ext))
            .collect();

        if let Some(asset) = matching_ext_candidates
            .iter()
            .find(|asset| matches_arch_explicit(&asset.name.to_lowercase(), &arch_lower))
        {
            return Some(**asset);
        }

        if let Some(asset) = matching_ext_candidates.first() {
            return Some(**asset);
        }
    }

    candidates
        .iter()
        .find(|asset| matches_arch_explicit(&asset.name.to_lowercase(), &arch_lower))
        .copied()
        .or_else(|| candidates.first().copied())
}

#[derive(Debug, Clone)]
pub struct UpdaterClient {
    pub repo_owner: String,
    pub repo_name: String,
    pub base_url: String,
    client: reqwest::Client,
}

impl UpdaterClient {
    pub fn new(repo_owner: impl Into<String>, repo_name: impl Into<String>) -> Self {
        Self::with_base_url(repo_owner, repo_name, "https://api.github.com")
    }

    pub fn with_base_url(
        repo_owner: impl Into<String>,
        repo_name: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("sharkord-updater")
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        Self {
            repo_owner: repo_owner.into(),
            repo_name: repo_name.into(),
            base_url: base_url.into(),
            client,
        }
    }

    pub fn default_github() -> Self {
        Self::new("HomelessAvatar", "sharkord_plus")
    }

    pub async fn fetch_latest_release(&self) -> Result<ReleaseInfo, SharkordError> {
        let base = self.base_url.trim_end_matches('/');
        let list_url = format!("{}/repos/{}/{}/releases?per_page=10", base, self.repo_owner, self.repo_name);

        if let Ok(resp) = self.client.get(&list_url).send().await {
            if resp.status().is_success() {
                if let Ok(releases) = resp.json::<Vec<ReleaseInfo>>().await {
                    for mut r in releases {
                        let tag_lower = r.tag_name.to_lowercase();
                        if tag_lower.starts_with("desktop-v") || tag_lower.starts_with('v') {
                            if r.version.is_empty() {
                                if let Some(idx) = r.tag_name.find(['v', 'V']) {
                                    r.version = r.tag_name[idx + 1..].to_string();
                                } else {
                                    r.version = r.tag_name.clone();
                                }
                            }
                            return Ok(r);
                        }
                    }
                }
            }
        }

        let url = format!(
            "{}/repos/{}/{}/releases/latest",
            base,
            self.repo_owner,
            self.repo_name
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            return Err(SharkordError::Network(format!(
                "Failed to fetch release: HTTP {}",
                resp.status()
            )));
        }
        let mut release: ReleaseInfo = resp.json().await?;
        if release.version.is_empty() {
            if let Some(idx) = release.tag_name.find(['v', 'V']) {
                release.version = release.tag_name[idx + 1..].to_string();
            } else {
                release.version = release.tag_name.clone();
            }
        }
        Ok(release)
    }

    pub async fn check_for_update(
        &self,
        current_version: &str,
    ) -> Result<UpdateStatus, SharkordError> {
        let release = self.fetch_latest_release().await?;
        if is_newer_version(current_version, &release.tag_name) {
            Ok(UpdateStatus::UpdateAvailable(release))
        } else {
            Ok(UpdateStatus::UpToDate)
        }
    }
}

impl Default for UpdaterClient {
    fn default() -> Self {
        Self::default_github()
    }
}
