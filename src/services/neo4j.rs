use anyhow::{Result, Context, bail};
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use super::config::Neo4jConfig;
use super::mysql::ServiceStatus;

pub struct Neo4jService {
    config: Neo4jConfig,
}

impl Neo4jService {
    pub fn new(config: Neo4jConfig) -> Self {
        Self { config }
    }

    /// Check if Neo4j is installed
    pub fn is_installed(&self) -> bool {
        // Check if neo4j command exists or if it's installed via apt
        Command::new("which")
            .arg("neo4j")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
        || std::path::Path::new("/usr/bin/neo4j").exists()
        || std::path::Path::new("/var/lib/neo4j").exists()
    }

    /// Check if Neo4j service is running
    pub fn is_running(&self) -> Result<bool> {
        // Try systemctl first
        if let Ok(output) = Command::new("systemctl")
            .args(["is-active", "neo4j"])
            .output()
        {
            if output.status.success() {
                return Ok(true);
            }
        }

        // Check if process is running
        if let Ok(output) = Command::new("pgrep")
            .args(["-f", "neo4j"])
            .output()
        {
            return Ok(output.status.success() && !output.stdout.is_empty());
        }

        Ok(false)
    }

    /// Install Neo4j Community Edition
    pub async fn install(&self) -> Result<()> {
        println!("📦 Installing Neo4j Community Edition...");

        // Check OS type
        if !self.is_wsl_ubuntu() {
            bail!("This installer currently supports WSL Ubuntu only");
        }

        // Install dependencies
        println!("  Installing dependencies...");
        let status = Command::new("sudo")
            .args(["apt-get", "install", "-y", "wget", "gnupg", "software-properties-common"])
            .status()
            .context("Failed to install dependencies")?;

        if !status.success() {
            bail!("Failed to install dependencies");
        }

        // Add Neo4j repository key
        println!("  Adding Neo4j repository...");
        let status = Command::new("wget")
            .args(["-O", "-", "https://debian.neo4j.com/neotechnology.gpg.key"])
            .stdout(Stdio::piped())
            .spawn()
            .and_then(|child| {
                Command::new("sudo")
                    .args(["apt-key", "add", "-"])
                    .stdin(child.stdout.unwrap())
                    .status()
            })
            .context("Failed to add Neo4j GPG key")?;

        if !status.success() {
            println!("  Warning: Could not add GPG key via apt-key (might be deprecated)");
            println!("  Trying alternative method...");
            
            // Alternative method for newer Ubuntu versions
            Command::new("wget")
                .args(["-O", "/tmp/neo4j.gpg.key", "https://debian.neo4j.com/neotechnology.gpg.key"])
                .status()?;
            
            Command::new("sudo")
                .args(["mkdir", "-p", "/etc/apt/keyrings"])
                .status()?;
            
            Command::new("sudo")
                .args(["gpg", "--dearmor", "-o", "/etc/apt/keyrings/neo4j.gpg", "/tmp/neo4j.gpg.key"])
                .status()?;
        }

        // Add Neo4j repository
        let repo_line = "deb [signed-by=/etc/apt/keyrings/neo4j.gpg] https://debian.neo4j.com stable latest";
        let status = Command::new("sudo")
            .arg("sh")
            .arg("-c")
            .arg(format!("echo '{}' > /etc/apt/sources.list.d/neo4j.list", repo_line))
            .status()
            .context("Failed to add Neo4j repository")?;

        if !status.success() {
            // Try simpler method
            Command::new("sudo")
                .arg("add-apt-repository")
                .arg("-y")
                .arg("deb https://debian.neo4j.com stable latest")
                .status()?;
        }

        // Update package list
        println!("  Updating package list...");
        Command::new("sudo")
            .args(["apt-get", "update", "-y"])
            .status()
            .context("Failed to update package list")?;

        // Install Neo4j
        println!("  Installing Neo4j...");
        let status = Command::new("sudo")
            .env("DEBIAN_FRONTEND", "noninteractive")
            .args(["apt-get", "install", "-y", "neo4j"])
            .status()
            .context("Failed to install Neo4j")?;

        if !status.success() {
            bail!("Failed to install Neo4j");
        }

        println!("✅ Neo4j installed successfully");
        Ok(())
    }

    /// Start Neo4j service
    pub async fn start(&self) -> Result<()> {
        println!("🚀 Starting Neo4j service...");

        let status = Command::new("sudo")
            .args(["systemctl", "start", "neo4j"])
            .status()
            .context("Failed to start Neo4j")?;

        if !status.success() {
            // Try alternative start method
            println!("  Trying alternative start method...");
            Command::new("sudo")
                .arg("neo4j")
                .arg("start")
                .status()
                .context("Failed to start Neo4j")?;
        }

        // Wait for service to be ready
        for i in 0..30 {
            sleep(Duration::from_secs(1)).await;
            if self.is_running()? {
                println!("✅ Neo4j service started");
                return Ok(());
            }
            if i == 29 {
                bail!("Neo4j service failed to start within timeout");
            }
        }

        Ok(())
    }

    /// Stop Neo4j service
    pub async fn stop(&self) -> Result<()> {
        println!("🛑 Stopping Neo4j service...");

        let status = Command::new("sudo")
            .args(["systemctl", "stop", "neo4j"])
            .status()
            .context("Failed to stop Neo4j")?;

        if !status.success() {
            // Try alternative stop method
            Command::new("sudo")
                .arg("neo4j")
                .arg("stop")
                .status()
                .context("Failed to stop Neo4j")?;
        }

        println!("✅ Neo4j service stopped");
        Ok(())
    }

    /// Initialize Neo4j with password
    pub async fn initialize(&self) -> Result<()> {
        println!("🔧 Initializing Neo4j...");

        // Set initial password using neo4j-admin
        let status = Command::new("sudo")
            .args([
                "neo4j-admin",
                "dbms",
                "set-initial-password",
                &self.config.password,
            ])
            .status();

        match status {
            Ok(s) if s.success() => {
                println!("✅ Neo4j password set");
            }
            _ => {
                println!("  Note: Password may already be set or neo4j-admin not available");
            }
        }

        Ok(())
    }

    /// Reset Neo4j database
    pub async fn reset(&self) -> Result<()> {
        println!("🔄 Resetting Neo4j database...");

        // Stop Neo4j first
        let was_running = self.is_running()?;
        if was_running {
            self.stop().await?;
        }

        // Remove data directory
        println!("  Removing Neo4j data...");
        let status = Command::new("sudo")
            .args(["rm", "-rf", "/var/lib/neo4j/data/databases/*"])
            .status()
            .context("Failed to remove Neo4j data")?;

        if !status.success() {
            println!("  Warning: Could not remove all data files");
        }

        // Restart if it was running
        if was_running {
            self.start().await?;
            self.initialize().await?;
        }

        println!("✅ Neo4j database reset complete");
        Ok(())
    }

    /// Get service status
    pub fn status(&self) -> ServiceStatus {
        ServiceStatus {
            name: "Neo4j".to_string(),
            installed: self.is_installed(),
            running: self.is_running().unwrap_or(false),
            host: self.config.host.clone(),
            port: self.config.port,
            database: Some(self.config.database.clone()),
        }
    }

    fn is_wsl_ubuntu(&self) -> bool {
        std::fs::read_to_string("/proc/version")
            .map(|s| s.to_lowercase().contains("microsoft") || s.contains("WSL"))
            .unwrap_or(false)
    }
}
