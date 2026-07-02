ALTER TABLE legislative_dossiers
    ADD COLUMN legislature SMALLINT NOT NULL DEFAULT 17,
    ADD COLUMN url TEXT,
    ADD COLUMN summary TEXT;

ALTER TABLE legislative_acts
    ADD COLUMN act_code TEXT;

CREATE TABLE dossier_documents (
    id BIGSERIAL PRIMARY KEY,
    dossier_uid TEXT NOT NULL REFERENCES legislative_dossiers(uid) ON DELETE CASCADE,
    document_uid TEXT NOT NULL,
    title TEXT NOT NULL,
    short_title TEXT,
    doc_type TEXT NOT NULL,
    doc_date DATE
);

CREATE INDEX idx_dossier_documents_dossier ON dossier_documents(dossier_uid);
