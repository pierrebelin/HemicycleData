use async_trait::async_trait;
use chrono::NaiveDate;
use sqlx::PgPool;

use crate::application::ports::dossier_repository::{DossierRepository, RepositoryError};
use crate::domain::dossier::{Committee, CurationStatus, DossierUid, Initiator, LegislativeAct, LegislativeDocument, LegislativeStage, LegislativeDossier, Score};

pub struct PgDossierRepository {
    pool: PgPool,
}

impl PgDossierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    async fn fetch_acts(&self, dossier_uid: &str) -> Result<Vec<LegislativeAct>, RepositoryError> {
        let rows = sqlx::query_as::<_, ActRow>(
            "SELECT act_date, label, act_code FROM legislative_acts WHERE dossier_uid = $1 ORDER BY act_date",
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
                code: r.act_code,
            })
            .collect())
    }

    async fn fetch_documents(&self, dossier_uid: &str) -> Result<Vec<LegislativeDocument>, RepositoryError> {
        let rows = sqlx::query_as::<_, DocumentRow>(
            "SELECT document_uid, title, short_title, doc_type, doc_date FROM dossier_documents WHERE dossier_uid = $1 ORDER BY id",
        )
        .bind(dossier_uid)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows
            .into_iter()
            .map(|r| LegislativeDocument {
                document_uid: r.document_uid,
                title: r.title,
                short_title: r.short_title,
                doc_type: r.doc_type,
                date: r.doc_date,
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
                "INSERT INTO legislative_dossiers (uid, title, procedure_label, last_activity_date, last_activity_label, score_progress, score_magnitude, score_momentum, score_total, current_stage_code, committee, curation_status, legislature, url, summary)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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
                    legislature = EXCLUDED.legislature,
                    url = EXCLUDED.url,
                    summary = COALESCE(EXCLUDED.summary, legislative_dossiers.summary),
                    updated_at = NOW()",
            )
            .bind(dossier.uid.as_str())
            .bind(&dossier.title)
            .bind(&dossier.procedure)
            .bind(dossier.last_activity_date)
            .bind(&dossier.last_activity_label)
            .bind(dossier.score.progress() as i16)
            .bind(dossier.score.magnitude() as i16)
            .bind(dossier.score.momentum() as i16)
            .bind(dossier.score.total() as i16)
            .bind(&stage_code)
            .bind(dossier.committee.as_ref().map(|c| c.as_str()))
            .bind(dossier.curation_status.as_str())
            .bind(dossier.legislature as i16)
            .bind(&dossier.url)
            .bind(&dossier.summary)
            .execute(&mut *tx)
            .await
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

            sqlx::query("DELETE FROM legislative_acts WHERE dossier_uid = $1")
                .bind(dossier.uid.as_str())
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            for act in &dossier.acts {
                sqlx::query(
                    "INSERT INTO legislative_acts (dossier_uid, act_date, label, act_code) VALUES ($1, $2, $3, $4)",
                )
                .bind(dossier.uid.as_str())
                .bind(act.date)
                .bind(&act.label)
                .bind(&act.code)
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            }

            sqlx::query("DELETE FROM dossier_initiators WHERE dossier_uid = $1")
                .bind(dossier.uid.as_str())
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            for initiator in &dossier.initiators {
                sqlx::query(
                    "INSERT INTO dossier_initiators (dossier_uid, full_name, group_sigle) VALUES ($1, $2, $3)",
                )
                .bind(dossier.uid.as_str())
                .bind(initiator.full_name())
                .bind(initiator.group())
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            }

            sqlx::query("DELETE FROM dossier_documents WHERE dossier_uid = $1")
                .bind(dossier.uid.as_str())
                .execute(&mut *tx)
                .await
                .map_err(|e| RepositoryError::Database(e.to_string()))?;

            for doc in &dossier.documents {
                sqlx::query(
                    "INSERT INTO dossier_documents (dossier_uid, document_uid, title, short_title, doc_type, doc_date) VALUES ($1, $2, $3, $4, $5, $6)",
                )
                .bind(dossier.uid.as_str())
                .bind(&doc.document_uid)
                .bind(&doc.title)
                .bind(&doc.short_title)
                .bind(&doc.doc_type)
                .bind(doc.date)
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
            .map(|r| {
                Initiator::new(r.full_name, r.group_sigle)
                    .expect("DB initiator name is non-empty")
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
                    current_stage_code, committee, curation_status,
                    legislature, url, summary
             FROM legislative_dossiers
             WHERE last_activity_date >= $1
             ORDER BY last_activity_date DESC",
        )
        .bind(since)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_dossier(vec![], vec![], vec![])).collect())
    }

    async fn find_by_uid(
        &self,
        uid: &DossierUid,
    ) -> Result<Option<LegislativeDossier>, RepositoryError> {
        let row = sqlx::query_as::<_, DossierRow>(
            "SELECT uid, title, procedure_label, last_activity_date, last_activity_label,
                    score_progress, score_magnitude, score_momentum, score_total,
                    current_stage_code, committee, curation_status,
                    legislature, url, summary
             FROM legislative_dossiers
             WHERE uid = $1",
        )
        .bind(uid.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => {
                let uid_str = r.uid.clone();
                let acts = self.fetch_acts(&uid_str).await?;
                let initiators = self.fetch_initiators(&uid_str).await?;
                let documents = self.fetch_documents(&uid_str).await?;
                Ok(Some(r.into_dossier(acts, initiators, documents)))
            }
            None => Ok(None),
        }
    }

    async fn find_suggestions(
        &self,
        count: usize,
    ) -> Result<Vec<LegislativeDossier>, RepositoryError> {
        let rows = sqlx::query_as::<_, DossierRow>(
            "SELECT uid, title, procedure_label, last_activity_date, last_activity_label,
                    score_progress, score_magnitude, score_momentum, score_total,
                    current_stage_code, committee, curation_status,
                    legislature, url, summary
             FROM legislative_dossiers
             WHERE curation_status = 'new'
             ORDER BY score_total DESC
             LIMIT $1",
        )
        .bind(count as i64)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(rows.into_iter().map(|r| r.into_dossier(vec![], vec![], vec![])).collect())
    }

    async fn update_curation_status(
        &self,
        uid: &DossierUid,
        status: CurationStatus,
    ) -> Result<bool, RepositoryError> {
        let result = sqlx::query(
            "UPDATE legislative_dossiers SET curation_status = $1, updated_at = NOW() WHERE uid = $2",
        )
        .bind(status.as_str())
        .bind(uid.as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(result.rows_affected() > 0)
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
    curation_status: String,
    legislature: i16,
    url: Option<String>,
    summary: Option<String>,
}

impl DossierRow {
    fn into_dossier(self, acts: Vec<LegislativeAct>, initiators: Vec<Initiator>, documents: Vec<LegislativeDocument>) -> LegislativeDossier {
        LegislativeDossier {
            uid: DossierUid::new(self.uid).expect("DB uid is non-empty"),
            title: self.title,
            procedure: self.procedure_label,
            legislature: self.legislature as u16,
            url: self.url,
            summary: self.summary,
            last_activity_date: self.last_activity_date,
            last_activity_label: self.last_activity_label,
            acts,
            documents,
            score: Score::new(
                self.score_progress as u8,
                self.score_magnitude as u8,
                self.score_momentum as u8,
                self.score_total as u8,
            )
            .expect("DB scores are in valid range"),
            current_stage: self
                .current_stage_code
                .as_deref()
                .and_then(LegislativeStage::from_code),
            initiators,
            committee: self.committee.map(|c| Committee::new(c).expect("DB committee is non-empty")),
            curation_status: CurationStatus::parse(&self.curation_status).unwrap_or(CurationStatus::New),
        }
    }
}

#[derive(sqlx::FromRow)]
struct ActRow {
    act_date: NaiveDate,
    label: String,
    act_code: Option<String>,
}

#[derive(sqlx::FromRow)]
struct InitiatorRow {
    full_name: String,
    group_sigle: Option<String>,
}

#[derive(sqlx::FromRow)]
struct DocumentRow {
    document_uid: String,
    title: String,
    short_title: Option<String>,
    doc_type: String,
    doc_date: Option<NaiveDate>,
}
