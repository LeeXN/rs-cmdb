mod api;
mod cache;
mod config;
mod constants;
mod dao;
mod db;
mod middleware;
mod queue;
mod repository;
mod service;
mod validation;
#[cfg(test)]
mod tests;

use anyhow::Result;
use clap::{Arg, Command};
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::config::{get_config, validate_jwt_secret};
use crate::db::redb_store::RedbStore;
use crate::queue::message_queue::MessageQueueFactory;
use crate::repository::{
    client_repository::ClientRepository, component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, hardware_repository::HardwareRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, user_repository::UserRepository,
};
use crate::service::{
    auth_service::{AuthService, validate_password_complexity},
    client_filter_service::ClientFilterService,
    client_service::ClientService,
    component_service::ComponentService,
    hardware_service::HardwareService,
    message_processor::MessageProcessor,
    stats_service::StatsService,
    validation_service::ValidationService,
};
use chrono::Utc;
use common::entity::user::{Role, User};
use std::env;
use uuid::Uuid;

/// Prompt for admin password interactively (for non-automated setups)
fn prompt_admin_password() -> anyhow::Result<String> {
    println!();
    println!("==========================================");
    println!("  CMDB Server - First Time Setup");
    println!("==========================================");
    println!();
    println!("No admin user exists. Please create an admin account.");
    println!("Password requirements:");
    println!("  - At least 12 characters");
    println!("  - At least one uppercase letter (A-Z)");
    println!("  - At least one lowercase letter (a-z)");
    println!("  - At least one number (0-9)");
    println!("  - At least one special character");
    println!();

    // First password entry
    let password1 = rpassword::prompt_password("Enter admin password: ")?;

    // Confirmation
    let password2 = rpassword::prompt_password("Confirm admin password: ")?;

    if password1 != password2 {
        anyhow::bail!("Passwords do not match");
    }

    Ok(password1)
}

/// Ensure admin user exists with secure credentials
///
/// This function checks if an admin user exists. If not, it creates one using:
/// 1. Environment variable CMDB_ADMIN_PASSWORD (for automated setups)
/// 2. Interactive prompt (for manual setups)
///
/// The password must meet complexity requirements before admin creation.
async fn ensure_admin_exists(
    user_repo: &Arc<UserRepository>,
    auth_service: &Arc<AuthService>,
) -> anyhow::Result<()> {
    // Check if admin user already exists
    if let Ok(Some(_)) = user_repo.find_by_username("admin").await {
        info!("Admin user already exists");
        return Ok(());
    }

    info!("No admin user found. Creating admin account...");

    // Try environment variable first (for automated setups)
    let password = match env::var("CMDB_ADMIN_PASSWORD") {
        Ok(pwd) => {
            info!("Using admin password from environment variable");
            pwd
        }
        Err(_) => {
            // Fall back to interactive prompt
            prompt_admin_password()?
        }
    };

    // Validate password complexity
    if let Err(e) = validate_password_complexity(&password) {
        anyhow::bail!(
            "Password validation failed: {}. Please use a password that meets the requirements.",
            e
        );
    }

    // Hash the password
    let password_hash = auth_service.hash_password(&password)?;

    // Create admin user
    let admin = User {
        id: Uuid::new_v4().to_string(),
        username: "admin".to_string(),
        password_hash,
        role: Role::Admin,
        created_at: Utc::now().to_rfc3339(),
        last_login: None,
        is_active: true,
    };

    user_repo.save(&admin).await?;
    info!("Admin user created successfully: username=admin");
    println!();
    println!("==========================================");
    println!("  Admin account created successfully!");
    println!("  Username: admin");
    println!("  You can now log in with your password.");
    println!("==========================================");
    println!();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("rs-cmdb-server")
        .version("0.1.0")
        .about("Configuration Management Database Server")
        .arg(
            Arg::new("host")
                .long("host")
                .short('H')
                .value_name("HOST")
                .help("Server bind address"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .short('p')
                .value_name("PORT")
                .help("Server port"),
        )
        .get_matches();

    // Load configuration
    let mut config = get_config().clone();

    // Validate JWT secret (fail fast if invalid)
    if let Err(e) = validate_jwt_secret(&config.jwt_secret) {
        eprintln!("Configuration validation failed: {}", e);
        eprintln!();
        eprintln!("CRITICAL: JWT secret validation failed!");
        eprintln!("Please set CMDB_JWT_SECRET environment variable to a secure value.");
        eprintln!("Example: export CMDB_JWT_SECRET='your-secure-secret-min-32-chars'");
        eprintln!();
        return Err(anyhow::anyhow!("JWT secret validation failed: {}", e));
    }

    // Override with command line arguments if provided
    if let Some(host) = matches.get_one::<String>("host") {
        config.host = host.clone();
    }

    if let Some(port_str) = matches.get_one::<String>("port") {
        config.port = port_str
            .parse::<u16>()
            .map_err(|_| anyhow::anyhow!("Invalid port number"))?;
    }

    // Initialize logging with tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(&config.log_level))
        .init();

    info!("Starting CMDB server...");

    // Ensure database directory exists
    let db_path = Path::new(&config.database.path);
    if let Some(parent) = db_path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)?;
    }

    // Initialize database
    let db = match RedbStore::new(&config.database.path) {
        Ok(db) => Arc::new(db),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(anyhow::anyhow!("Failed to initialize database: {}", e));
        }
    };
    info!("Database initialized: {}", config.database.path);

    // Initialize repositories
    let client_repo = Arc::new(ClientRepository::new(db.clone()));
    let hardware_repo = Arc::new(HardwareRepository::new(db.clone()));
    let user_repo = Arc::new(UserRepository::new(db.clone()));
    let person_repo = Arc::new(PersonRepository::new(db.clone()));
    let project_repo = Arc::new(ProjectRepository::new(db.clone()));
    let component_repo = Arc::new(ComponentRepository::new(db.clone()));
    let dictionary_repo = Arc::new(DictionaryRepository::new(db.clone()));
    let rack_repo = Arc::new(RackRepository::new(db.clone()));

    // Initialize auth service
    let auth_service = Arc::new(AuthService::new(config.jwt_secret.clone()));

    // Ensure admin user exists with secure credentials
    if let Err(e) = ensure_admin_exists(&user_repo, &auth_service).await {
        error!("Failed to ensure admin exists: {}", e);
        return Err(anyhow::anyhow!("Failed to ensure admin exists: {}", e));
    }

    // Initialize message queue
    let message_queue = MessageQueueFactory::create_flume_queue();

    // Initialize services
    let client_service = Arc::new(ClientService::from_repositories(
        client_repo.clone(),
        hardware_repo.clone(),
        rack_repo.clone(),
    ));
    let component_service = Arc::new(ComponentService::new(component_repo.clone()));
    let hardware_service = Arc::new(HardwareService::new(
        client_repo.clone(),
        hardware_repo.clone(),
        component_service.clone(),
        message_queue.clone(),
    ));
    let validation_service = Arc::new(ValidationService::new(
        client_repo.clone(),
        project_repo.clone(),
        rack_repo.clone(),
        person_repo.clone(),
    ));
    let stats_service = Arc::new(StatsService::new(
        client_repo.clone(),
        hardware_repo.clone(),
    ));
    let client_filter_service = Arc::new(ClientFilterService::new(
        client_repo.clone(),
        hardware_repo.clone(),
    ));

    // Initialize message processor
    let message_processor = Arc::new(MessageProcessor::new(
        message_queue.clone(),
        client_service.clone(),
        hardware_service.clone(),
    ));

    // Start message processor in a separate task
    let processor = message_processor.clone();
    tokio::task::spawn(async move {
        if let Err(e) = processor.start().await {
            error!("Message processor error: {}", e);
        }
    });

    // Setup scheduled tasks
    let scheduler = JobScheduler::new().await?;

    // Add scheduled job for client polling (if implemented)
    let poll_interval = config.poll_interval;
    let hw_service = hardware_service.clone();

    let poll_job = Job::new_async(
        format!("0 */{} * * * *", poll_interval / 60),
        move |_, _| {
            let _hw_svc = hw_service.clone(); // Use underscore to mark as intentionally unused
            Box::pin(async move {
                info!("Running scheduled client polling...");
                // In a real implementation, we would iterate through clients and initiate pull requests
                // This is left as a placeholder
            })
        },
    )?;

    scheduler.add(poll_job).await?;
    scheduler.start().await?;

    // Create router
    let app = api::create_router(
        client_repo,
        hardware_repo,
        user_repo,
        person_repo,
        project_repo,
        component_repo,
        dictionary_repo,
        rack_repo,
        message_queue,
        client_service,
        auth_service,
        validation_service,
        stats_service,
        client_filter_service,
        Arc::new(config.clone()),
    );

    // Start server
    let addr = format!("{}:{}", config.host, config.port);
    let listener = TcpListener::bind(&addr).await?;
    info!("Server listening on {}", addr);

    // Run server
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    info!("Server shutdown complete");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { info!("Received Ctrl+C, starting graceful shutdown..."); },
        _ = terminate => { info!("Received terminate signal, starting graceful shutdown..."); },
    }
}
