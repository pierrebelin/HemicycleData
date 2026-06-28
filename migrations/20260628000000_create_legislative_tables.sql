CREATE TABLE legislative_dossiers (
    uid TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    procedure_label TEXT NOT NULL,
    last_activity_date DATE NOT NULL,
    last_activity_label TEXT NOT NULL,
    score_progress SMALLINT NOT NULL,
    score_magnitude SMALLINT NOT NULL,
    score_momentum SMALLINT NOT NULL,
    score_total SMALLINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE legislative_acts (
    id BIGSERIAL PRIMARY KEY,
    dossier_uid TEXT NOT NULL REFERENCES legislative_dossiers(uid) ON DELETE CASCADE,
    act_date DATE NOT NULL,
    label TEXT NOT NULL
);

CREATE INDEX idx_legislative_dossiers_last_activity ON legislative_dossiers(last_activity_date DESC);
CREATE INDEX idx_legislative_acts_dossier ON legislative_acts(dossier_uid);
