ALTER TABLE legislative_dossiers
    ADD COLUMN current_stage_code TEXT,
    ADD COLUMN committee TEXT;

CREATE TABLE dossier_initiators (
    id BIGSERIAL PRIMARY KEY,
    dossier_uid TEXT NOT NULL REFERENCES legislative_dossiers(uid) ON DELETE CASCADE,
    full_name TEXT NOT NULL,
    group_sigle TEXT
);

CREATE INDEX idx_dossier_initiators_dossier ON dossier_initiators(dossier_uid);
