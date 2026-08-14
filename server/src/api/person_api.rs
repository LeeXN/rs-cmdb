use crate::middleware::permission::PermissionContext;
use crate::repository::client_repository::ClientRepository;
use crate::repository::person_repository::PersonRepository;
use crate::repository::project_repository::ProjectRepository;
use axum::{
    extract::{Extension, Json, Path, Query},
    http::StatusCode,
    response::IntoResponse,
};
use chrono::Utc;
use common::entity::permission::{PermissionAction, ResourceType, ScopeConstraint};
use common::entity::user::User;
use common::models::{ApiResponse, PaginatedResult, Person, PersonQuery};
use std::sync::Arc;
use uuid::Uuid;

/// List all persons
pub async fn list_persons(
    Query(query): Query<PersonQuery>,
    Extension(person_repo): Extension<Arc<PersonRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match person_repo.list_all().await {
        Ok(mut persons) => {
            let scope = perm_ctx
                .evaluate(&ResourceType::Person, &PermissionAction::View)
                .unwrap_or(ScopeConstraint::None);
            persons = PermissionContext::filter_by_scope(persons, &scope, &perm_ctx.user_id, |p| {
                p.created_by.as_deref()
            });
            // Filter by search term
            if let Some(ref search) = query.search {
                let search_lower = search.to_lowercase();
                persons.retain(|p| {
                    p.name.to_lowercase().contains(&search_lower)
                        || p.email.to_lowercase().contains(&search_lower)
                        || p.department
                            .as_ref()
                            .is_some_and(|d| d.to_lowercase().contains(&search_lower))
                });
            }

            // Filter by department
            if let Some(ref department) = query.department
                && !department.is_empty()
            {
                persons.retain(|p| p.department.as_ref() == Some(department));
            }

            // Sort by name
            persons.sort_by(|a, b| a.name.cmp(&b.name));

            // Pagination
            let total = persons.len();
            let page = query.page.unwrap_or(1);
            let page_size = query.page_size.unwrap_or(10);
            let total_pages = (total as f64 / page_size as f64).ceil() as usize;

            let start = (page - 1) * page_size;
            let end = std::cmp::min(start + page_size, total);

            let items = if start < total {
                persons[start..end].to_vec()
            } else {
                Vec::new()
            };

            let result = PaginatedResult {
                items,
                total,
                page,
                page_size,
                total_pages,
            };

            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(result),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<PaginatedResult<Person>> {
                status: 500,
                message: format!("Failed to list persons: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Get a person by ID
pub async fn get_person(
    Path(id): Path<String>,
    Extension(person_repo): Extension<Arc<PersonRepository>>,
    Extension(perm_ctx): Extension<PermissionContext>,
) -> impl IntoResponse {
    match person_repo.get(&id).await {
        Ok(Some(person)) => {
            if !perm_ctx.allows_resource(
                &ResourceType::Person,
                &PermissionAction::View,
                person.created_by.as_deref(),
            ) {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<Person> {
                        status: 403,
                        message: "Forbidden".into(),
                        data: None,
                    }),
                )
                    .into_response();
            }
            let response = ApiResponse {
                status: 200,
                message: "Success".to_string(),
                data: Some(person),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Ok(None) => {
            let response = ApiResponse::<Person> {
                status: 404,
                message: "Person not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Person> {
                status: 500,
                message: format!("Failed to get person: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Create a new person
pub async fn create_person(
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(user): Extension<User>,
    Extension(person_repo): Extension<Arc<PersonRepository>>,
    Json(mut person): Json<Person>,
) -> impl IntoResponse {
    if !perm_ctx.allows_action(&ResourceType::Person, &PermissionAction::Create) {
        let response = ApiResponse::<Person> {
            status: 403,
            message: "Forbidden: insufficient permission to create persons".to_string(),
            data: None,
        };
        return (StatusCode::FORBIDDEN, Json(response)).into_response();
    }

    // Ensure ID is set
    if person.id.is_empty() {
        person.id = Uuid::new_v4().to_string();
    }

    // Set ownership and timestamps
    let now = Utc::now().to_rfc3339();
    person.created_by = Some(user.id);
    person.created_at = now.clone();
    person.updated_at = now;

    match person_repo.save(&person).await {
        Ok(_) => {
            let response = ApiResponse {
                status: 201,
                message: "Person created successfully".to_string(),
                data: Some(person),
            };
            (StatusCode::CREATED, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Person> {
                status: 500,
                message: format!("Failed to create person: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// Update a person
pub async fn update_person(
    Path(id): Path<String>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(person_repo): Extension<Arc<PersonRepository>>,
    Json(mut person): Json<Person>,
) -> impl IntoResponse {
    // Check if exists
    match person_repo.exists(&id).await {
        Ok(true) => {
            if !person_repo.exists(&id).await.unwrap_or(false) {
                let response = ApiResponse::<Person> {
                    status: 404,
                    message: "Person not found".to_string(),
                    data: None,
                };
                return (StatusCode::NOT_FOUND, Json(response)).into_response();
            }

            match person_repo.get(&id).await {
                Ok(Some(existing_person)) => {
                    if !perm_ctx.allows_resource(
                        &ResourceType::Person,
                        &PermissionAction::Update,
                        existing_person.created_by.as_deref(),
                    ) {
                        let response = ApiResponse::<Person> {
                            status: 403,
                            message: "Forbidden: insufficient permission to update this person"
                                .to_string(),
                            data: None,
                        };
                        return (StatusCode::FORBIDDEN, Json(response)).into_response();
                    }

                    person.id = id;
                    person.created_by = existing_person.created_by;
                    person.created_at = existing_person.created_at;
                    person.updated_at = Utc::now().to_rfc3339();

                    match person_repo.save(&person).await {
                        Ok(_) => {
                            let response = ApiResponse {
                                status: 200,
                                message: "Person updated successfully".to_string(),
                                data: Some(person),
                            };
                            (StatusCode::OK, Json(response)).into_response()
                        }
                        Err(e) => {
                            let response = ApiResponse::<Person> {
                                status: 500,
                                message: format!("Failed to update person: {}", e),
                                data: None,
                            };
                            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
                        }
                    }
                }
                _ => {
                    let response = ApiResponse::<Person> {
                        status: 404,
                        message: "Person not found".to_string(),
                        data: None,
                    };
                    (StatusCode::NOT_FOUND, Json(response)).into_response()
                }
            }
        }
        Ok(false) => {
            let response = ApiResponse::<Person> {
                status: 404,
                message: "Person not found".to_string(),
                data: None,
            };
            (StatusCode::NOT_FOUND, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<Person> {
                status: 500,
                message: format!("Failed to check person existence: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::fixtures::{TestAppBuilder, auth_headers};
    use axum::{
        body::Body,
        extract::Request,
        http::{Method, StatusCode, header},
    };
    use serde_json::json;
    use tower::ServiceExt;

    async fn make_post(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_get(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::GET).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_put(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
        body: serde_json::Value,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder()
            .method(Method::PUT)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req
            .body(Body::from(serde_json::to_vec(&body).unwrap()))
            .unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    async fn make_delete(
        app: &axum::Router,
        path: &str,
        token: Option<&str>,
    ) -> (StatusCode, serde_json::Value) {
        let mut req = Request::builder().method(Method::DELETE).uri(path);
        if let Some(t) = token {
            let (k, v) = auth_headers(t);
            req = req.header(k, v);
        }
        let req = req.body(Body::empty()).unwrap();
        let resp = app.clone().oneshot(req).await.unwrap();
        let status = resp.status();
        let body: serde_json::Value = serde_json::from_slice(
            &axum::body::to_bytes(resp.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn test_create_person() {
        let app = TestAppBuilder::new().build().await;
        let (status, body) = make_post(
            &app.router,
            "/api/v1/users",
            Some(&app.admin_token),
            json!({
                "name": "John Doe",
                "email": "john@example.com",
                "department": "IT"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::CREATED, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "John Doe");
        assert_eq!(body["data"]["email"], "john@example.com");
    }

    #[tokio::test]
    async fn test_list_persons() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, _) = make_post(
            &app.router,
            "/api/v1/users",
            Some(&app.admin_token),
            json!({
                "name": "List Test",
                "email": "list@test.com"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);

        let (status, body) = make_get(&app.router, "/api/v1/users", Some(&app.admin_token)).await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert!(body["data"]["total"].as_u64().unwrap_or(0) >= 1);
    }

    #[tokio::test]
    async fn test_get_person() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/users",
            Some(&app.admin_token),
            json!({
                "name": "Get Test",
                "email": "get@test.com"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

        let (status, body) = make_get(
            &app.router,
            &format!("/api/v1/users/{}", person_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "Get Test");
    }

    #[tokio::test]
    async fn test_update_person() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/users",
            Some(&app.admin_token),
            json!({
                "name": "Original Name",
                "email": "original@test.com"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

        let (status, body) = make_put(
            &app.router,
            &format!("/api/v1/users/{}", person_id),
            Some(&app.admin_token),
            json!({
                "name": "Jane Doe",
                "email": "jane@example.com"
            }),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "body: {:?}", body);
        assert_eq!(body["data"]["name"], "Jane Doe");
        assert_eq!(body["data"]["email"], "jane@example.com");
    }

    #[tokio::test]
    async fn test_delete_person() {
        let app = TestAppBuilder::new().build().await;
        let (create_status, create_body) = make_post(
            &app.router,
            "/api/v1/users",
            Some(&app.admin_token),
            json!({
                "name": "Delete Test",
                "email": "delete@test.com"
            }),
        )
        .await;
        assert_eq!(create_status, StatusCode::CREATED);
        let person_id = create_body["data"]["id"].as_str().unwrap().to_string();

        let (status, _) = make_delete(
            &app.router,
            &format!("/api/v1/users/{}", person_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "expected 200 OK on delete");

        let (get_status, _) = make_get(
            &app.router,
            &format!("/api/v1/users/{}", person_id),
            Some(&app.admin_token),
        )
        .await;
        assert_eq!(get_status, StatusCode::NOT_FOUND);
    }
}

/// Delete a person
pub async fn delete_person(
    Path(id): Path<String>,
    Extension(perm_ctx): Extension<PermissionContext>,
    Extension(person_repo): Extension<Arc<PersonRepository>>,
    Extension(client_repo): Extension<Arc<ClientRepository>>,
    Extension(project_repo): Extension<Arc<ProjectRepository>>,
) -> impl IntoResponse {
    // Ownership check
    if let Ok(Some(existing)) = person_repo.get(&id).await
        && !perm_ctx.allows_resource(
            &ResourceType::Person,
            &PermissionAction::Delete,
            existing.created_by.as_deref(),
        )
    {
        let response = ApiResponse::<()> {
            status: 403,
            message: "Forbidden: insufficient permission to delete this person".to_string(),
            data: None,
        };
        return (StatusCode::FORBIDDEN, Json(response)).into_response();
    }

    // Cascade update: Set owner_id to null for clients
    if let Err(e) = client_repo.update_owner_to_null(&id).await {
        let response = ApiResponse::<()> {
            status: 500,
            message: format!("Failed to update client owners: {}", e),
            data: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
    }

    // Cascade update: Set manager_id to null for projects
    if let Err(e) = project_repo.update_manager_to_null(&id).await {
        let response = ApiResponse::<()> {
            status: 500,
            message: format!("Failed to update project managers: {}", e),
            data: None,
        };
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response();
    }

    match person_repo.delete(&id).await {
        Ok(_) => {
            let response = ApiResponse::<()> {
                status: 200,
                message: "Person deleted successfully".to_string(),
                data: None,
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            let response = ApiResponse::<()> {
                status: 500,
                message: format!("Failed to delete person: {}", e),
                data: None,
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}
