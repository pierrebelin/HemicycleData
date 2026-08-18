-- Références des documents qui composent un dossier. Ces champs décrivent la
-- notice officielle, jamais un rapprochement inféré avec un scrutin.
ALTER TABLE dossier_documents
    ADD COLUMN official_url TEXT,
    ADD COLUMN source_archive_url TEXT,
    ADD COLUMN source_license TEXT,
    ADD COLUMN source_metadata_fingerprint TEXT,
    ADD COLUMN source_retrieved_at TIMESTAMPTZ;

CREATE INDEX idx_dossier_documents_uid ON dossier_documents(document_uid);
