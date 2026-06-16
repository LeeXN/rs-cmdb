pub mod auth_api;
mod client_api;
pub mod component_api;
pub mod dictionary_api;
mod download_api;
mod hardware_api;
mod health_api;
pub mod person_api;
pub mod project_api;
pub mod rack_api;
pub mod stats_api;
pub mod user_api;

use std::sync::Arc;

use crate::config::ServerConfig;
use crate::middleware::auth::auth_middleware;
use crate::middleware::rbac;
use crate::queue::MessageQueue;
use crate::repository::{
    client_repository::ClientRepository, component_repository::ComponentRepository,
    dictionary_repository::DictionaryRepository, hardware_repository::HardwareRepository,
    person_repository::PersonRepository, project_repository::ProjectRepository,
    rack_repository::RackRepository, user_repository::UserRepository,
};
use crate::service::{
    auth_service::AuthService, client_filter_service::ClientFilterService,
    client_service::ClientService, export_service::ExportService, stats_service::StatsService,
    validation_service::ValidationService,
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
use tower_http::cors::{Any, CorsLayer};
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
) -> Router {
    // Create the CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 创建静态文件服务
    let static_files_service = ServeDir::new("dist");

    // Public routes
    let public_routes = Router::new()
        .route("/api/v1/health", get(health_api::health_check))
        .route("/api/v1/version", get(health_api::version))
        .route("/api/v1/auth/login", post(auth_api::login))
        // Download routes (usually public for agents)
        .route("/api/v1/download/info", get(download_api::get_client_info))
        .route(
            "/api/v1/download/client/{platform}/{arch}/{binary_name}",
            get(download_api::download_client),
        )
        .route("/install.sh", get(download_api::get_install_script))
        // Client registration (agent)
        .route(
            "/api/v1/clients/register",
            post(client_api::register_client),
        )
        // Hardware push (agent)
        .route(
            "/api/v1/clients/{id}/hardware",
            post(hardware_api::update_hardware),
        );

    // Admin routes
    let admin_routes = Router::new()
        .route("/api/v1/accounts", get(user_api::list_users))
        .route(
            "/api/v1/accounts/{id}",
            delete(user_api::delete_user).put(user_api::update_user),
        )
        .route("/api/v1/auth/register", post(auth_api::register))
        .route_layer(middleware::from_fn(rbac::require_admin));

    // Write routes (User & Admin)
    let write_routes = Router::new()
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

    // Read routes (All authenticated)
    let read_routes = Router::new()
        .route("/api/v1/auth/me", get(auth_api::me))
        .route(
            "/api/v1/auth/change-password",
            post(auth_api::change_password),
        )
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
        );

    // Protected routes
    let protected_routes = Router::new()
        .merge(admin_routes)
        .merge(write_routes)
        .merge(read_routes)
        .route_layer(middleware::from_fn(auth_middleware));

    // Main router
    Router::new()
        .merge(public_routes)
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
}
