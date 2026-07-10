pub mod agent_command_api;
pub mod agent_terminal_api;
pub mod auth_api;
mod client_api;
pub mod command_api;
pub mod component_api;
pub mod dictionary_api;
mod download_api;
mod hardware_api;
mod health_api;
pub mod person_api;
pub mod project_api;
pub mod rack_api;
pub mod stats_api;
pub mod approval_api;
pub mod cast_api;
pub mod exec_policy_api;
pub mod permission_admin_api;
pub mod remote_exec_ops_api;
pub mod user_api;
pub mod web_terminal_policy_api;
pub mod terminal_session_api;

use std::sync::Arc;

use crate::config::ServerConfig;
use crate::repository::exec_policy_repository::ExecPolicyRepository;
use crate::repository::approval_repository::ApprovalRepository;
use crate::repository::permission_repository::PermissionRepository;
use crate::repository::web_terminal_policy_repository::WebTerminalPolicyRepository;
use crate::service::approval_service::ApprovalService;
use crate::service::execution_session_service::ExecutionSessionService;
use crate::service::terminal_session_service::TerminalSessionService;
use crate::middleware::agent_auth::agent_auth_middleware;
use crate::middleware::auth::auth_middleware;
use crate::middleware::permission::permission_middleware;
use crate::middleware::rate_limit::rate_limit_middleware;
use crate::middleware::rate_limit::{make_limiter, strategies};
use crate::middleware::rbac;
use crate::queue::MessageQueue;
use crate::repository::{
    client_repository::ClientRepository,
    component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, hardware_repository::HardwareRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, user_repository::UserRepository,
};
use crate::service::{
    auth_service::AuthService, client_filter_service::ClientFilterService,
    client_service::ClientService, command_service::CommandService,
    export_service::ExportService, permission_service::PermissionService, sse_hub::SseHub,
    stats_service::StatsService, validation_service::ValidationService,
};
use axum::{
    Router,
    extract::Extension,
    handler::HandlerWithoutStateExt,
    http::StatusCode,
    middleware,
    response::Html,
    response::IntoResponse,
    routing::{delete, get, post, put},
};
use tower_http::cors::{Any, CorsLayer, AllowOrigin};
use tower_http::services::ServeDir;
use tracing::error;

async fn serve_index_html() -> impl IntoResponse {
    match tokio::fs::read_to_string("dist/index.html").await {
        Ok(content) => Html(content).into_response(),
        Err(e) => {
            error!("Failed to load index.html: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load index.html".to_string(),
            )
                .into_response()
        }
    }
}

/// Create the main router for the API
pub fn create_router(
    client_repo: Arc<ClientRepository>,
    hardware_repo: Arc<HardwareRepository>,
    user_repo: Arc<UserRepository>,
    person_repo: Arc<PersonRepository>,
    project_repo: Arc<ProjectRepository>,
    component_repo: Arc<ComponentRepository>,
    dictionary_repo: Arc<DictionaryRepository>,
    rack_repo: Arc<RackRepository>,
    message_queue: Arc<dyn MessageQueue>,
    client_service: Arc<ClientService>,
    auth_service: Arc<AuthService>,
    validation_service: Arc<ValidationService>,
    stats_service: Arc<StatsService>,
    client_filter_service: Arc<ClientFilterService>,
    export_service: Arc<ExportService>,
    config: Arc<ServerConfig>,
    command_service: Arc<CommandService>,
    sse_hub: Arc<SseHub>,
    perm_svc: Arc<PermissionService>,
    permission_repo: Arc<PermissionRepository>,
    exec_policy_repo: Arc<ExecPolicyRepository>,
    web_terminal_policy_repo: Arc<WebTerminalPolicyRepository>,
    approval_repo: Arc<ApprovalRepository>,
    approval_svc: Arc<ApprovalService>,
    session_svc: Arc<ExecutionSessionService>,
    terminal_svc: Arc<TerminalSessionService>,
) -> Router {
    // Create the CORS layer from config whitelist
    let cors_origins: Vec<axum::http::HeaderValue> = config
        .cors_allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    let cors = if cors_origins.is_empty() {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(cors_origins))
            .allow_methods(Any)
            .allow_headers(Any)
    };

    // 创建静态文件服务
    let static_files_service = ServeDir::new("dist");

    // Rate-limited login route (public, 10/min/IP)
    let login_route = Router::new()
        .route("/api/v1/auth/login", post(auth_api::login))
        .route_layer(middleware::from_fn_with_state(
            make_limiter(strategies::login()),
            rate_limit_middleware,
        ));

    // Public routes
    let public_routes = Router::new()
        .route("/api/v1/health", get(health_api::health_check))
        .route("/api/v1/version", get(health_api::version))
        // Download routes (usually public for agents)
        .route("/api/v1/download/info", get(download_api::get_client_info))
        .route(
            "/api/v1/download/client/{platform}/{arch}/{binary_name}",
            get(download_api::download_client),
        )
        .route("/install.sh", get(download_api::get_install_script))
        // Client registration (agent) — remains public for first-time registration
        .route(
            "/api/v1/clients/register",
            post(client_api::register_client),
        )
        .route("/api/v1/auth/refresh", post(auth_api::refresh))
        .merge(login_route);

    // Agent routes — require agent Bearer token auth
    let agent_routes = Router::new()
        .route("/api/v1/agent/commands/pending", get(agent_command_api::poll_pending))
        .route("/api/v1/agent/commands/{id}/start", post(agent_command_api::start_command))
        .route("/api/v1/agent/commands/{id}/logs", post(agent_command_api::push_logs))
        .route("/api/v1/agent/commands/{id}/complete", post(agent_command_api::complete_command))
        .route("/api/v1/agent/terminal-sessions/stream", get(agent_terminal_api::stream_terminal))
        .route("/api/v1/agent/terminal-sessions/pending", get(agent_terminal_api::poll_pending))
        .route("/api/v1/agent/terminal-sessions/{id}/output", post(agent_terminal_api::push_output))
        .route("/api/v1/agent/terminal-sessions/{id}/state", post(agent_terminal_api::report_state))
        // Hardware push (agent) — requires authentication and client_id matching
        .route(
            "/api/v1/clients/{id}/hardware",
            post(hardware_api::update_hardware),
        )
        .route_layer(middleware::from_fn(agent_auth_middleware));



    // Rate-limited register route (Admin-only, 5/hour/IP)
    let register_route = Router::new()
        .route("/api/v1/auth/register", post(auth_api::register))
        .route_layer(middleware::from_fn_with_state(
            make_limiter(strategies::register()),
            rate_limit_middleware,
        ))
        .route_layer(middleware::from_fn(rbac::require_admin));

    // Admin routes
    let admin_routes = Router::new()
        .route("/api/v1/accounts", get(user_api::list_users))
        .route(
            "/api/v1/accounts/{id}",
            delete(user_api::delete_user).put(user_api::update_user),
        )
        // Remote exec config (Admin only)
        .route("/api/v1/remote-exec/config", put(command_api::update_config))
        // Exec policy CRUD (Admin only)
        .route("/api/v1/permissions/exec-policies", get(exec_policy_api::list_exec_policies).post(exec_policy_api::create_exec_policy))
        .route("/api/v1/permissions/exec-policies/{id}", get(exec_policy_api::get_exec_policy).put(exec_policy_api::update_exec_policy).delete(exec_policy_api::delete_exec_policy))
        .route("/api/v1/permissions/overview", get(permission_admin_api::get_permission_overview))
        .route("/api/v1/permissions/rules", get(permission_admin_api::list_rules).post(permission_admin_api::create_rule))
        .route("/api/v1/permissions/rules/{id}", get(permission_admin_api::get_rule).put(permission_admin_api::update_rule).delete(permission_admin_api::delete_rule))
        .route("/api/v1/permissions/groups", get(permission_admin_api::list_groups).post(permission_admin_api::create_group))
        .route("/api/v1/permissions/groups/{id}", get(permission_admin_api::get_group).put(permission_admin_api::update_group).delete(permission_admin_api::delete_group))
        // Web terminal policy CRUD (Admin only)
        .route("/api/v1/permissions/web-terminal-policies", get(web_terminal_policy_api::list_web_terminal_policies).post(web_terminal_policy_api::create_web_terminal_policy))
        .route("/api/v1/permissions/web-terminal-policies/{id}", get(web_terminal_policy_api::get_web_terminal_policy).put(web_terminal_policy_api::update_web_terminal_policy).delete(web_terminal_policy_api::delete_web_terminal_policy))
        // Approval management (Admin only)
        .route("/api/v1/permissions/pending-approvals", get(approval_api::list_pending_approvals))
        .route("/api/v1/permissions/approval-summary", get(approval_api::get_approval_summary))
        .route("/api/v1/permissions/pending-approvals/{id}", get(approval_api::get_pending_approval))
        .route("/api/v1/permissions/pending-approvals/{id}/approve", post(approval_api::approve_request))
        .route("/api/v1/permissions/pending-approvals/{id}/reject", post(approval_api::reject_request))
        .route("/api/v1/remote-exec/ops/overview", get(remote_exec_ops_api::get_overview))
        .route("/api/v1/remote-exec/terminal-ops/summary", get(remote_exec_ops_api::get_terminal_summary))
        .route("/api/v1/remote-exec/terminal-sessions/all", get(remote_exec_ops_api::list_terminal_sessions))
        .route("/api/v1/remote-exec/casts/cleanup", post(remote_exec_ops_api::cleanup_casts))
        .route_layer(middleware::from_fn(rbac::require_admin));

    // Write routes (User & Admin) — entity CRUD, requires User+ role
    let user_write_routes = Router::new()
        .route("/api/v1/clients/import", post(client_api::import_clients))
        .route(
            "/api/v1/clients/{id}",
            put(client_api::update_client).delete(client_api::delete_client),
        )
        .route("/api/v1/users", post(person_api::create_person))
        .route(
            "/api/v1/users/{id}",
            put(person_api::update_person).delete(person_api::delete_person),
        )
        .route("/api/v1/projects", post(project_api::create_project))
        .route(
            "/api/v1/projects/{id}",
            put(project_api::update_project).delete(project_api::delete_project),
        )
        .route("/api/v1/components", post(component_api::create_component))
        .route(
            "/api/v1/components/batch/create",
            post(component_api::batch_create_components),
        )
        .route(
            "/api/v1/components/batch/delete",
            post(component_api::batch_delete_components),
        )
        .route(
            "/api/v1/components/batch/update",
            post(component_api::batch_update_components),
        )
        .route(
            "/api/v1/components/{id}",
            put(component_api::update_component),
        )
        .route(
            "/api/v1/dictionaries",
            post(dictionary_api::create_dictionary),
        )
        .route(
            "/api/v1/dictionaries/{id}",
            put(dictionary_api::update_dictionary).delete(dictionary_api::delete_dictionary),
        )
        .route(
            "/api/v1/clients/{id}/primary-ip",
            put(client_api::update_client_primary_ip),
        )
        .route("/api/v1/racks", post(rack_api::create_rack))
        .route(
            "/api/v1/racks/{id}",
            put(rack_api::update_rack).delete(rack_api::delete_rack),
        )
        .route(
            "/api/v1/clients/{id}/hardware/pull",
            post(hardware_api::pull_hardware),
        )
        .route_layer(middleware::from_fn(rbac::require_user));

    // Admin-only write routes
    let admin_write_routes = Router::new()
        .route("/api/v1/remote-exec/commands", post(command_api::create_command))
        .route_layer(middleware::from_fn(rbac::require_admin));

    // Rate-limited change-password route (5/hour/IP)
    let change_password_route = Router::new()
        .route(
            "/api/v1/auth/change-password",
            post(auth_api::change_password),
        )
        .route_layer(middleware::from_fn_with_state(
            make_limiter(strategies::change_password()),
            rate_limit_middleware,
        ));

    // Read routes (All authenticated)
    let read_routes = Router::new()
        .route("/api/v1/auth/me", get(auth_api::me))
        .route("/api/v1/clients", get(client_api::list_clients))
        .route("/api/v1/clients/search", get(client_api::search_clients))
        .route(
            "/api/v1/clients/filter_hardware",
            get(client_api::filter_clients_by_hardware),
        )
        .route("/api/v1/clients/export", get(client_api::export_clients))
        .route(
            "/api/v1/clients/export_filtered",
            post(client_api::export_filtered_clients),
        )
        .route("/api/v1/clients/{id}", get(client_api::get_client))
        .route("/api/v1/users", get(person_api::list_persons))
        .route("/api/v1/users/{id}", get(person_api::get_person))
        .route("/api/v1/projects", get(project_api::list_projects))
        .route("/api/v1/projects/{id}", get(project_api::get_project))
        .route("/api/v1/components", get(component_api::list_components))
        .route("/api/v1/components/{id}", get(component_api::get_component))
        .route(
            "/api/v1/dictionaries",
            get(dictionary_api::list_dictionaries),
        )
        .route(
            "/api/v1/dictionaries/{id}",
            get(dictionary_api::get_dictionary),
        )
        .route("/api/v1/racks", get(rack_api::list_racks))
        .route("/api/v1/racks/{id}", get(rack_api::get_rack))
        .route(
            "/api/v1/clients/{id}/hardware",
            get(hardware_api::get_hardware),
        )
        .route(
            "/api/v1/clients/{id}/hardware/history",
            get(hardware_api::get_hardware_history),
        )
        .route("/api/v1/stats/hardware", get(stats_api::get_hardware_stats))
        .route(
            "/api/v1/stats/clients",
            get(stats_api::get_clients_by_criteria),
        )
        .route("/api/v1/stats/detailed", get(stats_api::get_detailed_stats))
        .route(
            "/api/v1/stats/export",
            get(stats_api::export_client_hardware_data),
        )
        .route("/api/v1/clients/filter", post(stats_api::filter_clients))
        .route("/api/v1/filter_options", get(stats_api::get_filter_options))
        .route(
            "/api/v1/filter_options_by_ids",
            get(client_api::get_filter_options_by_client_ids),
        )
        // Approval: users can see their own requests
        .route("/api/v1/permissions/my-approvals", get(approval_api::list_my_approvals))
        // Session & replay: read-only routes (all authenticated)
        .route("/api/v1/sessions", get(cast_api::list_sessions))
        .route("/api/v1/sessions/{id}", get(cast_api::get_session))
        .route("/api/v1/sessions/{id}/cast", get(cast_api::get_cast_file))
        .route(
            "/api/v1/remote-exec/terminal-sessions",
            get(terminal_session_api::list_sessions).post(terminal_session_api::create_session),
        )
        .route("/api/v1/remote-exec/terminal-sessions/{id}", get(terminal_session_api::get_session).delete(terminal_session_api::close_session))
        .route("/api/v1/remote-exec/terminal-sessions/{id}/input", post(terminal_session_api::push_input))
        .route("/api/v1/remote-exec/terminal-sessions/{id}/resize", post(terminal_session_api::resize_session))
        .route("/api/v1/remote-exec/terminal-sessions/{id}/stream", get(terminal_session_api::stream_session))
        .route("/api/v1/remote-exec/terminal-sessions/{id}/ws", get(terminal_session_api::websocket_session))
        // Remote exec: read-only routes (all authenticated)
        .route("/api/v1/remote-exec/config", get(command_api::get_config))
        .route("/api/v1/remote-exec/commands", get(command_api::list_commands))
        .route("/api/v1/remote-exec/commands/{id}", get(command_api::get_command))
        .route("/api/v1/remote-exec/commands/{id}/logs", get(command_api::get_command_logs))
        .route("/api/v1/remote-exec/commands/{id}/stream", get(command_api::stream_command))
        .merge(change_password_route);

    // Protected routes
    let protected_routes = Router::new()
        .merge(admin_routes)
        .merge(register_route)
        .merge(user_write_routes)
        .merge(admin_write_routes)
        .merge(read_routes)
        .route_layer(middleware::from_fn(permission_middleware))
        .route_layer(middleware::from_fn(auth_middleware));

    // Main router
    Router::new()
        .merge(public_routes)
        .merge(agent_routes)
        .merge(protected_routes)
        // SPA fallback - 处理所有其他路由，包括静态资源
        .fallback_service(static_files_service.not_found_service(serve_index_html.into_service()))
        // Layer for cross-origin resource sharing
        .layer(cors)
        // Layer for dependency injection
        .layer(Extension(client_repo))
        .layer(Extension(hardware_repo))
        .layer(Extension(user_repo))
        .layer(Extension(person_repo))
        .layer(Extension(project_repo))
        .layer(Extension(component_repo))
        .layer(Extension(dictionary_repo))
        .layer(Extension(rack_repo))
        .layer(Extension(message_queue))
        .layer(Extension(client_service))
        .layer(Extension(auth_service))
        .layer(Extension(validation_service))
        .layer(Extension(stats_service))
        .layer(Extension(client_filter_service))
        .layer(Extension(export_service))
        .layer(Extension(config))
        .layer(Extension(command_service))
        .layer(Extension(sse_hub))
        .layer(Extension(perm_svc))
        .layer(Extension(permission_repo))
        .layer(Extension(exec_policy_repo))
        .layer(Extension(web_terminal_policy_repo))
        .layer(Extension(approval_repo))
        .layer(Extension(approval_svc))
        .layer(Extension(session_svc))
        .layer(Extension(terminal_svc))
}
