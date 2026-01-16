use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub mysql: MySqlConfig,
    pub neo4j: Neo4jConfig,
    pub milvus: MilvusConfig,
    pub server: ServerConfig,
    pub services: ServicePaths,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub enabled: bool,
    pub allowed_address: String,
    pub jwt_secret: String,
    pub session_duration: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub pool_min: u32,
    pub pool_max: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub bolt_url: String,
    pub http_url: String,
    pub username: String,
    pub password: String,
    pub database: String,
    pub encrypted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilvusConfig {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub collection_name: String,
    pub dimension: usize,
    pub metric: String,
    pub index_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicePaths {
    pub mysql_service: String,
    pub neo4j_home: String,
    pub neo4j_service: String,
    pub mysql_data_dir: String,
    pub neo4j_data_dir: String,
    pub backup_dir: String,
}

impl ServiceConfig {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        let config: ServiceConfig = toml::from_str(&content)
            .context("Failed to parse config file")?;
        Ok(config)
    }

    pub fn load_or_default() -> Result<Self> {
        let config_paths = vec!["config.toml", "./config.toml", "../config.toml"];
        
        for path in config_paths {
            if Path::new(path).exists() {
                return Self::load(path);
            }
        }
        
        anyhow::bail!("Config file not found. Please create config.toml")
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;
        fs::write(path, content)
            .context("Failed to write config file")?;
        Ok(())
    }
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            mysql: MySqlConfig {
                enabled: true,
                host: "localhost".to_string(),
                port: 3306,
                database: "mecp_db".to_string(),
                username: "mecp_user".to_string(),
                password: "mecp_password".to_string(),
                pool_min: 5,
                pool_max: 20,
                connect_timeout: 30,
            },
            neo4j: Neo4jConfig {
                enabled: true,
                host: "localhost".to_string(),
                port: 7687,
                bolt_url: "bolt://localhost:7687".to_string(),
                http_url: "http://localhost:7474".to_string(),
                username: "neo4j".to_string(),
                password: "mecp_neo4j_password".to_string(),
                database: "neo4j".to_string(),
                encrypted: false,
            },
            milvus: MilvusConfig {
                enabled: false,
                host: "localhost".to_string(),
                port: 19530,
                collection_name: "mecp_vectors".to_string(),
                dimension: 384,
                metric: "L2".to_string(),
                index_type: "IVF_FLAT".to_string(),
            },
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                log_level: "info".to_string(),
            },
            services: ServicePaths {
                mysql_service: "mysql".to_string(),
                neo4j_home: "/var/lib/neo4j".to_string(),
                neo4j_service: "neo4j".to_string(),
                mysql_data_dir: "/var/lib/mysql".to_string(),
                neo4j_data_dir: "/var/lib/neo4j/data".to_string(),
                backup_dir: "./backups".to_string(),
            },
            auth: None,
        }
    }
}
