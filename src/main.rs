use axum::{Json, Router, extract::State, http::StatusCode, routing::get};
use mongodb::{Client, bson::doc};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

#[derive(Serialize, ToSchema)]
struct Health {
    status: String,
}

#[utoipa::path(get, path = "/api/v1/health", responses((status = 200, description = "The app is running fine" , body = Health)))]
async fn get_health() -> Json<Health> {
    Json(Health {
        status: "ok".to_owned(),
    })
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
struct Team {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    #[schema(value_type = String)]
    id: Option<mongodb::bson::oid::ObjectId>,

    name: String,
    members: Vec<String>,
}

async fn create_team(state: &AppState) -> mongodb::error::Result<()> {
    let db = state.mongo.database("tour_de_app");
    let teams = db.collection::<Team>("teams");

    let team = Team {
        id: None,
        name: "V.I.B.E.".to_string(),
        members: vec![
            "Karel Lukeš".to_string(),
            "Kryštof Jurda".to_string(),
            "Jindřich Kraina".to_string(),
        ],
    };

    teams.insert_one(team).await?;

    Ok(())
}

#[utoipa::path(get, path = "/api/v1/team", responses((status = 200, description = "Team found" , body = Team), (status = 404, description = "Team not found")))]
async fn get_team(State(state): State<AppState>) -> Result<Json<Team>, (StatusCode, String)> {
    let db = state.mongo.database("tour_de_app");
    let teams = db.collection::<Team>("teams");

    let team = teams
        .find_one(doc! { "name": "V.I.B.E." })
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match team {
        Some(team) => Ok(Json(team)),
        None => Err((StatusCode::NOT_FOUND, "Could not find team".to_string())),
    }
}

#[derive(Clone)]
pub struct AppState {
    pub mongo: Client,
}

#[derive(OpenApi)]
#[openapi(paths(get_health, get_team), components(schemas(Health, Team)))]
struct ApiDoc;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    let mongo_uri = std::env::var("MONGODB_URI").unwrap_or_else(|_| {
        "mongodb://root:procMIkradesHESLOzmrde@localhost:6767/?authSource=admin".to_owned()
    });
    let mongo = Client::with_uri_str(mongo_uri).await?;

    let state = AppState { mongo };
    let initialization_state = state.clone();

    tokio::spawn(async move {
        for attempt in 1..=30 {
            match initialization_state
                .mongo
                .database("admin")
                .run_command(doc! { "ping": 1 })
                .await
            {
                Ok(_) => {
                    if let Err(error) = create_team(&initialization_state).await {
                        eprintln!("Could not initialize team data: {error}");
                    }
                    return;
                }
                Err(error) if attempt == 30 => {
                    eprintln!("MongoDB was not ready after 60 seconds: {error}");
                }
                Err(_) => tokio::time::sleep(std::time::Duration::from_secs(2)).await,
            }
        }
    });

    let app = Router::new()
        .route("/api/v1/health", get(get_health))
        .route("/api/v1/team", get(get_team))
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    println!("API running on http://localhost:8000");

    axum::serve(listener, app).await.unwrap();

    Ok(())
}
