mod api;
mod cache;
mod config;
mod constants;
mod dao;
mod db;
mod i18n;
mod middleware;
mod queue;
mod repository;
mod service;
#[cfg(test)]
mod tests;
mod validation;

use anyhow::Result;
use clap::{Arg, Command, arg};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::cache::{CacheConfigs, CachedClientRepository};
use crate::config::{get_config, validate_jwt_secret, validate_primary_ip_config};
use crate::dao::{ClientDao, RackDao};
use crate::db::Database;
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
    export_service::ExportService,
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
        .subcommand(
            Command::new("history")
                .about("History management commands")
                .subcommand_required(true)
                .subcommand(
                    Command::new("analyze")
                        .about("Analyze history storage")
                        .arg(
                            Arg::new("db-path")
                                .long("db-path")
                                .short('d')
                                .value_name("PATH")
                                .help("Database file path (default: from config)"),
                        ),
                )
                .subcommand(
                    Command::new("cleanup")
                        .about("Remove old history entries")
                        .arg(
                            Arg::new("db-path")
                                .long("db-path")
                                .short('d')
                                .value_name("PATH")
                                .help("Database file path (default: from config)"),
                        )
                        .arg(
                            arg!(--"keep-last" <N>)
                                .help("Number of latest entries to keep per client")
                                .required(true),
                        )
                        .arg(
                            arg!(--"dry-run")
                                .help("Only print what would be deleted, don't actually delete"),
                        ),
                )
                .subcommand(
                    Command::new("migrate")
                        .about("Migrate old full-snapshot history to delta format")
                        .arg(
                            Arg::new("db-path")
                                .long("db-path")
                                .short('d')
                                .value_name("PATH")
                                .help("Database file path (default: from config)"),
                        ),
                )
                .subcommand(
                    Command::new("compact")
                        .about("Compact database file, reclaiming unused space")
                        .arg(
                            Arg::new("db-path")
                                .long("db-path")
                                .short('d')
                                .value_name("PATH")
                                .help("Database file path (default: from config)"),
                        ),
                ),
        )
        .get_matches();

    // Handle history subcommands before normal server startup
    if let Some(history_matches) = matches.subcommand_matches("history") {
        let (sub_name, sub_matches) = history_matches
            .subcommand()
            .ok_or_else(|| anyhow::anyhow!("Missing history subcommand"))?;

        let db_path = sub_matches
            .get_one::<String>("db-path")
            .cloned()
            .unwrap_or_else(|| get_config().database.path.clone());

        if !Path::new(&db_path).exists() {
            eprintln!("Database file not found: {}", db_path);
            std::process::exit(1);
        }

        let redb_store = match crate::db::redb_store::RedbStore::new(&db_path) {
            Ok(db) => db,
            Err(e) => {
                eprintln!("Failed to open database: {}", e);
                std::process::exit(1);
            }
        };
        let db: Arc<dyn Database> = Arc::new(redb_store);

        match sub_name {
            "analyze" => {
                run_history_analyze(&db).await?;
            }
            "cleanup" => {
                let keep_last: usize = sub_matches
                    .get_one::<String>("keep-last")
                    .unwrap()
                    .parse()
                    .map_err(|_| anyhow::anyhow!("--keep-last must be a positive integer"))?;
                let dry_run = sub_matches.get_flag("dry-run");
                run_history_cleanup(&db, keep_last, dry_run).await?;
            }
            "migrate" => {
                run_history_migrate(&db).await?;
            }
            "compact" => {
                run_history_compact(&db, &db_path).await?;
            }
            _ => {
                anyhow::bail!("Unknown history subcommand");
            }
        }
        return Ok(());
    }

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

    // Validate primary IP config (parse CIDR, fail fast if invalid)
    let primary_ip_subnet = match validate_primary_ip_config(&config.primary_ip) {
        Ok(subnet) => subnet,
        Err(e) => {
            eprintln!("Configuration validation failed: {}", e);
            eprintln!();
            return Err(anyhow::anyhow!(
                "Primary IP config validation failed: {}",
                e
            ));
        }
    };

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
    let client_repo_inner = Arc::new(ClientRepository::new(db.clone()));
    let cache_configs = CacheConfigs::default();
    let client_repo = Arc::new(CachedClientRepository::new(
        client_repo_inner.clone(),
        &cache_configs,
    ));
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
    // Use DAOs for client service
    let client_dao = Arc::new(ClientDao::new(client_repo.clone(), hardware_repo.clone()));
    let rack_dao = Arc::new(RackDao::new(rack_repo.clone(), client_repo.clone()));

    let client_service = Arc::new(ClientService::new(
        client_dao,
        rack_dao,
        hardware_repo.clone(), // ClientService still keeps a ref to hardware_repo
    ));
    let component_service = Arc::new(ComponentService::new(component_repo.clone()));
    // Note: HardwareService constructor expects ClientRepository, but we have CachedClientRepository.
    // We need to check if HardwareService uses ClientRepository or CachedClientRepository.
    // It likely uses ClientRepository. CachedClientRepository does NOT impl Deref to ClientRepository or a common trait.
    // Let's check HardwareService. It probably takes Arc<ClientRepository>.
    // Since we shadowed `client_repo` with the cached one, we might need the inner one for services that don't support caching yet.
    // BUT, we want to use caching.
    // Let's assume for now HardwareService needs to be updated or we pass the inner repo.
    // Ideally we update HardwareService too. For now let's pass the cached repo if the type matches?
    // CachedClientRepository is NOT ClientRepository.
    // So we should have kept client_repo_inner accessible.
    // Let's use `client_repo` (which is Cached) for ClientService, and `client_repo_inner` for others?
    // Or better, update HardwareService.
    // Let's assume we pass `client_repo_inner` to others to avoid massive refactoring right now,
    // but `ClientService` gets the cached one.

    let hardware_service = Arc::new(HardwareService::new(
        client_repo.clone(),
        hardware_repo.clone(),
        component_service.clone(),
        message_queue.clone(),
        primary_ip_subnet,
    ));
    let validation_service = Arc::new(ValidationService::new(
        client_repo_inner.clone(),
        project_repo.clone(),
        rack_repo.clone(),
        person_repo.clone(),
    ));
    let stats_service = Arc::new(StatsService::new(
        client_repo_inner.clone(),
        hardware_repo.clone(),
    ));
    let client_filter_service = Arc::new(ClientFilterService::new(
        client_repo_inner.clone(),
        hardware_repo.clone(),
    ));
    let export_service = Arc::new(ExportService::new(
        client_repo_inner.clone(),
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
        client_repo_inner,
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
        export_service,
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

/// Extract client_id and timestamp from a history key
fn parse_history_key(key: &str) -> Option<(String, i64)> {
    let prefix = "hardware:";
    let suffix = ":history:";
    let key = key.strip_prefix(prefix)?;
    let (client_id, rest) = key.split_once(suffix)?;
    let timestamp: i64 = rest.parse().ok()?;
    Some((client_id.to_string(), timestamp))
}

/// Analyze history storage
async fn run_history_analyze(db: &Arc<dyn Database>) -> Result<()> {
    let keys = db.list_keys("hardware:").await?;
    let mut buckets: HashMap<String, Vec<(String, i64)>> = HashMap::new();

    for key in &keys {
        if let Some((client_id, timestamp)) = parse_history_key(key) {
            buckets
                .entry(client_id)
                .or_default()
                .push((key.clone(), timestamp));
        }
    }

    let total_snapshots: usize = buckets.values().map(|v| v.len()).sum();
    println!("=== History Analysis ===");
    println!("Clients with history: {}", buckets.len());
    println!("Total history entries: {}", total_snapshots);

    // Sort buckets by history count descending
    let mut sorted: Vec<_> = buckets.into_iter().collect();
    sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    println!("\nTop clients by history count:");
    for (client_id, entries) in sorted.iter().take(20) {
        let timestamps: Vec<i64> = entries.iter().map(|(_, ts)| *ts).collect();
        let oldest = timestamps.iter().min().unwrap_or(&0);
        let newest = timestamps.iter().max().unwrap_or(&0);
        println!(
            "  {}: {} entries (oldest: {}, newest: {})",
            client_id,
            entries.len(),
            oldest,
            newest
        );
    }

    Ok(())
}

/// Cleanup old history entries, keeping only the newest N per client
async fn run_history_cleanup(
    db: &Arc<dyn Database>,
    keep_last: usize,
    dry_run: bool,
) -> Result<()> {
    let keys = db.list_keys("hardware:").await?;
    let mut buckets: HashMap<String, Vec<(String, i64)>> = HashMap::new();

    for key in &keys {
        if let Some((client_id, timestamp)) = parse_history_key(key) {
            buckets
                .entry(client_id)
                .or_default()
                .push((key.clone(), timestamp));
        }
    }

    // Sort each client's entries newest-first
    let mut to_delete = Vec::new();
    for (_client_id, mut entries) in buckets.iter_mut() {
        entries.sort_by(|a, b| b.1.cmp(&a.1));
        if entries.len() > keep_last {
            for (key, _) in entries.iter().skip(keep_last) {
                to_delete.push(key.clone());
            }
        }
    }

    println!("=== History Cleanup ===");
    println!("Dry run: {}", dry_run);
    println!("Entries to delete: {}", to_delete.len());

    if !dry_run {
        for key in &to_delete {
            db.delete(key).await?;
        }
        println!("Cleaned up {} history entries", to_delete.len());
    }

    Ok(())
}

/// Migrate old full-snapshot history entries to new delta format.
///
/// Safety: writes new format BEFORE deleting old entries (same key, overwrite).
/// If interrupted mid-run, partially migrated entries are still readable
/// (read path handles both old and new format). Re-running is idempotent.
async fn run_history_migrate(db: &Arc<dyn Database>) -> Result<()> {
    use common::models::{HardwareHistoryEntry, build_hardware_history_entries};

    let hardware_repo = crate::repository::hardware_repository::HardwareRepository::new(db.clone());

    // Get all client IDs that have history
    let keys = db.list_keys("hardware:").await?;
    let mut client_ids: Vec<String> = keys
        .iter()
        .filter_map(|k| parse_history_key(k))
        .map(|(id, _)| id)
        .collect();
    client_ids.sort();
    client_ids.dedup();

    println!("Found {} clients with history entries", client_ids.len());

    let mut total_migrated_entries = 0;
    let mut total_migrated_clients = 0;
    let mut total_skipped = 0;
    let mut client_count = 0;
    let total_clients = client_ids.len();

    for client_id in &client_ids {
        client_count += 1;
        let history_prefix = format!("hardware:{}:history:", client_id);
        let entries = db.list_entries(&history_prefix).await?;
        let mut has_old_format = false;

        for (_key, data) in &entries {
            if serde_json::from_slice::<common::entity::hardware::Hardware>(data).is_ok() {
                has_old_format = true;
                break;
            }
        }

        if !has_old_format {
            total_skipped += 1;
            if client_count % 10 == 0 || client_count == total_clients {
                println!(
                    "  [{}/{}] processed ({} migrated, {} skipped)...",
                    client_count, total_clients, total_migrated_clients, total_skipped
                );
            }
            continue;
        }

        // Extract old-format snapshots
        let snapshots: Vec<(String, common::entity::hardware::Hardware)> = entries
            .iter()
            .filter_map(|(key, data)| {
                let timestamp = key.strip_prefix(&history_prefix)?.to_string();
                let hw = serde_json::from_slice::<common::entity::hardware::Hardware>(data).ok()?;
                Some((timestamp, hw))
            })
            .collect();

        if snapshots.is_empty() {
            continue;
        }

        let history_entries = build_hardware_history_entries(&snapshots);
        let timestamp_prefix = format!("hardware:{}:history:", client_id);

        println!(
            "  [{}/{}] Migrating client {} ({} entries)...",
            client_count,
            total_clients,
            client_id,
            history_entries.len()
        );

        // Write new format FIRST (overwrite same keys), THEN delete any remaining
        // old-format entries that weren't overwritten.
        let mut written_keys: Vec<String> = Vec::new();
        for entry in &history_entries {
            let key = format!("{}{}", timestamp_prefix, entry.timestamp);
            let mut delta_entry = entry.clone();
            delta_entry.snapshot = None;
            let data = serde_json::to_vec(&delta_entry)?;
            db.set(&key, &data).await?;
            written_keys.push(key);
        }

        total_migrated_entries += history_entries.len();
        total_migrated_clients += 1;

        // Delete any old-format entries that still exist but weren't overwritten
        // (shouldn't happen if all timestamps match, but be safe)
        for (key, data) in &entries {
            if serde_json::from_slice::<common::entity::hardware::Hardware>(data).is_ok()
                && !written_keys.contains(key)
            {
                db.delete(key).await?;
            }
        }
    }

    println!();
    println!("=== History Migration Complete ===");
    println!(
        "Clients migrated: {} ({} total entries)",
        total_migrated_clients, total_migrated_entries
    );
    println!("Clients already in new format: {}", total_skipped);

    Ok(())
}

/// Compact database file by rewriting to a new file, reclaiming unused space.
async fn run_history_compact(db: &Arc<dyn Database>, db_path: &str) -> Result<()> {
    let keys = db.list_keys("").await?;
    println!("Reading {} entries from {}", keys.len(), db_path);

    let mut entries = Vec::with_capacity(keys.len());
    for key in &keys {
        if let Some(value) = db.get(key).await? {
            entries.push((key.clone(), value));
        }
    }
    let tmp_path = format!("{}.compact", db_path);
    let new_store = crate::db::redb_store::RedbStore::new(&tmp_path)?;
    let new_db: Arc<dyn Database> = Arc::new(new_store);
    for (key, value) in &entries {
        new_db.set(key, value).await?;
    }
    drop(new_db);

    std::fs::rename(&tmp_path, db_path)?;
    let new_size = std::fs::metadata(db_path).map(|m| m.len()).unwrap_or(0);
    println!(
        "Compacted: {} entries rewritten, new size: {} bytes ({:.1} MB)",
        entries.len(),
        new_size,
        new_size as f64 / 1_048_576.0
    );

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
