use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Clone, Debug, Default)]
pub struct FlatpakApp {
    pub name: String,
    pub application_id: String,
    pub version: String,
    pub branch: String,
    pub details: Option<String>,
}

pub async fn get_installed_apps() -> Result<Vec<FlatpakApp>> {
    get_flatpak_list("--app").await
}

pub async fn get_installed_runtimes() -> Result<Vec<FlatpakApp>> {
    get_flatpak_list("--runtime").await
}

async fn get_flatpak_list(app_type: &str) -> Result<Vec<FlatpakApp>> {
    let output = Command::new("flatpak")
        .arg("list")
        .arg(app_type)
        .arg("--columns=name,application,version,branch")
        .stdout(Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        // Flatpak usually outputs columns separated by tabs or spaces.
        // When piped, it typically emits tabs.
        let parts: Vec<&str> = if line.contains('\t') {
            line.split('\t').collect()
        } else {
            // fallback: split by 2 or more spaces
            line.split("  ").filter(|s| !s.trim().is_empty()).collect()
        };

        if parts.len() >= 2 {
            apps.push(FlatpakApp {
                name: parts[0].trim().to_string(),
                application_id: parts[1].trim().to_string(),
                version: parts.get(2).unwrap_or(&"").trim().to_string(),
                branch: parts.get(3).unwrap_or(&"").trim().to_string(),
                details: None,
            });
        }
    }

    Ok(apps)
}

pub async fn get_app_info(application_id: &str) -> Result<String> {
    let output = Command::new("flatpak")
        .arg("info")
        .arg(application_id)
        .stdout(Stdio::piped())
        .output()
        .await?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn uninstall_app(application_id: &str) -> Result<()> {
    let output = Command::new("flatpak")
        .arg("uninstall")
        .arg("-y")
        .arg("--noninteractive")
        .arg(application_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Uninstall command failed on {}", application_id);
    }
    Ok(())
}

pub async fn update_app(application_id: &str) -> Result<()> {
    let output = Command::new("flatpak")
        .arg("update")
        .arg("-y")
        .arg("--noninteractive")
        .arg(application_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Update command failed on {}", application_id);
    }
    Ok(())
}

pub async fn get_updates() -> Result<Vec<FlatpakApp>> {
    let output = Command::new("flatpak")
        .arg("remote-ls")
        .arg("--updates")
        .arg("--columns=name,application,version,branch")
        .stdout(Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();

    for line in stdout.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let parts: Vec<&str> = if line.contains('\t') {
            line.split('\t').collect()
        } else {
            line.split("  ").filter(|s| !s.trim().is_empty()).collect()
        };

        if parts.len() >= 2 {
            apps.push(FlatpakApp {
                name: parts[0].trim().to_string(),
                application_id: parts[1].trim().to_string(),
                version: parts.get(2).unwrap_or(&"").trim().to_string(),
                branch: parts.get(3).unwrap_or(&"").trim().to_string(),
                details: None,
            });
        }
    }

    Ok(apps)
}

pub async fn update_all() -> Result<()> {
    let output = Command::new("flatpak")
        .arg("update")
        .arg("-y")
        .arg("--noninteractive")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Failed to update all");
    }
    Ok(())
}
