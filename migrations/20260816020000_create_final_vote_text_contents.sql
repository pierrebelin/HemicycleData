-- Copie horodatee d'une version Open Data HTML explicitement rattachee a un
-- vote sur l'ensemble. Le HTML brut reste la reference pour les syntheses et
-- la future lecture integrale ; le texte extrait ne sert qu'a la recherche et
-- a la generation. Cette table n'est pas exposee directement au lecteur.
CREATE TABLE final_vote_text_contents (
    scrutin_uid TEXT PRIMARY KEY
        REFERENCES final_vote_text_versions(scrutin_uid) ON DELETE CASCADE,
    content_url TEXT NOT NULL CHECK (content_url LIKE 'https://%'),
    document_html TEXT NOT NULL CHECK (length(trim(document_html)) > 0),
    document_text TEXT NOT NULL CHECK (length(trim(document_text)) > 0),
    content_fingerprint TEXT NOT NULL CHECK (content_fingerprint LIKE 'sha256:%'),
    source_retrieved_at TIMESTAMPTZ NOT NULL
);
