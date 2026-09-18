use crate::paths;
use gpui::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const REPO_OWNER: &str = "ToonionOfficial";
const REPO_NAME: &str = "tnotes";
const COOLDOWN_SECONDS: u64 = 4 * 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    Stable,
    Beta,
    Alpha,
}

impl ReleaseChannel {
    pub const ALL: [Self; 3] = [Self::Stable, Self::Beta, Self::Alpha];

    pub fn default_for_version(version_str: &str) -> Self {
        let version_lower = version_str.to_ascii_lowercase();
        if version_lower.contains("alpha") {
            Self::Alpha
        } else if version_lower.contains("beta") || version_lower.contains("rc") {
            Self::Beta
        } else {
            Self::Stable
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Self::Stable => "Stable",
            Self::Beta => "Beta",
            Self::Alpha => "Alpha",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Stable => "Thoroughly tested releases for maximum reliability.",
            Self::Beta => "Preview upcoming features and bug fixes before general release.",
            Self::Alpha => "Bleeding-edge experimental builds for testing.",
        }
    }

    pub fn allows_version(self, version_str: &str) -> bool {
        let version_lower = version_str.to_ascii_lowercase();
        let is_alpha = version_lower.contains("alpha");
        let is_beta = version_lower.contains("beta");
        let is_rc = version_lower.contains("rc");
        let is_prerelease = is_alpha || is_beta || is_rc;

        match self {
            Self::Stable => !is_prerelease,
            Self::Beta => !is_alpha,
            Self::Alpha => true,
        }
    }
}

impl Default for ReleaseChannel {
    fn default() -> Self {
        Self::default_for_version(env!("CARGO_PKG_VERSION"))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UpdateSettings {
    pub check_on_launch: bool,
    pub auto_download: bool,
    #[serde(default)]
    pub channel: ReleaseChannel,
    pub last_checked: Option<u64>,
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            check_on_launch: true,
            auto_download: false,
            channel: ReleaseChannel::default(),
            last_checked: None,
        }
    }
}

impl UpdateSettings {
    pub fn load() -> Self {
        let settings_path = paths::update_settings_file();
        if let Ok(content) = std::fs::read_to_string(&settings_path)
            && let Ok(parsed) = serde_json::from_str::<Self>(&content)
        {
            return parsed;
        }
        Self::default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let settings_path = paths::update_settings_file();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(settings_path, content)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ReleaseManifest {
    pub version: String,
    pub tag: String,
    pub pub_date: String,
    pub changelog_url: Option<String>,
    #[serde(default)]
    pub platforms: HashMap<String, PlatformAsset>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PlatformAsset {
    pub name: String,
    pub url: String,
    pub sha256: String,
    pub size: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate {
        checked_at: u64,
    },
    Available {
        version: String,
        tag: String,
        download_url: String,
        sha256: String,
        size_bytes: u64,
        changelog_url: Option<String>,
    },
    Downloading {
        version: String,
        progress: f32,
        downloaded_bytes: u64,
        total_bytes: u64,
    },
    Verifying,
    ReadyToRestart {
        version: String,
        staged_path: PathBuf,
        changelog_url: Option<String>,
    },
    Error(String),
}

#[derive(Debug)]
enum DownloadMessage {
    Progress { downloaded: u64, total: u64 },
    Finished(std::result::Result<PathBuf, String>),
}

pub struct UpdateManager {
    settings: UpdateSettings,
    status: UpdateStatus,
    banner_dismissed: bool,
    _task: Option<Task<()>>,
}

impl UpdateManager {
    pub fn new(_cx: &mut Context<Self>) -> Self {
        Self {
            settings: UpdateSettings::load(),
            status: UpdateStatus::Idle,
            banner_dismissed: false,
            _task: None,
        }
    }

    pub fn status(&self) -> &UpdateStatus {
        &self.status
    }

    pub fn settings(&self) -> &UpdateSettings {
        &self.settings
    }

    pub fn is_banner_visible(&self) -> bool {
        if self.banner_dismissed {
            return false;
        }
        matches!(
            self.status,
            UpdateStatus::Available { .. } | UpdateStatus::ReadyToRestart { .. }
        )
    }

    pub fn dismiss_banner(&mut self, cx: &mut Context<Self>) {
        self.banner_dismissed = true;
        cx.notify();
    }

    pub fn set_channel(&mut self, channel: ReleaseChannel, cx: &mut Context<Self>) {
        self.settings.channel = channel;
        if let Err(error) = self.settings.save() {
            eprintln!("Failed to persist update settings: {error}");
        }
        cx.notify();
        self.check_for_updates(false, cx);
    }

    pub fn set_check_on_launch(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.settings.check_on_launch = enabled;
        if let Err(error) = self.settings.save() {
            eprintln!("Failed to persist update settings: {error}");
        }
        cx.notify();
    }

    pub fn set_auto_download(&mut self, enabled: bool, cx: &mut Context<Self>) {
        self.settings.auto_download = enabled;
        if let Err(error) = self.settings.save() {
            eprintln!("Failed to persist update settings: {error}");
        }
        cx.notify();
    }

    pub fn check_for_updates(&mut self, is_launch_check: bool, cx: &mut Context<Self>) {
        let now_epoch = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if is_launch_check {
            if !self.settings.check_on_launch {
                return;
            }
            if let Some(last_check) = self.settings.last_checked
                && now_epoch.saturating_sub(last_check) < COOLDOWN_SECONDS
            {
                return;
            }
        }

        self.status = UpdateStatus::Checking;
        cx.notify();

        let channel = self.settings.channel;
        let auto_download = self.settings.auto_download;
        let current_version_str = env!("CARGO_PKG_VERSION").to_string();

        let update_task = cx.spawn({
            let current_version_str = current_version_str.clone();
            async move |this, cx| {
                let check_result = cx
                    .background_executor()
                    .spawn(
                        async move { fetch_latest_manifest(channel, &current_version_str).await },
                    )
                    .await;

                this.update(cx, |manager, cx| {
                    let now_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();

                    manager.settings.last_checked = Some(now_timestamp);
                    if let Err(error) = manager.settings.save() {
                        eprintln!("Failed to persist update timestamp: {error}");
                    }

                    match check_result {
                        Ok(Some(available_update)) => {
                            manager.banner_dismissed = false;
                            manager.status = UpdateStatus::Available {
                                version: available_update.version,
                                tag: available_update.tag,
                                download_url: available_update.download_url,
                                sha256: available_update.sha256,
                                size_bytes: available_update.size_bytes,
                                changelog_url: available_update.changelog_url,
                            };
                            cx.notify();

                            if auto_download {
                                manager.start_download(cx);
                            }
                        }
                        Ok(None) => {
                            manager.status = UpdateStatus::UpToDate {
                                checked_at: now_timestamp,
                            };
                            cx.notify();
                        }
                        Err(error_message) => {
                            if !is_launch_check {
                                manager.status = UpdateStatus::Error(error_message);
                                cx.notify();
                            } else {
                                manager.status = UpdateStatus::Idle;
                                cx.notify();
                            }
                        }
                    }
                })
                .ok();
            }
        });

        self._task = Some(update_task);
    }

    pub fn start_download(&mut self, cx: &mut Context<Self>) {
        let (version, download_url, expected_sha256, size_bytes, changelog_url) = match &self.status
        {
            UpdateStatus::Available {
                version,
                download_url,
                sha256,
                size_bytes,
                changelog_url,
                ..
            } => (
                version.clone(),
                download_url.clone(),
                sha256.clone(),
                *size_bytes,
                changelog_url.clone(),
            ),
            _ => return,
        };

        self.status = UpdateStatus::Downloading {
            version: version.clone(),
            progress: 0.0,
            downloaded_bytes: 0,
            total_bytes: size_bytes,
        };
        cx.notify();

        let download_task = cx.spawn({
            let version = version.clone();
            async move |this, cx| {
                let (message_sender, message_receiver) = std::sync::mpsc::channel();
                let sender_for_bg = message_sender.clone();
                let download_url_clone = download_url.clone();
                let expected_sha256_clone = expected_sha256.clone();
                let version_clone = version.clone();

                cx.background_executor()
                    .spawn(async move {
                        let result = download_and_verify_asset(
                            &download_url_clone,
                            &expected_sha256_clone,
                            &version_clone,
                            message_sender,
                        );
                        let _ = sender_for_bg.send(DownloadMessage::Finished(result));
                    })
                    .detach();

                let mut completed_result = None;
                while completed_result.is_none() {
                    while let Ok(message) = message_receiver.try_recv() {
                        match message {
                            DownloadMessage::Progress { downloaded, total } => {
                                let progress_fraction = if total > 0 {
                                    downloaded as f32 / total as f32
                                } else {
                                    0.0
                                };

                                this.update(cx, |manager, cx| {
                                    manager.status = UpdateStatus::Downloading {
                                        version: version.clone(),
                                        progress: progress_fraction.clamp(0.0, 1.0),
                                        downloaded_bytes: downloaded,
                                        total_bytes: total,
                                    };
                                    cx.notify();
                                })
                                .ok();
                            }
                            DownloadMessage::Finished(final_result) => {
                                completed_result = Some(final_result);
                                break;
                            }
                        }
                    }

                    if completed_result.is_none() {
                        cx.background_executor()
                            .timer(Duration::from_millis(50))
                            .await;
                    }
                }

                if let Some(final_result) = completed_result {
                    this.update(cx, |manager, cx| match final_result {
                        Ok(staged_path) => {
                            manager.status = UpdateStatus::ReadyToRestart {
                                version: version.clone(),
                                staged_path,
                                changelog_url,
                            };
                            manager.banner_dismissed = false;
                            cx.notify();
                        }
                        Err(error_message) => {
                            manager.status = UpdateStatus::Error(error_message);
                            cx.notify();
                        }
                    })
                    .ok();
                }
            }
        });

        self._task = Some(download_task);
    }

    pub fn install_and_restart(&mut self, cx: &mut Context<Self>) {
        let (staged_path, changelog_url) = match &self.status {
            UpdateStatus::ReadyToRestart {
                staged_path,
                changelog_url,
                ..
            } => (staged_path.clone(), changelog_url.clone()),
            _ => return,
        };

        if let Err(error) = perform_platform_install(&staged_path, changelog_url.as_deref()) {
            self.status = UpdateStatus::Error(format!("Failed to install update: {error}"));
            cx.notify();
            return;
        }

        cx.quit();
    }
}

pub struct AvailableUpdate {
    pub version: String,
    pub tag: String,
    pub download_url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub changelog_url: Option<String>,
}

fn current_platform_key() -> Option<&'static str> {
    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    return Some("linux-x86_64");

    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    return Some("darwin-aarch64");

    #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
    return Some("darwin-x86_64");

    #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
    return Some("windows-x86_64");

    #[allow(unreachable_code)]
    None
}

async fn fetch_latest_manifest(
    channel: ReleaseChannel,
    current_version_str: &str,
) -> std::result::Result<Option<AvailableUpdate>, String> {
    let platform_key = current_platform_key().ok_or_else(|| {
        "Current OS or architecture is not supported for auto-updates.".to_string()
    })?;

    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("TNotes-Desktop/{current_version_str}"))
        .timeout(Duration::from_secs(15))
        .build()
        .map_err(|error| format!("Failed to initialize HTTP client: {error}"))?;

    let feed_url = format!("https://github.com/{REPO_OWNER}/{REPO_NAME}/releases.atom");
    let manifest_url = match resolve_tag_from_feed(&client, &feed_url, channel) {
        Some(tag) => {
            format!(
                "https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/download/{tag}/latest.json"
            )
        }
        None => {
            if channel == ReleaseChannel::Stable {
                format!(
                    "https://github.com/{REPO_OWNER}/{REPO_NAME}/releases/latest/download/latest.json"
                )
            } else {
                return Ok(None);
            }
        }
    };

    let response = client
        .get(&manifest_url)
        .send()
        .map_err(|error| format!("Failed to reach update server: {error}"))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }

    if !response.status().is_success() {
        return Err(format!(
            "Update server returned HTTP status: {}",
            response.status()
        ));
    }

    let manifest: ReleaseManifest = response
        .json()
        .map_err(|error| format!("Failed to parse release manifest: {error}"))?;

    if !channel.allows_version(&manifest.version) {
        return Ok(None);
    }

    let current_semver = Version::parse(current_version_str.trim_start_matches('v'))
        .map_err(|error| format!("Invalid local version format: {error}"))?;
    let remote_semver = Version::parse(manifest.version.trim_start_matches('v'))
        .map_err(|error| format!("Invalid remote version format: {error}"))?;

    if remote_semver <= current_semver {
        return Ok(None);
    }

    let platform_asset = manifest.platforms.get(platform_key).ok_or_else(|| {
        format!(
            "No release asset found for platform '{platform_key}' in release v{}.",
            manifest.version
        )
    })?;

    Ok(Some(AvailableUpdate {
        version: manifest.version,
        tag: manifest.tag,
        download_url: platform_asset.url.clone(),
        sha256: platform_asset.sha256.clone(),
        size_bytes: platform_asset.size,
        changelog_url: manifest.changelog_url,
    }))
}

pub fn parse_tag_from_feed(feed_xml: &str, channel: ReleaseChannel) -> Option<String> {
    for line in feed_xml.lines() {
        if line.contains("<link") && let Some(start) = line.find("/releases/tag/") {
            let rest = &line[start + "/releases/tag/".len()..];
            if let Some(tag) = rest.split(&['"', '<', '>', '/'][..]).next() {
                let tag = tag.trim();
                if !tag.is_empty() && channel.allows_version(tag) {
                    return Some(tag.to_string());
                }
            }
        }
    }
    None
}

fn resolve_tag_from_feed(
    client: &reqwest::blocking::Client,
    feed_url: &str,
    channel: ReleaseChannel,
) -> Option<String> {
    let response = client.get(feed_url).send().ok()?;
    if !response.status().is_success() {
        return None;
    }
    let body = response.text().ok()?;
    parse_tag_from_feed(&body, channel)
}

fn download_and_verify_asset(
    download_url: &str,
    expected_sha256: &str,
    version: &str,
    progress_sender: std::sync::mpsc::Sender<DownloadMessage>,
) -> std::result::Result<PathBuf, String> {
    let client = reqwest::blocking::Client::builder()
        .user_agent(format!("TNotes-Desktop/{version}"))
        .timeout(Duration::from_secs(300))
        .build()
        .map_err(|error| format!("Failed to create download client: {error}"))?;

    let mut response = client
        .get(download_url)
        .send()
        .map_err(|error| format!("Failed to start download: {error}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "Download failed with status: {}",
            response.status()
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let cache_directory = paths::cache_dir();
    let temp_filename = format!("tnotes-update-{version}.staging");
    let staging_path = cache_directory.join(temp_filename);

    let mut file = File::create(&staging_path)
        .map_err(|error| format!("Failed to create staging file: {error}"))?;

    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 16384];
    let mut downloaded_bytes: u64 = 0;

    loop {
        let bytes_read = response
            .read(&mut buffer)
            .map_err(|error| format!("Error while reading download stream: {error}"))?;

        if bytes_read == 0 {
            break;
        }

        file.write_all(&buffer[..bytes_read])
            .map_err(|error| format!("Failed to write to update file: {error}"))?;
        hasher.update(&buffer[..bytes_read]);

        downloaded_bytes += bytes_read as u64;
        let _ = progress_sender.send(DownloadMessage::Progress {
            downloaded: downloaded_bytes,
            total: total_size,
        });
    }

    file.flush()
        .map_err(|error| format!("Failed to flush file: {error}"))?;

    let calculated_hash = format!("{:x}", hasher.finalize());
    if calculated_hash.to_lowercase() != expected_sha256.to_lowercase() {
        let _ = std::fs::remove_file(&staging_path);
        return Err(format!(
            "Checksum verification failed! Expected: {expected_sha256}, calculated: {calculated_hash}"
        ));
    }

    Ok(staging_path)
}

fn perform_platform_install(
    staged_path: &PathBuf,
    changelog_url: Option<&str>,
) -> std::result::Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(appimage_env) = std::env::var("APPIMAGE") {
            use std::os::unix::fs::PermissionsExt;
            let target_appimage = PathBuf::from(appimage_env);

            let metadata = std::fs::metadata(staged_path)
                .map_err(|error| format!("Failed to read metadata: {error}"))?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o755);
            std::fs::set_permissions(staged_path, permissions)
                .map_err(|error| format!("Failed to set executable bit: {error}"))?;

            std::fs::rename(staged_path, &target_appimage)
                .map_err(|error| format!("Failed to replace AppImage file: {error}"))?;

            std::process::Command::new(&target_appimage)
                .spawn()
                .map_err(|error| format!("Failed to relaunch new AppImage: {error}"))?;

            return Ok(());
        }

        // Fallback for non-AppImage Linux installations
        if let Some(url) = changelog_url {
            let _ = std::process::Command::new("xdg-open").arg(url).spawn();
        }
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        let destination_app = PathBuf::from("/Applications/TNotes.app");
        let pid = std::process::id();
        let extract_script = format!(
            "while kill -0 {pid} 2>/dev/null; do sleep 0.2; done; \
             ditto -x -k '{staged}' '{cache}' && \
             rm -rf '{dest}' && \
             cp -R '{cache}/TNotes.app' '{dest}' && \
             open -n '{dest}'",
            pid = pid,
            staged = staged_path.display(),
            cache = paths::cache_dir().display(),
            dest = destination_app.display()
        );

        std::process::Command::new("sh")
            .arg("-c")
            .arg(extract_script)
            .spawn()
            .map_err(|error| format!("Failed to launch macOS updater script: {error}"))?;

        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new(staged_path)
            .args(["/SILENT", "/CLOSEAPPLICATIONS", "/RESTARTAPPLICATIONS"])
            .spawn()
            .map_err(|error| format!("Failed to spawn Windows installer: {error}"))?;

        return Ok(());
    }

    #[allow(unreachable_code)]
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn release_channel_metadata() {
        assert_eq!(ReleaseChannel::ALL.len(), 3);
        assert_eq!(ReleaseChannel::Stable.title(), "Stable");
        assert_eq!(ReleaseChannel::Beta.title(), "Beta");
        assert_eq!(ReleaseChannel::Alpha.title(), "Alpha");

        assert!(!ReleaseChannel::Stable.description().is_empty());
        assert!(!ReleaseChannel::Beta.description().is_empty());
        assert!(!ReleaseChannel::Alpha.description().is_empty());
    }

    #[test]
    fn release_channel_filter_logic() {
        // Stable
        assert!(ReleaseChannel::Stable.allows_version("0.2.0"));
        assert!(ReleaseChannel::Stable.allows_version("1.0.0"));
        assert!(!ReleaseChannel::Stable.allows_version("0.2.0-alpha.1"));
        assert!(!ReleaseChannel::Stable.allows_version("0.2.0-beta.2"));
        assert!(!ReleaseChannel::Stable.allows_version("0.2.0-rc.1"));

        // Beta
        assert!(ReleaseChannel::Beta.allows_version("0.2.0"));
        assert!(ReleaseChannel::Beta.allows_version("0.2.0-beta.1"));
        assert!(ReleaseChannel::Beta.allows_version("0.2.0-rc.1"));
        assert!(!ReleaseChannel::Beta.allows_version("0.2.0-alpha.1"));

        // Alpha
        assert!(ReleaseChannel::Alpha.allows_version("0.2.0"));
        assert!(ReleaseChannel::Alpha.allows_version("0.2.0-alpha.1"));
        assert!(ReleaseChannel::Alpha.allows_version("0.2.0-beta.1"));
        assert!(ReleaseChannel::Alpha.allows_version("0.2.0-rc.1"));
    }

    #[test]
    fn release_channel_default_for_version() {
        assert_eq!(
            ReleaseChannel::default_for_version("0.2.0-alpha.2"),
            ReleaseChannel::Alpha
        );
        assert_eq!(
            ReleaseChannel::default_for_version("0.2.0-beta.1"),
            ReleaseChannel::Beta
        );
        assert_eq!(
            ReleaseChannel::default_for_version("0.2.0-rc.1"),
            ReleaseChannel::Beta
        );
        assert_eq!(
            ReleaseChannel::default_for_version("0.2.0"),
            ReleaseChannel::Stable
        );
        assert_eq!(
            ReleaseChannel::default_for_version("1.0.0"),
            ReleaseChannel::Stable
        );
    }

    #[test]
    fn parse_tag_from_feed_filtering() {
        let sample_feed = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <id>tag:github.com,2008:Repository/123/v0.2.0-alpha.2</id>
    <link rel="alternate" type="text/html" href="https://github.com/ToonionOfficial/tnotes/releases/tag/v0.2.0-alpha.2"/>
    <title>v0.2.0-alpha.2</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/123/v0.2.0-alpha.1</id>
    <link rel="alternate" type="text/html" href="https://github.com/ToonionOfficial/tnotes/releases/tag/v0.2.0-alpha.1"/>
    <title>v0.2.0-alpha.1</title>
  </entry>
  <entry>
    <id>tag:github.com,2008:Repository/123/v0.1.0</id>
    <link rel="alternate" type="text/html" href="https://github.com/ToonionOfficial/tnotes/releases/tag/v0.1.0"/>
    <title>v0.1.0</title>
  </entry>
</feed>"#;

        assert_eq!(
            parse_tag_from_feed(sample_feed, ReleaseChannel::Alpha),
            Some("v0.2.0-alpha.2".to_string())
        );
        assert_eq!(
            parse_tag_from_feed(sample_feed, ReleaseChannel::Beta),
            Some("v0.1.0".to_string())
        );
        assert_eq!(
            parse_tag_from_feed(sample_feed, ReleaseChannel::Stable),
            Some("v0.1.0".to_string())
        );

        let feed_without_stable = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <link rel="alternate" type="text/html" href="https://github.com/ToonionOfficial/tnotes/releases/tag/v0.2.0-alpha.2"/>
  </entry>
</feed>"#;

        assert_eq!(
            parse_tag_from_feed(feed_without_stable, ReleaseChannel::Alpha),
            Some("v0.2.0-alpha.2".to_string())
        );
        assert_eq!(
            parse_tag_from_feed(feed_without_feed_channel(ReleaseChannel::Beta), ReleaseChannel::Beta),
            None
        );
        assert_eq!(
            parse_tag_from_feed(feed_without_stable, ReleaseChannel::Stable),
            None
        );
    }

    fn feed_without_feed_channel(_ch: ReleaseChannel) -> &'static str {
        r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <entry>
    <link rel="alternate" type="text/html" href="https://github.com/ToonionOfficial/tnotes/releases/tag/v0.2.0-alpha.2"/>
  </entry>
</feed>"#
    }

    #[test]
    fn update_settings_default_and_roundtrip() {
        let default_settings = UpdateSettings::default();
        assert!(default_settings.check_on_launch);
        assert!(!default_settings.auto_download);
        assert_eq!(default_settings.channel, ReleaseChannel::default());
        assert_eq!(default_settings.last_checked, None);

        let json = serde_json::to_string(&default_settings).unwrap();
        let parsed: UpdateSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.check_on_launch, default_settings.check_on_launch);
        assert_eq!(parsed.auto_download, default_settings.auto_download);
        assert_eq!(parsed.channel, default_settings.channel);
        assert_eq!(parsed.last_checked, default_settings.last_checked);
    }

    #[test]
    fn release_manifest_deserialization() {
        let manifest_raw = r#"{
            "version": "0.3.0",
            "tag": "v0.3.0",
            "pub_date": "2026-09-18T00:00:00Z",
            "changelog_url": "https://github.com/ToonionOfficial/tnotes/releases/tag/v0.3.0",
            "platforms": {
                "linux-x86_64": {
                    "name": "tnotes-v0.3.0-linux-x86_64.AppImage",
                    "url": "https://github.com/ToonionOfficial/tnotes/releases/download/v0.3.0/tnotes-v0.3.0-linux-x86_64.AppImage",
                    "sha256": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
                    "size": 45000000
                }
            }
        }"#;

        let manifest: ReleaseManifest = serde_json::from_str(manifest_raw).unwrap();
        assert_eq!(manifest.version, "0.3.0");
        assert_eq!(manifest.tag, "v0.3.0");
        assert!(manifest.platforms.contains_key("linux-x86_64"));

        let linux_asset = manifest.platforms.get("linux-x86_64").unwrap();
        assert_eq!(linux_asset.name, "tnotes-v0.3.0-linux-x86_64.AppImage");
        assert_eq!(linux_asset.size, 45000000);
    }
}
