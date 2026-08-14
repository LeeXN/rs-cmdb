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
mod tls;
mod validation;

use anyhow::Result;
use clap::{Arg, Command, arg};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::signal;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{Level, debug, error, info};

use crate::cache::{CacheConfigs, CachedClientRepository};
use crate::config::{get_config, validate_primary_ip_config};
use crate::dao::{ClientDao, RackDao};
use crate::db::Database;
use crate::db::redb_store::RedbStore;
use crate::queue::message_queue::MessageQueueFactory;
use crate::repository::{
    approval_repository::ApprovalRepository, client_repository::ClientRepository,
    command_repository::CommandRepository, component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, exec_policy_repository::ExecPolicyRepository,
    execution_session_repository::ExecutionSessionRepository,
    hardware_repository::HardwareRepository, permission_repository::PermissionRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, terminal_session_repository::TerminalSessionRepository,
    user_repository::UserRepository, web_terminal_policy_repository::WebTerminalPolicyRepository,
};
use crate::service::{
    approval_service::ApprovalService,
    auth_service::{AuthService, init_master_client_key_from_db, validate_password_complexity},
    client_filter_service::ClientFilterService,
    client_service::ClientService,
    command_service::CommandService,
    component_service::ComponentService,
    danger_detection::DangerDetectionService,
    exec_policy_engine::ExecPolicyEngine,
    execution_session_service::ExecutionSessionService,
    export_service::ExportService,
    hardware_service::HardwareService,
    message_processor::MessageProcessor,
    permission_service::PermissionService,
    sse_hub::SseHub,
    stats_service::StatsService,
    terminal_session_service::TerminalSessionService,
    token_blacklist::TokenBlacklist,
    validation_service::ValidationService,
    web_terminal_service::WebTerminalService,
};
use chrono::Utc;
use common::entity::user::{Role, User};
use std::env;
use uuid::Uuid;

fn mask_secret(secret: &str) -> String {
    if secret.is_empty() {
        return "<empty>".to_string();
    }
    if secret.len() <= 8 {
        return "<redacted>".to_string();
    }
    format!("{}***{}", &secret[..4], &secret[secret.len() - 4..])
}

fn log_effective_config(config: &config::ServerConfig) {
    if !tracing::enabled!(Level::DEBUG) {
        return;
    }

    let snapshot = serde_json::json!({
        "host": config.host,
        "port": config.port,
        "database": {
            "db_type": config.database.db_type,
            "path": config.database.path,
        },
        "queue": {
            "queue_type": config.queue.queue_type,
            "capacity": config.queue.capacity,
        },
        "primary_ip": config.primary_ip,
        "poll_interval": config.poll_interval,
        "client_timeout": config.client_timeout,
        "log_level": config.log_level,
        "enable_tls": config.enable_tls,
        "tls_cert": config.tls_cert,
        "tls_key": config.tls_key,
        "jwt_secret": mask_secret(&config.jwt_secret),
        "component_missing_grace_period_hours": config.component_missing_grace_period_hours,
        "ssh_known_hosts_file": config.ssh_known_hosts_file,
        "cors_allowed_origins": config.cors_allowed_origins,
        "max_batch_size": config.max_batch_size,
        "expose_version": config.expose_version,
    });

    debug!(config = %snapshot, "Effective server configuration loaded");
}

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

    // Auto-generate or load JWT secret
    {
        let jwt_secret_path = {
            let home = env::var("HOME").unwrap_or_else(|_| "/etc/rs-cmdb".to_string());
            let base = Path::new(&home);
            if home.starts_with('/') && base.is_absolute() {
                base.join(".rs-cmdb").join("jwt_secret")
            } else {
                Path::new("/etc/rs-cmdb/jwt_secret").to_path_buf()
            }
        };

        // If CMDB_JWT_SECRET env var is set, it takes precedence
        if env::var("CMDB_JWT_SECRET").is_ok() {
            info!("Using JWT secret from environment variable CMDB_JWT_SECRET");
        } else if jwt_secret_path.exists() {
            let stored = fs::read_to_string(&jwt_secret_path)
                .map_err(|e| anyhow::anyhow!("Failed to read JWT secret file: {}", e))?;
            let stored = stored.trim().to_string();
            if stored.len() >= 32 {
                info!("Loaded JWT secret from {}", jwt_secret_path.display());
                config.jwt_secret = stored;
            }
        }

        // If still not set or too short, auto-generate and persist
        if config.jwt_secret.len() < 32 {
            use rand::Rng;
            let secret: String = rand::thread_rng()
                .sample_iter(&rand::distributions::Alphanumeric)
                .take(64)
                .map(char::from)
                .collect();

            // Ensure parent directory exists
            if let Some(parent) = jwt_secret_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&jwt_secret_path, &secret)?;
            // Set permissions to 0o600 (owner read/write only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&jwt_secret_path, fs::Permissions::from_mode(0o600))?;
            }
            info!("Generated new JWT secret at {}", jwt_secret_path.display());
            config.jwt_secret = secret;
        }
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
    log_effective_config(&config);

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

    // Initialize token blacklist and auth service
    let token_blacklist = Arc::new(TokenBlacklist::new(db.clone()));
    let auth_service = Arc::new(
        AuthService::new(config.jwt_secret.clone()).with_blacklist(token_blacklist.clone()),
    );

    // Initialize the durable master key used for HMAC-based agent token
    // generation. It must survive server restarts so registered agents keep
    // working without receiving a new token on every boot.
    if let Err(e) = init_master_client_key_from_db(db.clone()).await {
        error!("Failed to initialize master client key: {}", e);
        return Err(anyhow::anyhow!(
            "Failed to initialize master client key: {}",
            e
        ));
    }

    // Ensure admin user exists with secure credentials
    if let Err(e) = ensure_admin_exists(&user_repo, &auth_service).await {
        error!("Failed to ensure admin exists: {}", e);
        return Err(anyhow::anyhow!("Failed to ensure admin exists: {}", e));
    }

    // Initialize message queue
    let message_queue = MessageQueueFactory::create_flume_queue();

    // Initialize permission engine
    let perm_repo = Arc::new(PermissionRepository::new(db.clone()));
    let perm_svc = Arc::new(PermissionService::new(perm_repo.clone()));
    if let Err(e) = perm_svc.ensure_default_rules().await {
        tracing::warn!("Failed to ensure default permission rules: {}", e);
    }
    if let Err(e) = perm_svc.refresh_cache().await {
        tracing::warn!("Failed to refresh permission cache: {}", e);
    }

    // Initialize services
    // Use DAOs for client service
    let client_dao = Arc::new(ClientDao::new(client_repo.clone(), hardware_repo.clone()));
    let rack_dao = Arc::new(RackDao::new(rack_repo.clone(), client_repo.clone()));

    let client_service = Arc::new(
        ClientService::new(
            client_dao,
            rack_dao,
            hardware_repo.clone(), // ClientService still keeps a ref to hardware_repo
        )
        .with_queue(message_queue.clone()),
    );
    let component_service = Arc::new(ComponentService::with_queue(
        component_repo.clone(),
        message_queue.clone(),
    ));
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

    // Initialize remote command execution services
    let command_repo = Arc::new(CommandRepository::new(db.clone()));
    let danger_svc = Arc::new(DangerDetectionService::new());

    // Initialize message processor
    let message_processor = Arc::new(MessageProcessor::new(
        message_queue.clone(),
        client_service.clone(),
        hardware_service.clone(),
        command_repo.clone(),
    ));
    let sse_hub = SseHub::new();
    let exec_policy_repo = Arc::new(ExecPolicyRepository::new(db.clone()));
    let exec_policy_engine = Arc::new(
        ExecPolicyEngine::new(exec_policy_repo.clone()).with_client_repo(client_repo_inner.clone()),
    );
    let web_terminal_policy_repo = Arc::new(WebTerminalPolicyRepository::new(db.clone()));
    let web_terminal_service = Arc::new(
        WebTerminalService::new(web_terminal_policy_repo.clone())
            .with_client_repo(client_repo_inner.clone()),
    );
    let approval_repo = Arc::new(ApprovalRepository::new(db.clone()));
    let approval_svc = Arc::new(ApprovalService::new(
        approval_repo.clone(),
        command_repo.clone(),
    ));
    let execution_session_repo = Arc::new(ExecutionSessionRepository::new(db.clone()));
    let terminal_session_repo = Arc::new(TerminalSessionRepository::new(db.clone()));
    let session_svc = Arc::new(ExecutionSessionService::new(
        execution_session_repo,
        client_repo_inner.clone(),
    ));
    let terminal_svc = TerminalSessionService::new(terminal_session_repo, web_terminal_service);
    terminal_svc.configure_history(session_svc.clone());
    let _ = service::cast_recorder::CastRecorderInner::ensure_cast_dir();
    let cast_recorder = std::sync::Arc::new(service::cast_recorder::CastRecorderInner);

    let command_service = Arc::new(
        CommandService::new(command_repo.clone(), danger_svc.clone(), sse_hub.clone())
            .await
            .expect("Failed to initialize CommandService")
            .with_policy_engine(exec_policy_engine.clone())
            .with_approval_svc(approval_svc.clone())
            .with_session_svc(session_svc.clone())
            .with_cast_recorder(cast_recorder),
    );

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

    // Scheduled job: expire pending tasks that exceeded their deadline (every minute)
    let cmd_svc_expire = command_service.clone();
    let expire_job = Job::new_async("0 * * * * *", move |_, _| {
        let svc = cmd_svc_expire.clone();
        Box::pin(async move {
            match svc.expire_pending_tasks().await {
                Ok(n) if n > 0 => info!("Expired {} pending command tasks", n),
                Err(e) => error!("expire_pending_tasks: {}", e),
                _ => {}
            }
            match svc.expire_running_tasks().await {
                Ok(n) if n > 0 => info!("Timed out {} running command tasks", n),
                Err(e) => error!("expire_running_tasks: {}", e),
                _ => {}
            }
        })
    })?;
    scheduler.add(expire_job).await?;

    // Close terminal sessions that were never claimed or have stopped
    // heartbeating, releasing their in-memory concurrency slots.
    let terminal_svc_cleanup = terminal_svc.clone();
    let terminal_cleanup_job = Job::new_async("0 * * * * *", move |_, _| {
        let svc = terminal_svc_cleanup.clone();
        Box::pin(async move {
            if let Err(e) = svc.list_all_sessions().await {
                error!("terminal session cleanup: {}", e);
            }
        })
    })?;
    scheduler.add(terminal_cleanup_job).await?;

    // Scheduled job: clean up old command history (daily at 03:00)
    let cmd_svc_cleanup = command_service.clone();
    let cleanup_job = Job::new_async("0 0 3 * * *", move |_, _| {
        let svc = cmd_svc_cleanup.clone();
        Box::pin(async move {
            match svc.cleanup_old_tasks(90).await {
                Ok(n) if n > 0 => info!("Cleaned up {} old command tasks (>90 days)", n),
                Err(e) => error!("cleanup_old_tasks: {}", e),
                _ => {}
            }
        })
    })?;
    scheduler.add(cleanup_job).await?;

    // Scheduled job: clean up expired token blacklist entries (hourly)
    let bl_cleanup = token_blacklist.clone();
    let bl_cleanup_job = Job::new_async("0 0 * * * *", move |_, _| {
        let bl = bl_cleanup.clone();
        Box::pin(async move {
            let n = bl.cleanup_expired().await;
            if n > 0 {
                info!("Cleaned up {} expired token blacklist entries", n);
            }
        })
    })?;
    scheduler.add(bl_cleanup_job).await?;

    // Scheduled job: expire pending approvals (every 5 minutes)
    let approval_svc_expire = approval_svc.clone();
    let approval_expire_job = Job::new_async("0 */5 * * * *", move |_, _| {
        let svc = approval_svc_expire.clone();
        Box::pin(async move {
            match svc.expire_old().await {
                Ok(n) if n > 0 => info!("Expired {} pending approval requests", n),
                Err(e) => error!("expire pending approvals: {}", e),
                _ => {}
            }
        })
    })?;
    scheduler.add(approval_expire_job).await?;

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
        command_service,
        sse_hub,
        perm_svc,
        perm_repo,
        exec_policy_repo,
        web_terminal_policy_repo,
        approval_repo,
        approval_svc,
        session_svc,
        terminal_svc,
    );

    // Configure TLS
    let tls_config = crate::tls::load_or_generate_tls_config(
        &config.tls_cert,
        &config.tls_key,
        config.enable_tls,
    )?;

    let addr = format!("{}:{}", config.host, config.port);

    match tls_config {
        Some(tls_config) => {
            info!("TLS enabled - starting HTTPS server on {}", addr);
            let listener = crate::tls::TlsListener::bind(&addr, tls_config).await?;
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        }
        None => {
            info!("Server listening on {} (HTTP)", addr);
            let listener = TcpListener::bind(&addr).await?;
            axum::serve(listener, app)
                .with_graceful_shutdown(shutdown_signal())
                .await?;
        }
    }

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
    sorted.sort_by_key(|entry| std::cmp::Reverse(entry.1.len()));

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
    for (_client_id, entries) in buckets.iter_mut() {
        entries.sort_by_key(|entry| std::cmp::Reverse(entry.1));
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
    use common::models::build_hardware_history_entries;

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
        let mut snapshots: Vec<(String, common::entity::hardware::Hardware)> = entries
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

        snapshots.sort_by(|a, b| b.0.cmp(&a.0));
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
