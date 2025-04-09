use actix_web::{web, App, HttpServer};
use actix_cors::Cors;
use sqlx::SqlitePool;
use std::env;

mod models;
mod routes;

use routes::{
    create_series, delete_series, get_all_series, get_series_by_id, patch_series_downvote,
    patch_series_episode, patch_series_status, patch_series_upvote, update_series,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Iniciando app...");

    // Carga variables de entorno (por si usas .env para tu DB_URL, etc.)
    dotenv::dotenv().ok();

    // Definimos la cadena de conexión. Por ejemplo: "sqlite://app.db"
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://data.db".to_string());
    // link del frontend de series-tracker (Configuración CORS)
    let frontend_url = env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost".to_string());

    // Creamos el pool de conexión
    let pool = SqlitePool::connect(&database_url)
        .await
        .expect("No se pudo conectar a la base de datos SQLite");

    // IMPORTANTE: Creación de la tabla si no existe.
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS series (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            title TEXT NOT NULL,
            status TEXT NOT NULL,
            lastEpisodeWatched INTEGER NOT NULL,
            totalEpisodes INTEGER NOT NULL,
            ranking INTEGER NOT NULL
        );
        "#,
    )
    .execute(&pool)
    .await
    .expect("Fallo al crear la tabla series");

    // Arrancamos el servidor HTTP
    HttpServer::new(move || {
        App::new()
            .wrap( // ### Configuración y manejo de solicitudes CORS ###
                Cors::default()
                    // Permite solicitudes del frontend
                    .allowed_origin(&frontend_url).allowed_origin("https://hoppscotch.io")
                    // Métodos permitidos
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "PATCH"])
                    // Encabezados permitidos (por ejemplo, Content-Type)
                    .allowed_headers(vec![actix_web::http::header::CONTENT_TYPE])
                    .max_age(3600)
            )
            // Clonamos pool para accederlo en cada handler
            .app_data(web::Data::new(pool.clone()))
            // Rutas
            .route("/api/series", web::get().to(get_all_series))
            .route("/api/series/{id}", web::get().to(get_series_by_id))
            .route("/api/series", web::post().to(create_series))
            .route("/api/series/{id}", web::put().to(update_series))
            .route("/api/series/{id}", web::delete().to(delete_series))
            .route("/api/series/{id}/status", web::patch().to(patch_series_status))
            .route("/api/series/{id}/episode", web::patch().to(patch_series_episode))
            .route("/api/series/{id}/upvote", web::patch().to(patch_series_upvote))
            .route("/api/series/{id}/downvote", web::patch().to(patch_series_downvote))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}