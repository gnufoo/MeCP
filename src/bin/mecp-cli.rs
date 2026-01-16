use anyhow::Result;
use clap::{Parser, Subcommand};
use mecp::services::{ServiceConfig, ServiceManager};
use colored::*;

#[derive(Parser)]
#[command(name = "mecp-cli")]
#[command(author = "MeCP Team")]
#[command(version = "0.1.0")]
#[command(about = "MeCP Database Service Manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Path to configuration file
    #[arg(short, long, default_value = "config.toml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Show status of all services
    Status {
        /// Show status for specific service only (mysql, neo4j, milvus)
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Start database services
    Start {
        /// Start specific service only (mysql, neo4j, milvus)
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Stop database services
    Stop {
        /// Stop specific service only (mysql, neo4j, milvus)
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Shutdown all services (alias for stop)
    Shutdown {
        /// Shutdown specific service only (mysql, neo4j, milvus)
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Reset databases to clean state (WARNING: deletes all data)
    Reset {
        /// Reset specific service only (mysql, neo4j, milvus)
        #[arg(short, long)]
        service: Option<String>,

        /// Skip confirmation prompt
        #[arg(short = 'y', long)]
        yes: bool,
    },

    /// Install required services
    Install {
        /// Install specific service only (mysql, neo4j, milvus)
        #[arg(short, long)]
        service: Option<String>,
    },

    /// Check configuration and service health
    Check,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Print banner
    print_banner();

    // Load configuration
    let config = match ServiceConfig::load(&cli.config) {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("{}", format!("❌ Failed to load config from '{}': {}", cli.config, e).red());
            eprintln!("{}", "   Run with --config <path> to specify a different config file".yellow());
            std::process::exit(1);
        }
    };

    let manager = ServiceManager::new(config);

    // Execute command
    match cli.command {
        Commands::Status { service } => {
            if let Some(svc) = service {
                show_service_status(&manager, &svc);
            } else {
                manager.status_all();
            }
        }

        Commands::Start { service } => {
            if let Some(svc) = service {
                manager.start_service(&svc).await?;
            } else {
                // Check and install if needed
                manager.install_all().await?;
                manager.start_all().await?;
            }
        }

        Commands::Stop { service } | Commands::Shutdown { service } => {
            if let Some(svc) = service {
                manager.stop_service(&svc).await?;
            } else {
                manager.stop_all().await?;
            }
        }

        Commands::Reset { service, yes } => {
            if !yes {
                println!("{}", "⚠️  WARNING: This will DELETE ALL DATA in the databases!".red().bold());
                println!("Are you sure you want to continue? (type 'yes' to confirm)");
                
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;
                
                if input.trim().to_lowercase() != "yes" {
                    println!("{}", "❌ Reset cancelled".yellow());
                    return Ok(());
                }
            }

            if let Some(svc) = service {
                manager.reset_service(&svc).await?;
            } else {
                manager.reset_all().await?;
            }
        }

        Commands::Install { service } => {
            if let Some(svc) = service {
                manager.install_service(&svc).await?;
            } else {
                manager.install_all().await?;
            }
        }

        Commands::Check => {
            println!("{}", "🔍 Checking configuration and services...\n".cyan());
            
            // Show config file location
            println!("📄 Configuration file: {}", cli.config.green());
            println!();

            // Check each service
            check_service_health(&manager).await;
        }
    }

    Ok(())
}

fn print_banner() {
    println!("{}", "╔════════════════════════════════════════╗".cyan());
    println!("{}", "║     MeCP Service Manager CLI v0.1      ║".cyan());
    println!("{}", "╚════════════════════════════════════════╝".cyan());
    println!();
}

fn show_service_status(manager: &ServiceManager, service_name: &str) {
    println!("📊 Service Status: {}\n", service_name.green());
    println!("════════════════════════════════════════");

    match service_name.to_lowercase().as_str() {
        "mysql" => manager.mysql.status().print(),
        "neo4j" => manager.neo4j.status().print(),
        "milvus" => manager.milvus.status().print(),
        _ => {
            eprintln!("{}", format!("❌ Unknown service: {}", service_name).red());
            eprintln!("   Available services: mysql, neo4j, milvus");
        }
    }

    println!("════════════════════════════════════════");
}

async fn check_service_health(manager: &ServiceManager) {
    // MySQL
    if manager.config.mysql.enabled {
        println!("🔍 MySQL:");
        println!("   Installed: {}", format_bool(manager.mysql.is_installed()));
        println!("   Running:   {}", format_bool(manager.mysql.is_running().unwrap_or(false)));
        println!("   Host:      {}:{}", manager.config.mysql.host, manager.config.mysql.port);
        println!("   Database:  {}", manager.config.mysql.database);
        println!();
    }

    // Neo4j
    if manager.config.neo4j.enabled {
        println!("🔍 Neo4j:");
        println!("   Installed: {}", format_bool(manager.neo4j.is_installed()));
        println!("   Running:   {}", format_bool(manager.neo4j.is_running().unwrap_or(false)));
        println!("   Bolt URL:  {}", manager.config.neo4j.bolt_url);
        println!("   HTTP URL:  {}", manager.config.neo4j.http_url);
        println!();
    }

    // Milvus
    if manager.config.milvus.enabled {
        println!("🔍 Milvus:");
        println!("   Installed:   {}", format_bool(manager.milvus.is_installed()));
        println!("   Running:     {}", format_bool(manager.milvus.is_running().unwrap_or(false)));
        println!("   Host:        {}", manager.config.milvus.host);
        println!("   Port:        {}", manager.config.milvus.port);
        println!("   Collection:  {}", manager.config.milvus.collection_name);
        println!();
    }

    println!("{}", "✅ Health check complete".green());
}

fn format_bool(value: bool) -> String {
    if value {
        "✅ Yes".green().to_string()
    } else {
        "❌ No".red().to_string()
    }
}
