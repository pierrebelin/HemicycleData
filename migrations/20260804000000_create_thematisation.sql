-- Thematisation (todo/SPEC-thematisation.md).
--
-- Le porteur du rattachement est le texte debattu, pas le dossier: mesure du
-- 3 aout 2026, la source publie le lien dossier de facon irreguliere a
-- l'interieur d'un meme texte (H3).

CREATE TABLE theme_families (
    code TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    scope TEXT NOT NULL,
    display_order SMALLINT NOT NULL
);

-- Referentiel ferme (RM-08). Libelles repris de README.md §5.
INSERT INTO theme_families (code, label, scope, display_order) VALUES
    ('pouvoir-achat-fiscalite', 'Pouvoir d''achat / fiscalité',
     'Impôts, taxes, prestations monétaires, prix, budget de l''État.', 1),
    ('logement', 'Logement',
     'Loyers, accès à la propriété, construction, locations de courte durée, urbanisme.', 2),
    ('travail-emploi', 'Travail / emploi',
     'Droit du travail, chômage, retraites, indépendants, dialogue social.', 3),
    ('environnement-energie', 'Environnement / énergie',
     'Prix de l''énergie, transition, transports, agriculture, eau, biodiversité.', 4),
    ('numerique', 'Numérique',
     'Données personnelles, intelligence artificielle, réseaux sociaux, fraude en ligne.', 5),
    ('sante-social', 'Santé / social',
     'Remboursements, accès aux soins, hôpital, congés, handicap, action sociale.', 6),
    ('societe-libertes', 'Société / libertés',
     'Justice, sécurité, immigration, égalité, fin de vie, éducation, culture. Rattachement sur l''objet du texte, jamais sur son orientation.', 7),
    ('institutions-procedure', 'Institutions / procédure',
     'Motions de censure, révisions constitutionnelles, lois de finances dans leur volet procédural, collectivités, élections.', 8);

-- Texte nomme par l'objet d'un scrutin, cle normalisee (RM-02).
CREATE TABLE debated_texts (
    text_key TEXT PRIMARY KEY,
    label TEXT NOT NULL,
    -- Suivi de la derniere tentative de proposition. Distingue « le modele n'a
    -- retenu aucune famille » de « le modele n'a pas repondu » sur la page
    -- methode.
    last_attempt_on DATE,
    last_attempt_outcome TEXT
        CHECK (last_attempt_outcome IN ('proposed', 'no_family', 'failed')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Un scrutin porte au plus un texte. Les 7 objets qui n'en nomment aucun
-- n'ont pas de ligne ici et restent consultables (RM-01, H1).
CREATE TABLE scrutin_debated_texts (
    scrutin_uid TEXT PRIMARY KEY REFERENCES scrutins(uid) ON DELETE CASCADE,
    text_key TEXT NOT NULL REFERENCES debated_texts(text_key) ON DELETE CASCADE
);

CREATE INDEX idx_scrutin_debated_texts_key ON scrutin_debated_texts(text_key);

-- Lien dossier -> texte, etabli par les scrutins que la source rattache aux
-- deux. Aucune correspondance devinee sur les libelles: le dossier « Fin de
-- vie » ne ressemble pas a « proposition de loi relative au droit a l'aide a
-- mourir », seuls ses scrutins les relient (RM-06).
-- Cle composite: un dossier dont les scrutins nomment deux textes les porte
-- tous les deux. Choisir le plus vote perdrait le second (Q2).
CREATE TABLE dossier_debated_texts (
    dossier_uid TEXT NOT NULL,
    text_key TEXT NOT NULL REFERENCES debated_texts(text_key) ON DELETE CASCADE,
    scrutin_count INTEGER NOT NULL,
    PRIMARY KEY (dossier_uid, text_key)
);

CREATE INDEX idx_dossier_debated_texts_key ON dossier_debated_texts(text_key);

-- Rattachement date. Une revision clot la ligne et en ouvre une autre: rien
-- n'est supprime (RM-07).
CREATE TABLE theme_assignments (
    id BIGSERIAL PRIMARY KEY,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('text', 'dossier')),
    subject_id TEXT NOT NULL,
    family_code TEXT NOT NULL REFERENCES theme_families(code),
    origin TEXT NOT NULL CHECK (origin IN ('proposal', 'human_arbitration')),
    opened_on DATE NOT NULL,
    closed_on DATE,
    author TEXT NOT NULL,
    motive TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT theme_assignments_dates CHECK (closed_on IS NULL OR closed_on >= opened_on)
);

CREATE UNIQUE INDEX idx_theme_assignments_current
    ON theme_assignments(subject_kind, subject_id, family_code)
    WHERE closed_on IS NULL;

CREATE INDEX idx_theme_assignments_subject
    ON theme_assignments(subject_kind, subject_id);

CREATE INDEX idx_theme_assignments_family_current
    ON theme_assignments(family_code)
    WHERE closed_on IS NULL;

-- Proposition du modele, conservee telle que rendue. Aucun nombre: le modele
-- n'en produit pas (RM-10).
CREATE TABLE theme_proposals (
    id BIGSERIAL PRIMARY KEY,
    subject_kind TEXT NOT NULL CHECK (subject_kind IN ('text', 'dossier')),
    subject_id TEXT NOT NULL,
    model TEXT NOT NULL,
    prompt_version TEXT NOT NULL,
    produced_on DATE NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_theme_proposals_subject ON theme_proposals(subject_kind, subject_id);

CREATE TABLE theme_proposal_families (
    proposal_id BIGINT NOT NULL REFERENCES theme_proposals(id) ON DELETE CASCADE,
    family_code TEXT NOT NULL REFERENCES theme_families(code),
    ordinal SMALLINT NOT NULL,
    justification TEXT NOT NULL,
    PRIMARY KEY (proposal_id, family_code)
);
