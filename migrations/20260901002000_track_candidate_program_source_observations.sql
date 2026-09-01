-- Les contenus de programme restent revus avant publication (README.md §8.2).
-- Cette table ne conserve donc que les métadonnées techniques permettant de
-- détecter une évolution d'une source déjà attribuée, jamais une copie du
-- contenu éditorial de cette source.
CREATE TABLE candidate_program_source_observations (
    source_url TEXT PRIMARY KEY CHECK (source_url LIKE 'https://%'),
    last_checked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    http_status SMALLINT NOT NULL CHECK (http_status BETWEEN 0 AND 599),
    etag TEXT,
    last_modified TEXT,
    content_sha256 TEXT CHECK (content_sha256 IS NULL OR content_sha256 ~ '^[0-9a-f]{64}$')
);

CREATE INDEX idx_candidate_program_source_observations_last_changed
    ON candidate_program_source_observations(last_changed_at DESC);
