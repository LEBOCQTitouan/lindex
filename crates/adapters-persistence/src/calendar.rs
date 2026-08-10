//! [`ScrutinCalendar`] adapter: the work-set lookup a date-range re-ingest uses
//! to learn which scrutins are on record for a chamber over a day-range (US-8.4).

use crate::{backend, chamber_code, PgStore};
use async_trait::async_trait;
use chrono::NaiveDate;
use lindex_application::ops::ScrutinCalendar;
use lindex_application::ports::RepoError;
use lindex_domain::{Chamber, ScrutinId};
use sqlx::Row;

#[async_trait]
impl ScrutinCalendar for PgStore {
    async fn ids_in_range(
        &self,
        chamber: Chamber,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<ScrutinId>, RepoError> {
        let rows = sqlx::query(
            "select id from facts.scrutin \
             where chamber = $1 and held_on between $2 and $3 \
             order by id",
        )
        .bind(chamber_code(chamber))
        .bind(from)
        .bind(to)
        .fetch_all(&self.pool)
        .await
        .map_err(backend)?;
        rows.iter()
            .map(|row| Ok(ScrutinId(row.try_get::<String, _>("id").map_err(backend)?)))
            .collect()
    }
}
