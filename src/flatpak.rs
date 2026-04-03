use anyhow::Result;
use std::process::Stdio;
use tokio::process::Command;

#[derive(Clone, Debug, Default)]
pub struct FlatpakApp {
    pub name: String,
    pub description: String,
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
        .arg("--columns=name,description,application,version,branch")
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
            line.split("  ").filter(|s| !s.trim().is_empty()).collect()
        };

        if parts.len() >= 2 {
            let app_idx = parts.iter().position(|s| s.contains('.') && !s.contains(' ')).unwrap_or(2.min(parts.len() - 1));
            let desc = if app_idx > 1 { parts[1..app_idx].join(" ") } else { "".to_string() };
            
            apps.push(FlatpakApp {
                name: parts[0].trim().to_string(),
                description: desc.trim().to_string(),
                application_id: parts[app_idx].trim().to_string(),
                version: parts.get(app_idx + 1).unwrap_or(&"").trim().to_string(),
                branch: parts.get(app_idx + 2).unwrap_or(&"").trim().to_string(),
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
        .arg("--columns=name,description,application,version,branch")
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
            let app_idx = parts.iter().position(|s| s.contains('.') && !s.contains(' ')).unwrap_or(2.min(parts.len() - 1));
            let desc = if app_idx > 1 { parts[1..app_idx].join(" ") } else { "".to_string() };
            
            apps.push(FlatpakApp {
                name: parts[0].trim().to_string(),
                description: desc.trim().to_string(),
                application_id: parts[app_idx].trim().to_string(),
                version: parts.get(app_idx + 1).unwrap_or(&"").trim().to_string(),
                branch: parts.get(app_idx + 2).unwrap_or(&"").trim().to_string(),
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

pub async fn search_remote_apps(query: &str) -> Result<Vec<FlatpakApp>> {
    let output = Command::new("flatpak")
        .arg("search")
        .arg(query)
        .stdout(Stdio::piped())
        .output()
        .await?;
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut apps = Vec::new();
    
    for line in stdout.lines() {
        if line.trim().is_empty() || line.starts_with("Name") || line.contains("Application ID") { 
            continue; 
        }
        
        let parts: Vec<&str> = if line.contains('\t') { 
            line.split('\t').collect() 
        } else { 
            line.split("  ").filter(|s| !s.trim().is_empty()).collect() 
        };
        
        if parts.len() >= 2 {
            let app_idx = parts.iter().position(|s| s.contains('.') && !s.contains(' ')).unwrap_or(2.min(parts.len() - 1));
            let desc = if app_idx > 1 { parts[1..app_idx].join(" ") } else { "".to_string() };
            
            apps.push(FlatpakApp {
                name: parts[0].trim().to_string(),
                description: desc.trim().to_string(),
                application_id: parts[app_idx].trim().to_string(),
                version: parts.get(app_idx + 1).unwrap_or(&"").trim().to_string(),
                branch: parts.get(app_idx + 2).unwrap_or(&"").trim().to_string(),
                details: None,
            });
        }
    }
    Ok(apps)
}

pub async fn install_app(application_id: &str) -> Result<()> {
    let output = Command::new("flatpak")
        .arg("install")
        .arg("-y")
        .arg("--noninteractive")
        .arg(application_id)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await?;

    if !output.status.success() {
        anyhow::bail!("Install command failed on {}", application_id);
    }
    Ok(())
}

pub async fn get_app_permissions(application_id: &str) -> Result<Vec<(String, bool)>> {
    // We'll use 'flatpak info --show-permissions' to get current permissions
    // and 'flatpak info -M' to get all possible/default permissions if needed.
    // Simplifying: just get what's currently active.
    let output = Command::new("flatpak")
        .arg("info")
        .arg("--show-permissions")
        .arg(application_id)
        .stdout(Stdio::piped())
        .output()
        .await?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut permissions = Vec::new();
    
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("[") {
            continue;
        }
        // Lines are usually like "filesystems=home;xdg-config/kdeglobals:ro;"
        if let Some((key, values)) = line.split_once('=') {
            for val in values.split(';') {
                if !val.is_empty() {
                    permissions.push((format!("{}={}", key, val), true));
                }
            }
        } else {
            permissions.push((line.to_string(), true));
        }
    }
    Ok(permissions)
}

pub async fn set_app_permission(application_id: &str, permission: &str, enable: bool) -> Result<()> {
    // Try user override first, if it fails, suggest root/system
    let action = if enable { "--unoverride" } else { "--nosocket" }; // Simplified for now
    // Actually, flatpak override use --[enable|disable] or --nosocket etc.
    // For simplicity, let's use the explicit override syntax:
    // flatpak override --user --nosocket=wayland org.test.App
    
    let mut command = Command::new("flatpak");
    command.arg("override").arg("--user");
    
    let parts: Vec<&str> = permission.splitn(2, '=').collect();
    if parts.len() == 2 {
        let key = parts[0];
        let val = parts[1];
        if enable {
             command.arg(format!("--{}={}", key, val));
        } else {
             command.arg(format!("--no-{}={}", key, val));
        }
    } else {
        if enable {
            command.arg(format!("--{}", permission));
        } else {
            command.arg(format!("--no-{}", permission));
        }
    }
    
    command.arg(application_id);
    
    let output = command.output().await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Permission change failed: {}. Try running as root for system-wide apps.", stderr);
    }
    
    Ok(())
}
