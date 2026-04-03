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
    let output = Command::new("flatpak")
        .arg("list")
        .arg("--app")
        .arg("--columns=name,description,application,version,branch")
        .stdout(Stdio::piped())
        .output()
        .await?;
    Ok(parse_flatpak_list(&String::from_utf8_lossy(&output.stdout)))
}

pub async fn get_installed_runtimes() -> Result<Vec<FlatpakApp>> {
    let output = Command::new("flatpak")
        .arg("list")
        .arg("--runtime")
        .arg("--columns=name,description,application,version,branch")
        .stdout(Stdio::piped())
        .output()
        .await?;
    Ok(parse_flatpak_list(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_flatpak_list(stdout: &str) -> Vec<FlatpakApp> {
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
    apps
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

    Ok(parse_flatpak_list(&String::from_utf8_lossy(&output.stdout)))
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
        
    Ok(parse_search_results(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_search_results(stdout: &str) -> Vec<FlatpakApp> {
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
            let app_idx = parts.iter().position(|s| s.trim().contains('.') && !s.trim().contains(' ')).unwrap_or(2.min(parts.len() - 1));
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
    apps
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
    let output = Command::new("flatpak")
        .arg("info")
        .arg("--show-permissions")
        .arg(application_id)
        .stdout(Stdio::piped())
        .output()
        .await?;

    Ok(parse_app_permissions(&String::from_utf8_lossy(&output.stdout)))
}

fn parse_app_permissions(stdout: &str) -> Vec<(String, bool)> {
    let mut permissions = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("[") {
            continue;
        }
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
    permissions
}

pub async fn set_app_permission(application_id: &str, permission: &str, enable: bool) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_parse_flatpak_list_tabbed() {
        let stdout = "Firefox\tWeb Browser\torg.mozilla.firefox\t123.0\tstable\n";
        let apps = parse_flatpak_list(stdout);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Firefox");
        assert_eq!(apps[0].application_id, "org.mozilla.firefox");
        assert_eq!(apps[0].version, "123.0");
    }

    #[test]
    fn test_parse_flatpak_list_spaced() {
        let stdout = "Firefox  Web Browser  org.mozilla.firefox  123.0  stable\n";
        let apps = parse_flatpak_list(stdout);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "Firefox");
        assert_eq!(apps[0].application_id, "org.mozilla.firefox");
    }

    #[test]
    fn test_parse_search_results() {
        let stdout = "Name  Description  Application ID  Version  Branch\n\
                       VLC   Player       org.videolan.VLC  3.0.20   stable\n";
        let apps = parse_search_results(stdout);
        assert_eq!(apps.len(), 1);
        assert_eq!(apps[0].name, "VLC");
        assert_eq!(apps[0].application_id, "org.videolan.VLC");
    }

    #[test]
    fn test_parse_app_permissions() {
        let stdout = "[Context]\n\
                      filesystems=home;xdg-config/kdeglobals:ro;\n\
                      sockets=wayland;x11;\n";
        let perms = parse_app_permissions(stdout);
        assert_eq!(perms.len(), 4);
        assert!(perms.contains(&("filesystems=home".to_string(), true)));
        assert!(perms.contains(&("sockets=wayland".to_string(), true)));
        assert!(perms.contains(&("filesystems=xdg-config/kdeglobals:ro".to_string(), true)));
    }

    #[test]
    fn test_parse_flatpak_list_empty() {
        assert_eq!(parse_flatpak_list("").len(), 0);
        assert_eq!(parse_flatpak_list("\n\n").len(), 0);
    }

    #[test]
    fn test_parse_flatpak_list_malformed() {
        let stdout = "Malformed Line Without Enough Parts\n";
        let apps = parse_flatpak_list(stdout);
        assert_eq!(apps.len(), 0);
    }

    #[test]
    fn test_parse_app_permissions_empty() {
        assert_eq!(parse_app_permissions("").len(), 0);
    }
}
