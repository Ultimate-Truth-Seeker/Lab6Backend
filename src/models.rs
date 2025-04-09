use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Series {
    pub id: i64,
    pub title: String,
    pub status: String,
    pub lastEpisodeWatched: i32,
    pub totalEpisodes: i32,
    pub ranking: i32,
}

// Este struct se usará para crear nuevas series (POST). 
// No incluye `id` porque la base de datos lo genera automáticamente.
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateSeries {
    pub title: String,
    pub status: String,
    pub lastEpisodeWatched: i32,
    pub totalEpisodes: i32,
    pub ranking: i32,
}

// Este struct se usaría en PUT / PATCH cuando queremos actualizar valores.
// En este ejemplo, se hace PUT con SeriesEntero o a veces se hacen PATCHs muy concretos.
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSeries {
    pub title: Option<String>,
    pub status: Option<String>,
    pub lastEpisodeWatched: Option<i32>,
    pub totalEpisodes: Option<i32>,
    pub ranking: Option<i32>,
}