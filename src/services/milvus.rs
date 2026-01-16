use anyhow::{Result, bail};
use super::config::MilvusConfig;
use super::mysql::ServiceStatus;
use std::process::Command;

pub struct MilvusService {
    config: MilvusConfig,
}

impl MilvusService {
    pub fn new(config: MilvusConfig) -> Self {
        Self { config }
    }

    /// Check if Milvus is installed (via Docker)
    pub fn is_installed(&self) -> bool {
        // Check if Docker is available
        if !Command::new("docker")
            .arg("--version")
            .output()
            .is_ok()
        {
            return false;
        }

        // Check if milvus-standalone image exists
        Command::new("docker")
            .args(["images", "-q", "milvusdb/milvus:latest"])
            .output()
            .map(|output| !output.stdout.is_empty())
            .unwrap_or(false)
    }

    /// Check if Milvus container is running
    pub fn is_running(&self) -> Result<bool> {
        let output = Command::new("docker")
            .args(["ps", "--filter", "name=milvus-standalone", "--format", "{{.Names}}"])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).contains("milvus-standalone"))
    }

    /// Install Milvus (pull Docker image and setup)
    pub async fn install(&self) -> Result<()> {
        println!("📦 Installing Milvus...");
        
        // Check if Docker is installed
        if !Command::new("docker")
            .arg("--version")
            .output()
            .is_ok()
        {
            bail!("Docker is not installed. Please install Docker first.\nSee: https://docs.docker.com/get-docker/");
        }

        println!("  Pulling Milvus Docker image...");
        let status = Command::new("docker")
            .args(["pull", "milvusdb/milvus:latest"])
            .status()?;

        if !status.success() {
            bail!("Failed to pull Milvus Docker image");
        }

        // Pull etcd image (required for Milvus standalone)
        println!("  Pulling etcd image...");
        Command::new("docker")
            .args(["pull", "quay.io/coreos/etcd:latest"])
            .status()?;

        // Pull MinIO image (required for Milvus storage)
        println!("  Pulling MinIO image...");
        Command::new("docker")
            .args(["pull", "minio/minio:latest"])
            .status()?;

        println!("✅ Milvus installation complete!");
        Ok(())
    }

    /// Start Milvus service (Docker container)
    pub async fn start(&self) -> Result<()> {
        println!("🚀 Starting Milvus...");

        // Check if container already exists
        let existing = Command::new("docker")
            .args(["ps", "-a", "--filter", "name=milvus-standalone", "--format", "{{.Names}}"])
            .output()?;

        if String::from_utf8_lossy(&existing.stdout).contains("milvus-standalone") {
            // Container exists, just start it
            println!("  Starting existing Milvus container...");
            let status = Command::new("docker")
                .args(["start", "milvus-standalone"])
                .status()?;

            if !status.success() {
                bail!("Failed to start Milvus container");
            }
        } else {
            // Create and start new container
            println!("  Creating Milvus container...");
            let status = Command::new("docker")
                .args([
                    "run",
                    "-d",
                    "--name", "milvus-standalone",
                    "-p", &format!("{}:19530", self.config.port),
                    "-p", "9091:9091",
                    "-p", "2379:2379",
                    "-v", "milvus-standalone-etcd:/var/lib/etcd",
                    "-v", "milvus-standalone-data:/var/lib/milvus",
                    "--health-cmd", "curl -f http://localhost:9091/healthz || exit 1",
                    "--health-interval", "30s",
                    "--health-retries", "3",
                    "--health-timeout", "20s",
                    "-e", "ETCD_USE_EMBED=true",
                    "-e", "ETCD_DATA_DIR=/var/lib/etcd",
                    "-e", "COMMON_STORAGETYPE=local",
                    "-e", "MILVUS_LOG_LEVEL=info",
                    "milvusdb/milvus:v2.3.3",
                    "milvus",
                    "run",
                    "standalone"
                ])
                .status()?;

            if !status.success() {
                bail!("Failed to create Milvus container");
            }
        }

        // Wait for Milvus to be ready
        println!("  Waiting for Milvus to be ready...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        println!("✅ Milvus started successfully");
        println!("   gRPC endpoint: {}:{}", self.config.host, self.config.port);
        println!("   Web UI: http://localhost:9091");
        
        Ok(())
    }

    /// Stop Milvus service
    pub async fn stop(&self) -> Result<()> {
        println!("🛑 Stopping Milvus...");

        let status = Command::new("docker")
            .args(["stop", "milvus-standalone"])
            .status()?;

        if !status.success() {
            bail!("Failed to stop Milvus");
        }

        println!("✅ Milvus stopped");
        Ok(())
    }

    /// Initialize Milvus (create default collection if needed)
    pub async fn initialize(&self) -> Result<()> {
        println!("🔧 Milvus initialization:");
        println!("   Host: {}", self.config.host);
        println!("   Port: {}", self.config.port);
        println!("   Collection: {}", self.config.collection_name);
        println!("   Dimension: {}", self.config.dimension);
        println!("   Metric: {}", self.config.metric);
        println!();
        println!("   Note: Collection will be created on first use");
        Ok(())
    }

    /// Reset Milvus (remove container and data)
    pub async fn reset(&self) -> Result<()> {
        println!("⚠️  Resetting Milvus...");
        
        // Stop the container
        let _ = Command::new("docker")
            .args(["stop", "milvus-standalone"])
            .status();

        // Remove the container
        let status = Command::new("docker")
            .args(["rm", "-f", "milvus-standalone"])
            .status()?;

        if !status.success() {
            bail!("Failed to remove Milvus container");
        }

        // Remove volumes
        let _ = Command::new("docker")
            .args(["volume", "rm", "-f", "milvus-standalone-etcd", "milvus-standalone-minio"])
            .status();

        println!("✅ Milvus reset complete (container and data removed)");
        Ok(())
    }

    /// Get service status
    pub fn status(&self) -> ServiceStatus {
        ServiceStatus {
            name: "Milvus".to_string(),
            installed: self.is_installed(),
            running: self.is_running().unwrap_or(false),
            host: self.config.host.clone(),
            port: self.config.port,
            database: Some(self.config.collection_name.clone()),
        }
    }
}
