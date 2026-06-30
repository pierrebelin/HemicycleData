ALTER TABLE legislative_dossiers
    ADD COLUMN curation_status TEXT NOT NULL DEFAULT 'new';

CREATE INDEX idx_legislative_dossiers_curation ON legislative_dossiers(curation_status, score_total DESC);
