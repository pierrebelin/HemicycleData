-- Syntheses automatiques pre-calculees pour la fiche dossier.
-- Le texte est masquable tant que l'empreinte des faits n'est pas celle de la
-- derniere generation. Les faits bruts restent servis independamment.

CREATE TABLE dossier_group_summaries (
    dossier_uid TEXT NOT NULL REFERENCES legislative_dossiers(uid) ON DELETE CASCADE,
    group_uid TEXT NOT NULL REFERENCES parliamentary_groups(uid) ON DELETE CASCADE,
    status TEXT NOT NULL,
    paragraph TEXT,
    facts_fingerprint TEXT NOT NULL,
    model TEXT,
    prompt_version TEXT,
    generated_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (dossier_uid, group_uid),
    CONSTRAINT dossier_group_summaries_status CHECK (status IN ('pending', 'ready')),
    CONSTRAINT dossier_group_summaries_ready_text CHECK (status = 'pending' OR paragraph IS NOT NULL)
);

CREATE TABLE dossier_group_summary_sources (
    dossier_uid TEXT NOT NULL,
    group_uid TEXT NOT NULL,
    ordinal SMALLINT NOT NULL,
    source_id TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_uid TEXT NOT NULL,
    source_label TEXT NOT NULL,
    official_url TEXT,
    PRIMARY KEY (dossier_uid, group_uid, ordinal),
    FOREIGN KEY (dossier_uid, group_uid)
        REFERENCES dossier_group_summaries(dossier_uid, group_uid)
        ON DELETE CASCADE
);

CREATE INDEX idx_dossier_group_summaries_fingerprint
    ON dossier_group_summaries(dossier_uid, facts_fingerprint, status);
