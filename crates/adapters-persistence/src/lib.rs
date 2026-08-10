#![forbid(unsafe_code)]
//! Postgres persistence adapter. Implements the application storage ports via
//! sqlx. Writes the `facts` schema only — never `app` (owned by the app plane).
//!
//! Queries are runtime-bound (`sqlx::query`, not the compile-checked macro) so
//! the crate builds with no live database — CI has none. JSONB columns are sent
//! as text with an explicit `::jsonb` cast.

use async_trait::async_trait;
use chrono::NaiveDate;
use lindex_application::ports::{
    RawScrutin, ReadModelStore, RepoError, ScrutinRepository, ScrutinView,
};
use lindex_domain::{
    Chamber, GroupId, GroupTally, Provenance, ProvenanceTier, Scrutin, ScrutinId, Sourced,
    VoteTotals,
};
use serde_json::{json, Value};
use sqlx::{PgPool, Row};

mod calendar;

pub struct PgStore {
    pub pool: PgPool,
}

impl PgStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

pub(crate) fn backend<E: std::fmt::Display>(e: E) -> RepoError {
    RepoError::Backend(e.to_string())
}

pub(crate) fn chamber_code(chamber: Chamber) -> &'static str {
    match chamber {
        Chamber::AssembleeNationale => "AN",
        Chamber::Senat => "SENAT",
    }
}

fn chamber_from_code(code: &str) -> Result<Chamber, RepoError> {
    match code {
        "AN" => Ok(Chamber::AssembleeNationale),
        "SENAT" => Ok(Chamber::Senat),
        other => Err(RepoError::Backend(format!(
            "unknown chamber code {other:?}"
        ))),
    }
}

fn tier_code(tier: ProvenanceTier) -> &'static str {
    match tier {
        ProvenanceTier::ActeAuthentique => "ActeAuthentique",
        ProvenanceTier::OrganismeIndependant => "OrganismeIndependant",
        ProvenanceTier::CommunicationGouv => "CommunicationGouv",
        ProvenanceTier::TravauxParlementaires => "TravauxParlementaires",
        ProvenanceTier::PresseSocieteCivile => "PresseSocieteCivile",
    }
}

/// `facts.scrutin.totals` JSON shape (round-tripped by `cohort_totals`/`by_id`).
fn totals_to_json(t: &VoteTotals) -> Value {
    json!({
        "pour": t.pour,
        "contre": t.contre,
        "abstention": t.abstention,
        "nonVotants": t.non_votants,
        "membersTotal": t.members_total,
    })
}

fn u32_field(v: &Value, key: &str) -> Result<u32, RepoError> {
    v.get(key)
        .and_then(Value::as_u64)
        .and_then(|n| u32::try_from(n).ok())
        .ok_or_else(|| RepoError::Backend(format!("totals JSON missing u32 field {key:?}")))
}

fn totals_from_json(v: &Value) -> Result<VoteTotals, RepoError> {
    Ok(VoteTotals {
        pour: u32_field(v, "pour")?,
        contre: u32_field(v, "contre")?,
        abstention: u32_field(v, "abstention")?,
        non_votants: u32_field(v, "nonVotants")?,
        members_total: u32_field(v, "membersTotal")?,
    })
}

fn breakdown_to_json(breakdown: &[GroupTally]) -> Value {
    Value::Array(
        breakdown
            .iter()
            .map(|g| {
                json!({
                    "group": g.group.0,
                    "pour": g.pour,
                    "contre": g.contre,
                    "abstention": g.abstention,
                    "nonVotant": g.non_votant,
                })
            })
            .collect(),
    )
}

fn breakdown_from_json(v: &Value) -> Result<Vec<GroupTally>, RepoError> {
    let arr = v
        .as_array()
        .ok_or_else(|| RepoError::Backend("breakdown JSON is not an array".into()))?;
    arr.iter()
        .map(|g| {
            let group = g
                .get("group")
                .and_then(Value::as_str)
                .ok_or_else(|| RepoError::Backend("breakdown entry missing group".into()))?;
            Ok(GroupTally {
                group: GroupId(group.to_string()),
                pour: u32_field(g, "pour")?,
                contre: u32_field(g, "contre")?,
                abstention: u32_field(g, "abstention")?,
                non_votant: u32_field(g, "nonVotant")?,
            })
        })
        .collect()
}

impl PgStore {
    async fn insert_source_record(
        &self,
        prov: &Provenance,
        payload: &str,
    ) -> Result<(), RepoError> {
        sqlx::query(
            "insert into facts.source_record (id, tier, url, fetched_at, payload) \
             values ($1, $2, $3, $4, $5::jsonb) \
             on conflict (id) do update set \
               tier = excluded.tier, url = excluded.url, \
               fetched_at = excluded.fetched_at, payload = excluded.payload",
        )
        .bind(&prov.source.record_id)
        .bind(tier_code(prov.source.tier))
        .bind(&prov.source.url)
        .bind(prov.retrieved_at)
        .bind(payload)
        .execute(&self.pool)
        .await
        .map_err(backend)?;
        Ok(())
    }
}

#[async_trait]
impl ScrutinRepository for PgStore {
    async fn put_source_record(&self, raw: &Sourced<RawScrutin>) -> Result<(), RepoError> {
        self.insert_source_record(raw.provenance(), &raw.value().payload)
            .await
    }

    async fn upsert(&self, scrutin: &Sourced<Scrutin>) -> Result<(), RepoError> {
        let s = scrutin.value();
        let totals = totals_to_json(&s.totals).to_string();
        let breakdown = breakdown_to_json(&s.breakdown).to_string();
        sqlx::query(
            "insert into facts.scrutin \
               (id, chamber, title, held_on, totals, breakdown, source_record, updated_at) \
             values ($1, $2, $3, $4, $5::jsonb, $6::jsonb, $7, now()) \
             on conflict (id) do update set \
               chamber = excluded.chamber, title = excluded.title, \
               held_on = excluded.held_on, totals = excluded.totals, \
               breakdown = excluded.breakdown, source_record = excluded.source_record, \
               updated_at = now()",
        )
        .bind(&s.id.0)
        .bind(chamber_code(s.chamber))
        .bind(&s.title)
        .bind(s.held_on)
        .bind(totals)
        .bind(breakdown)
        .bind(&scrutin.provenance().source.record_id)
        .execute(&self.pool)
        .await
        .map_err(backend)?;
        Ok(())
    }

    async fn by_id(&self, id: &ScrutinId) -> Result<Option<Scrutin>, RepoError> {
        let row = sqlx::query(
            "select chamber, title, held_on, totals, breakdown \
             from facts.scrutin where id = $1",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(backend)?;
        let Some(row) = row else { return Ok(None) };

        let chamber = chamber_from_code(
            row.try_get::<String, _>("chamber")
                .map_err(backend)?
                .as_str(),
        )?;
        let title: String = row.try_get("title").map_err(backend)?;
        let held_on: NaiveDate = row.try_get("held_on").map_err(backend)?;
        let totals: Value = row.try_get("totals").map_err(backend)?;
        let breakdown: Value = row.try_get("breakdown").map_err(backend)?;

        let scrutin = Scrutin::try_new(
            id.clone(),
            chamber,
            title,
            held_on,
            totals_from_json(&totals)?,
            breakdown_from_json(&breakdown)?,
        )
        .map_err(backend)?;
        Ok(Some(scrutin))
    }

    async fn cohort_totals(
        &self,
        chamber: &Chamber,
        held_on: NaiveDate,
    ) -> Result<Vec<VoteTotals>, RepoError> {
        let rows =
            sqlx::query("select totals from facts.scrutin where chamber = $1 and held_on = $2")
                .bind(chamber_code(*chamber))
                .bind(held_on)
                .fetch_all(&self.pool)
                .await
                .map_err(backend)?;
        rows.iter()
            .map(|row| {
                let totals: Value = row.try_get("totals").map_err(backend)?;
                totals_from_json(&totals)
            })
            .collect()
    }
}

#[async_trait]
impl ReadModelStore for PgStore {
    async fn put_scrutin_view(&self, view: &ScrutinView) -> Result<(), RepoError> {
        let held_on = NaiveDate::parse_from_str(&view.held_on, "%Y-%m-%d")
            .map_err(|e| RepoError::Backend(format!("bad held_on {:?}: {e}", view.held_on)))?;
        sqlx::query(
            "insert into facts.read_scrutin \
               (scrutin_id, chamber, title, held_on, outcome, totals, breakdown, \
                baselines, provenance, updated_at) \
             values ($1, $2, $3, $4, $5, $6::jsonb, $7::jsonb, $8::jsonb, $9::jsonb, now()) \
             on conflict (scrutin_id) do update set \
               chamber = excluded.chamber, title = excluded.title, \
               held_on = excluded.held_on, outcome = excluded.outcome, \
               totals = excluded.totals, breakdown = excluded.breakdown, \
               baselines = excluded.baselines, provenance = excluded.provenance, \
               updated_at = now()",
        )
        .bind(&view.scrutin_id)
        .bind(&view.chamber)
        .bind(&view.title)
        .bind(held_on)
        .bind(&view.outcome)
        .bind(&view.totals_json)
        .bind(&view.breakdown_json)
        .bind(&view.baselines_json)
        .bind(&view.provenance_json)
        .execute(&self.pool)
        .await
        .map_err(backend)?;
        Ok(())
    }

    async fn refresh_day_baselines(
        &self,
        chamber: &str,
        held_on: NaiveDate,
        baselines_json: &str,
    ) -> Result<(), RepoError> {
        sqlx::query(
            "update facts.read_scrutin set baselines = $3::jsonb, updated_at = now() \
             where chamber = $1 and held_on = $2",
        )
        .bind(chamber)
        .bind(held_on)
        .bind(baselines_json)
        .execute(&self.pool)
        .await
        .map_err(backend)?;
        Ok(())
    }
}
