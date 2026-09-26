use axum::{Json, Router, extract::Path, routing::get};
use serde::Serialize;
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, ToSchema)]
struct User {
    id: u64,
    name: String,
}

#[utoipa::path(
    get,
    path = "/api/users/{id}",
    params(
        ("id" = u64, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User found", body = User)
    )
)]
async fn get_user(Path(id): Path<u64>) -> Json<User> {
    Json(User {
        id,
        name: "Karel".to_string(),
    })
}

#[derive(OpenApi)]
#[openapi(paths(get_user), components(schemas(User)))]
struct ApiDoc;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/api/users/{id}", get(get_user))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
        .await
        .unwrap();

    println!("API running on http://localhost:8000");

    axum::serve(listener, app).await.unwrap();
}
