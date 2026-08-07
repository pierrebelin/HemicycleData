-- Sort du dossier, derive des actes a l'ingestion.
--
-- 'no_recorded_conclusion' est la valeur par defaut parce que c'est le cas
-- majoritaire et le seul honnete quand la source ne conclut rien : 2 788 des
-- 3 035 dossiers de la legislature 17 ne portent aucun acte de conclusion.
ALTER TABLE legislative_dossiers
    ADD COLUMN outcome_kind TEXT NOT NULL DEFAULT 'no_recorded_conclusion',
    ADD COLUMN outcome_date DATE,
    ADD COLUMN outcome_label TEXT,
    ADD COLUMN law_code TEXT,
    ADD COLUMN law_jo_date DATE,
    ADD COLUMN law_legifrance_url TEXT,
    ADD COLUMN merged_into_uid TEXT,
    ADD COLUMN merge_cause TEXT;

-- Le rafraichissement lit ces trois colonnes pour tous les dossiers avant
-- d'ecrire quoi que ce soit : il saute ceux dont rien n'a bouge.
CREATE INDEX idx_legislative_dossiers_outcome
    ON legislative_dossiers(outcome_kind, last_activity_date DESC);
