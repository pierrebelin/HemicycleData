use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::application::ports::dossier_repository::{DossierRepository, RepositoryError};
use crate::domain::dossier::{Initiator, LegislativeAct, LegislativeStage, LegislativeDossier, Score};

pub struct PgDossierRepository {
    pool: PgPool,
}

impl PgDossierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn fetch_acts(&self, dossier_uid: &str) -> Result<Vec<LegislativeAct>, RepositoryError> {
        let rows = sqlx::query_as::<_, ActRow>(
            "SELECT act_date, label FROM legislative_acts WHERE dossier_uid = $1 ORDER BY act_date",
        )
        .bind(dossier_uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| LegislativeAct {
                date: r.act_date,
                label: r.label,
            })
            .collect())
    }

    async fn save_batch(&self, dossiers: &[LegislativeDossier]) -> Result<(), RepositoryError> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        for dossier in dossiers {
            let stage_code = dossier.current_stage.map(|s| s.to_code().to_string());

            sqlx::query(
                "INSERT INTO legislative_dossiers (uid, title, procedure_label, last_activity_date, last_activity_label, score_progress, score_magnitude, score_momentum, score_total, current_stage_code, committee)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                 ON CONFLICT (uid) DO UPDATE SET
                    title = EXCLUDED.title,
                    procedure_label = EXCLUDED.procedure_label,
                    last_activity_date = EXCLUDED.last_activity_date,
                    last_activity_label = EXCLUDED.last_activity_label,
                    score_progress = EXCLUDED.score_progress,
                    score_magnitude = EXCLUDED.score_magnitude,
                    score_momentum = EXCLUDED.score_momentum,
                    score_total = EXCLUDED.score_total,
                    current_stage_code = EXCLUDED.current_stage_code,
                    committee = EXCLUDED.committee,
                    updated_at = NOW()",
            )
            .bind(&dossier.uid)
            .bind(&dossier.title)
            .bind(&dossier.procedure)
            .bind(dossier.last_activity_date)
            .bind(&dossier.last_activity_label)
            .bind(dossier.score.progress as i16)
            .bind(dossier.score.magnitude as i16)
            .bind(dossier.score.momentum as i16)
            .bind(dossier.score.total as i16)
            .bind(&stage_code)
            .bind(&dossier.committee)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

            sqlx::query("DELETE FROM legislative_acts WHERE dossier_uid = $1")
                .bind(&dossier.uid)
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            for act in &dossier.acts {
                sqlx::query(
                    "INSERT INTO legislative_acts (dossier_uid, act_date, label) VALUES ($1, $2, $3)",
                )
                .bind(&dossier.uid)
                .bind(act.date)
                .bind(&act.label)
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            }

            sqlx::query("DELETE FROM dossier_initiators WHERE dossier_uid = $1")
                .bind(&dossier.uid)
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            for initiator in &dossier.initiators {
                sqlx::query(
                    "INSERT INTO dossier_initiators (dossier_uid, full_name, group_sigle) VALUES ($1, $2, $3)",
                )
                .bind(&dossier.uid)
                .bind(&initiator.full_name)
                .bind(&initiator.group)
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }

    async fn fetch_initiators(&self, dossier_uid: &str) -> Result<Vec<Initiator>, RepositoryError> {
        let rows = sqlx::query_as::<_, InitiatorRow>(
            "SELECT full_name, group_sigle FROM dossier_initiators WHERE dossier_uid = $1 ORDER BY id",
        )
        .bind(dossier_uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| Initiator {
                full_name: r.full_name,
                group: r.group_sigle,
            })
            .collect())
    }
}

const BATCH_SIZE: usize = 50;

#[async_trait]
impl DossierRepository for PgDossierRepository {
    async fn save_all(&self, dossiers: &[LegislativeDossier]) -> Result<usize, RepositoryError> {
        let mut saved = 0;
        for chunk in dossiers.chunks(BATCH_SIZE) {
            self.save_batch(chunk).await?;
            saved += chunk.len();
            tracing::debug!("Saved {saved}/{} dossiers", dossiers.len());
        }
        Ok(saved)
    }

    async fn find_recent(
        &self,
        since: NaiveDate,
    ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
        let rows = sqlx::query_as::<_, DossierRow>(
            "SELECT uid, title, procedure_label, last_activity_date, last_activity_label,
                    score_progress, score_magnitude, score_momentum, score_total,
                    current_stage_code, committee
             FROM legislative_dossiers
             WHERE last_activity_date >= $1
             ORDER BY last_activity_date DESC",
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_dossier(vec![], vec![])).collect())
    }

    async fn find_by_uid(
        &self,
        uid: &str,
    ) -> Result<Option<LegislativeDossier>, RepositoryError> {
        let row = sqlx::query_as::<_, DossierRow>(
            "SELECT uid, title, procedure_label, last_activity_date, last_activity_label,
                    score_progress, score_magnitude, score_momentum, score_total,
                    current_stage_code, committee
             FROM legislative_dossiers
             WHERE uid = $1",
        )
        .bind(uid)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let acts = self.fetch_acts(&r.uid).await?;
                let initiators = self.fetch_initiators(&r.uid).await?;
                Ok(Some(r.into_dossier(acts, initiators)))
            }
            None => Ok(None),
        }
    }
}

#[derive(sqlx::FromRow)]
struct DossierRow {
    uid: String,
    title: String,
    procedure_label: String,
    last_activity_date: NaiveDate,
    last_activity_label: String,
    score_progress: i16,
    score_magnitude: i16,
    score_momentum: i16,
    score_total: i16,
    current_stage_code: Option<String>,
    committee: Option<String>,
}

impl DossierRow {
    fn into_dossier(self, acts: Vec<LegislativeAct>, initiators: Vec<Initiator>) -> LegislativeDossier {
        LegislativeDossier {
            uid: self.uid,
            title: self.title,
            procedure: self.procedure_label,
            last_activity_date: self.last_activity_date,
            last_activity_label: self.last_activity_label,
            acts,
            score: Score {
                progress: self.score_progress as u8,
                magnitude: self.score_magnitude as u8,
                momentum: self.score_momentum as u8,
                total: self.score_total as u8,
            },
            current_stage: self
                .current_stage_code
                .as_deref()
                .and_then(LegislativeStage::from_code),
            initiators,
            committee: self.committee,
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActRow {
    act_date: NaiveDate,
    label: String,
}

#[derive(sqlx::FromRow)]
struct InitiatorRow {
    full_name: String,
    group_sigle: Option<String>,
}
