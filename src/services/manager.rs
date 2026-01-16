use anyhow::Result;
use super::config::ServiceConfig;
use super::mysql::MySqlService;
use super::neo4j::Neo4jService;
use super::milvus::MilvusService;

pub struct ServiceManager {
    pub config: ServiceConfig,
    pub mysql: MySqlService,
    pub neo4j: Neo4jService,
    pub milvus: MilvusService,
}

impl ServiceManager {
    pub fn new(config: ServiceConfig) -> Self {
        let mysql = MySqlService::new(config.mysql.clone());
        let neo4j = Neo4jService::new(config.neo4j.clone());
        let milvus = MilvusService::new(config.milvus.clone());

        Self {
            config,
            mysql,
            neo4j,
            milvus,
        }
    }

    /// Check and install all enabled services
    pub async fn install_all(&self) -> Result<()> {
        println!("🔧 Checking and installing services...\n");

        if self.config.mysql.enabled {
            if !self.mysql.is_installed() {
                self.mysql.install().await?;
            } else {
                println!("✅ MySQL already installed");
            }
        }

        if self.config.neo4j.enabled {
            if !self.neo4j.is_installed() {
                self.neo4j.install().await?;
            } else {
                println!("✅ Neo4j already installed");
            }
        }

        if self.config.milvus.enabled {
            if !self.milvus.is_installed() {
                self.milvus.install().await?;
            } else {
                println!("✅ Milvus already installed");
            }
        }

        Ok(())
    }

    /// Start all enabled services
    pub async fn start_all(&self) -> Result<()> {
        println!("🚀 Starting services...\n");

        if self.config.mysql.enabled {
            if !self.mysql.is_running()? {
                self.mysql.start().await?;
                self.mysql.initialize().await?;
            } else {
                println!("✅ MySQL already running");
            }
        }

        if self.config.neo4j.enabled {
            if !self.neo4j.is_running()? {
                self.neo4j.start().await?;
                self.neo4j.initialize().await?;
            } else {
                println!("✅ Neo4j already running");
            }
        }

        if self.config.milvus.enabled {
            if !self.milvus.is_running()? {
                self.milvus.start().await?;
                self.milvus.initialize().await?;
            } else {
                println!("✅ Milvus already running");
            }
        }

        println!("\n✨ All services started successfully!");
        Ok(())
    }

    /// Stop all enabled services
    pub async fn stop_all(&self) -> Result<()> {
        println!("🛑 Stopping services...\n");

        if self.config.mysql.enabled && self.mysql.is_running()? {
            self.mysql.stop().await?;
        }

        if self.config.neo4j.enabled && self.neo4j.is_running()? {
            self.neo4j.stop().await?;
        }

        if self.config.milvus.enabled && self.milvus.is_running()? {
            self.milvus.stop().await?;
        }

        println!("\n✨ All services stopped!");
        Ok(())
    }

    /// Reset all enabled services to clean state
    pub async fn reset_all(&self) -> Result<()> {
        println!("🔄 Resetting all services to clean state...\n");
        println!("⚠️  This will delete all data in the databases!");
        
        if self.config.mysql.enabled {
            self.mysql.reset().await?;
        }

        if self.config.neo4j.enabled {
            self.neo4j.reset().await?;
        }

        if self.config.milvus.enabled {
            self.milvus.reset().await?;
        }

        println!("\n✨ All services reset complete!");
        Ok(())
    }

    /// Show status of all services
    pub fn status_all(&self) {
        println!("📊 Service Status\n");
        println!("════════════════════════════════════════");

        if self.config.mysql.enabled {
            self.mysql.status().print();
            println!();
        }

        if self.config.neo4j.enabled {
            self.neo4j.status().print();
            println!();
        }

        if self.config.milvus.enabled {
            self.milvus.status().print();
            println!();
        }

        println!("════════════════════════════════════════");
    }

    /// Install specific service
    pub async fn install_service(&self, service_name: &str) -> Result<()> {
        match service_name.to_lowercase().as_str() {
            "mysql" => {
                if !self.mysql.is_installed() {
                    self.mysql.install().await
                } else {
                    println!("✅ MySQL already installed");
                    Ok(())
                }
            }
            "neo4j" => {
                if !self.neo4j.is_installed() {
                    self.neo4j.install().await
                } else {
                    println!("✅ Neo4j already installed");
                    Ok(())
                }
            }
            "milvus" => {
                if !self.milvus.is_installed() {
                    self.milvus.install().await
                } else {
                    println!("✅ Milvus already installed");
                    Ok(())
                }
            }
            _ => anyhow::bail!("Unknown service: {}", service_name),
        }
    }

    /// Start specific service
    pub async fn start_service(&self, service_name: &str) -> Result<()> {
        match service_name.to_lowercase().as_str() {
            "mysql" => {
                if !self.mysql.is_running()? {
                    self.mysql.start().await?;
                    self.mysql.initialize().await
                } else {
                    println!("✅ MySQL already running");
                    Ok(())
                }
            }
            "neo4j" => {
                if !self.neo4j.is_running()? {
                    self.neo4j.start().await?;
                    self.neo4j.initialize().await
                } else {
                    println!("✅ Neo4j already running");
                    Ok(())
                }
            }
            "milvus" => {
                if !self.milvus.is_running()? {
                    self.milvus.start().await?;
                    self.milvus.initialize().await
                } else {
                    println!("✅ Milvus already running");
                    Ok(())
                }
            }
            _ => anyhow::bail!("Unknown service: {}", service_name),
        }
    }

    /// Stop specific service
    pub async fn stop_service(&self, service_name: &str) -> Result<()> {
        match service_name.to_lowercase().as_str() {
            "mysql" => self.mysql.stop().await,
            "neo4j" => self.neo4j.stop().await,
            "milvus" => self.milvus.stop().await,
            _ => anyhow::bail!("Unknown service: {}", service_name),
        }
    }

    /// Reset specific service
    pub async fn reset_service(&self, service_name: &str) -> Result<()> {
        match service_name.to_lowercase().as_str() {
            "mysql" => self.mysql.reset().await,
            "neo4j" => self.neo4j.reset().await,
            "milvus" => self.milvus.reset().await,
            _ => anyhow::bail!("Unknown service: {}", service_name),
        }
    }
}
