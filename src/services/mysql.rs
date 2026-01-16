use anyhow::{Result, Context, bail};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use super::config::MySqlConfig;

pub struct MySqlService {
    config: MySqlConfig,
}

impl MySqlService {
    pub fn new(config: MySqlConfig) -> Self {
        Self { config }
    }

    /// Check if MySQL is installed
    pub fn is_installed(&self) -> bool {
        Command::new("which")
            .arg("mysql")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Check if MySQL service is running
    pub fn is_running(&self) -> Result<bool> {
        let output = Command::new("systemctl")
            .args(["is-active", "mysql"])
            .output()
            .context("Failed to check MySQL status")?;

        Ok(output.status.success())
    }

    /// Install MySQL server (community edition)
    pub async fn install(&self) -> Result<()> {
        println!("📦 Installing MySQL Community Server...");

        // Check OS type
        if !self.is_wsl_ubuntu() {
            bail!("This installer currently supports WSL Ubuntu only");
        }

        // Update package list
        println!("  Updating package list...");
        let status = Command::new("sudo")
            .args(["apt-get", "update", "-y"])
            .status()
            .context("Failed to update package list")?;

        if !status.success() {
            bail!("Failed to update package list");
        }

        // Install MySQL
        println!("  Installing MySQL server...");
        let status = Command::new("sudo")
            .env("DEBIAN_FRONTEND", "noninteractive")
            .args(["apt-get", "install", "-y", "mysql-server"])
            .status()
            .context("Failed to install MySQL")?;

        if !status.success() {
            bail!("Failed to install MySQL server");
        }

        println!("✅ MySQL installed successfully");
        Ok(())
    }

    /// Start MySQL service
    pub async fn start(&self) -> Result<()> {
        println!("🚀 Starting MySQL service...");

        let status = Command::new("sudo")
            .args(["systemctl", "start", "mysql"])
            .status()
            .context("Failed to start MySQL")?;

        if !status.success() {
            bail!("Failed to start MySQL service");
        }

        // Wait for service to be ready
        for i in 0..10 {
            sleep(Duration::from_secs(1)).await;
            if self.is_running()? {
                println!("✅ MySQL service started");
                return Ok(());
            }
            if i == 9 {
                bail!("MySQL service failed to start within timeout");
            }
        }

        Ok(())
    }

    /// Stop MySQL service
    pub async fn stop(&self) -> Result<()> {
        println!("🛑 Stopping MySQL service...");

        let status = Command::new("sudo")
            .args(["systemctl", "stop", "mysql"])
            .status()
            .context("Failed to stop MySQL")?;

        if !status.success() {
            bail!("Failed to stop MySQL service");
        }

        println!("✅ MySQL service stopped");
        Ok(())
    }

    /// Initialize database and user
    pub async fn initialize(&self) -> Result<()> {
        println!("🔧 Initializing MySQL database...");

        // Create database and user
        let sql_commands = format!(
            "CREATE DATABASE IF NOT EXISTS {}; \
             CREATE USER IF NOT EXISTS '{}'@'localhost' IDENTIFIED BY '{}'; \
             GRANT ALL PRIVILEGES ON {}.* TO '{}'@'localhost'; \
             FLUSH PRIVILEGES;",
            self.config.database,
            self.config.username,
            self.config.password,
            self.config.database,
            self.config.username
        );

        let status = Command::new("sudo")
            .arg("mysql")
            .arg("-e")
            .arg(&sql_commands)
            .status()
            .context("Failed to initialize MySQL database")?;

        if !status.success() {
            bail!("Failed to initialize MySQL database");
        }

        println!("✅ MySQL database initialized");
        Ok(())
    }

    /// Reset database to clean state
    pub async fn reset(&self) -> Result<()> {
        println!("🔄 Resetting MySQL database...");

        let sql_commands = format!(
            "DROP DATABASE IF EXISTS {}; \
             CREATE DATABASE {}; \
             FLUSH PRIVILEGES;",
            self.config.database, self.config.database
        );

        let status = Command::new("sudo")
            .arg("mysql")
            .arg("-e")
            .arg(&sql_commands)
            .status()
            .context("Failed to reset MySQL database")?;

        if !status.success() {
            bail!("Failed to reset MySQL database");
        }

        println!("✅ MySQL database reset complete");
        Ok(())
    }

    /// Get service status
    pub fn status(&self) -> ServiceStatus {
        ServiceStatus {
            name: "MySQL".to_string(),
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

#[derive(Debug, Clone)]
pub struct ServiceStatus {
    pub name: String,
    pub installed: bool,
    pub running: bool,
    pub host: String,
    pub port: u16,
    pub database: Option<String>,
}

impl ServiceStatus {
    pub fn print(&self) {
        println!("  📦 {}", self.name);
        println!("     Installed: {}", if self.installed { "✅ Yes" } else { "❌ No" });
        println!("     Running:   {}", if self.running { "✅ Yes" } else { "❌ No" });
        println!("     Host:      {}:{}", self.host, self.port);
        if let Some(db) = &self.database {
            println!("     Database:  {}", db);
        }
    }
}
