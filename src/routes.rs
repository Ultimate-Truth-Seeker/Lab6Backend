use actix_web::{web, HttpResponse, Responder};
use sqlx::{SqlitePool, sqlite::SqliteArguments, Arguments};
use serde_json::json;

use crate::models::{CreateSeries, Series, UpdateSeries};

use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct SeriesQueryParams {
    // Parámetro para buscar en el título (usa LIKE)
    pub search: Option<String>,
    // Parámetro para filtrar por estado (ej. "Watching", "Completed", etc.)
    pub status: Option<String>,
    // Parámetro para el orden del ranking ("asc" o "desc")
    pub sort: Option<String>,
}

/// GET /api/series - Obtener todas las series
// Aquí hice la modificación para que se puedan aplicar los filtros en el frontend
pub async fn get_all_series(
    pool: web::Data<SqlitePool>,
    query: Option<web::Query<SeriesQueryParams>>,
) -> impl Responder {
    // Extraemos los parámetros
    let query = query.map(|q| q.into_inner()).unwrap_or_default();
    let mut sql = String::from("SELECT * FROM series");
    let mut conditions = Vec::new();
    let mut args = sqlx::sqlite::SqliteArguments::default();

    // Agregar filtro para el título, si se especifica y no está vacío.
    if let Some(search) = query.search {
        if !search.trim().is_empty() {
            conditions.push("title LIKE ?");
            // Agregamos % para que la búsqueda sea parcial
            args.add(format!("%{}%", search));
        }
    }
    
    // Filtrar por el estado, si se recibe un valor.
    if let Some(status) = query.status {
        if !status.trim().is_empty() {
            conditions.push("status = ?");
            args.add(status);
        }
    }
    
    // Si se aplicó al menos un filtro, agregamos la cláusula WHERE
    if !conditions.is_empty() {
        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
    }

    // Ordenamiento por ranking
    if let Some(sort) = query.sort {
        if sort.to_lowercase() == "asc" {
            sql.push_str(" ORDER BY ranking ASC");
        } else if sort.to_lowercase() == "desc" {
            sql.push_str(" ORDER BY ranking DESC");
        }
    }

    // Ejecutamos la consulta con los argumentos dinámicos
    let result = sqlx::query_as_with::<_, Series, _>(&sql, args)
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(series_list) => HttpResponse::Ok().json(series_list),
        Err(e) => {
            eprintln!("Error al obtener series: {:?}", e);
            HttpResponse::InternalServerError().body("Error al obtener series")
        }
    }
}

/// GET /api/series/{id} - Obtener una serie por ID
pub async fn get_series_by_id(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();
    let result = sqlx::query_as::<_, Series>("SELECT * FROM series WHERE id = ?")
        .bind(id)
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(series) => HttpResponse::Ok().json(series),
        Err(sqlx::Error::RowNotFound) => HttpResponse::NotFound().body("Serie no encontrada"),
        Err(e) => {
            eprintln!("Error al obtener serie: {:?}", e);
            HttpResponse::InternalServerError().body("Error al obtener la serie")
        }
    }
}

/// POST /api/series - Crear una nueva serie
pub async fn create_series(
    pool: web::Data<SqlitePool>,
    new_series: web::Json<CreateSeries>,
) -> impl Responder {
    let result = sqlx::query(
        "INSERT INTO series (title, status, lastEpisodeWatched, totalEpisodes, ranking)
        VALUES (?1, ?2, ?3, ?4, ?5)",
    )
    .bind(&new_series.title)
    .bind(&new_series.status)
    .bind(new_series.lastEpisodeWatched)
    .bind(new_series.totalEpisodes)
    .bind(new_series.ranking)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(query_result) => {
            // query_result.last_insert_rowid() nos da el ID autogenerado (en SQLite)
            let created_id = query_result.last_insert_rowid();
            HttpResponse::Created().json(created_id)
        }
        Err(e) => {
            eprintln!("Error al crear serie: {:?}", e);
            HttpResponse::InternalServerError().body("Error al crear la serie")
        }
    }
}

/// PUT /api/series/{id} - Actualizar una serie existente (reemplazo total)
pub async fn update_series(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    updated_data: web::Json<CreateSeries>,
) -> impl Responder {
    let id = path.into_inner();

    // Realizamos un UPDATE con todos los campos. Asumimos un reemplazo "completo".
    let result = sqlx::query(
        "UPDATE series
        SET title = ?1, status = ?2, lastEpisodeWatched = ?3, totalEpisodes = ?4, ranking = ?5
        WHERE id = ?6",
    )
    .bind(&updated_data.title)
    .bind(&updated_data.status)
    .bind(updated_data.lastEpisodeWatched)
    .bind(updated_data.totalEpisodes)
    .bind(updated_data.ranking)
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Serie actualizada correctamente"
        })),
        Err(e) => {
            eprintln!("Error al actualizar serie: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Error al actualizar la serie"
            }))
        }
    }
}

/// DELETE /api/series/{id} - Eliminar una serie
pub async fn delete_series(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query("DELETE FROM series WHERE id = ?")
        .bind(id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Serie eliminada correctamente"),
        Err(e) => {
            eprintln!("Error al eliminar serie: {:?}", e);
            HttpResponse::InternalServerError().body("Error al eliminar la serie")
        }
    }
}

/// PATCH /api/series/{id}/status - actualizar el estado de una serie
pub async fn patch_series_status(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
    data: web::Json<UpdateSeries>,
) -> impl Responder {
    let id = path.into_inner();

    if let Some(new_status) = &data.status {
        let result = sqlx::query("UPDATE series SET status = ?1 WHERE id = ?2")
            .bind(new_status)
            .bind(id)
            .execute(pool.get_ref())
            .await;

        match result {
            Ok(_) => HttpResponse::Ok().json(json!({
                "success": true,
                "message": "Estado actualizado"
            })),
            Err(e) => {
                eprintln!("Error al actualizar estado: {:?}", e);
                HttpResponse::InternalServerError().json(json!({
                    "success": false,
                    "message": "Error al actualizar estado"
                }))
            }
        }
    } else {
        HttpResponse::BadRequest().body("Falta el campo 'status'")
    }
}

/// PATCH /api/series/{id}/episode - incrementar el episodio actual
pub async fn patch_series_episode(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query(
        "UPDATE series 
         SET lastEpisodeWatched = lastEpisodeWatched + 1
         WHERE id = ?1"
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Episodio incrementado"
        })),
        Err(e) => {
            eprintln!("Error al incrementar episodio: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Error al incrementar episodio"
            }))
        }
    }
}

/// PATCH /api/series/{id}/upvote - para aumentar la puntuación (ranking)
pub async fn patch_series_upvote(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query(
        "UPDATE series
         SET ranking = ranking + 1
         WHERE id = ?1"
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Ranking incrementado"
        })),
        Err(e) => {
            eprintln!("Error al incrementar ranking: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Error al incrementar ranking"
            }))
        }
    }
}

/// PATCH /api/series/{id}/downvote - para disminuir la puntuación (ranking)
pub async fn patch_series_downvote(
    pool: web::Data<SqlitePool>,
    path: web::Path<i64>,
) -> impl Responder {
    let id = path.into_inner();

    let result = sqlx::query(
        "UPDATE series
         SET ranking = ranking - 1
         WHERE id = ?1"
    )
    .bind(id)
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(json!({
            "success": true,
            "message": "Ranking decremenado"
        })),
        Err(e) => {
            eprintln!("Error al decrementar ranking: {:?}", e);
            HttpResponse::InternalServerError().json(json!({
                "success": false,
                "message": "Error al decrementar ranking"
            }))
        }
    }
}