-- Version précise d'un texte explicitement reliée à un vote sur l'ensemble.
--
-- Cette table n'est jamais alimentée par ressemblance d'intitulé, date ou
-- proximité dans un dossier. Chaque ligne doit citer l'acte officiel qui
-- établit le lien entre le scrutin et le document. Sans ligne, le site expose
-- le vote mais ne prétend pas connaître la version exacte du texte.
CREATE TABLE final_vote_text_versions (
    scrutin_uid TEXT PRIMARY KEY REFERENCES scrutins(uid) ON DELETE CASCADE,
    document_uid TEXT NOT NULL CHECK (length(trim(document_uid)) > 0),
    document_title TEXT NOT NULL CHECK (length(trim(document_title)) > 0),
    version_label TEXT NOT NULL CHECK (length(trim(version_label)) > 0),
    document_published_on DATE,
    official_url TEXT NOT NULL CHECK (official_url LIKE 'https://%'),
    -- URL de la séance, analyse de scrutin ou autre publication officielle qui
    -- fait le lien. Elle est distincte de l'URL du document à lire.
    mapping_source_url TEXT NOT NULL CHECK (mapping_source_url LIKE 'https://%'),
    source_producer TEXT NOT NULL CHECK (length(trim(source_producer)) > 0),
    source_license TEXT NOT NULL CHECK (length(trim(source_license)) > 0),
    source_metadata_fingerprint TEXT,
    source_retrieved_at TIMESTAMPTZ NOT NULL
);
