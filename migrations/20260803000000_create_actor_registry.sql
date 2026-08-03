-- Referentiel des acteurs, groupes parlementaires et appartenances datees.
-- Source: jeu historique officiel AN (AMO30). Voir todo/SPEC-acteurs-appartenances.md.

CREATE TABLE actors (
    uid TEXT PRIMARY KEY,
    civility TEXT,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE parliamentary_groups (
    uid TEXT PRIMARY KEY,
    legislature SMALLINT NOT NULL,
    label TEXT NOT NULL,
    abbrev TEXT NOT NULL,
    color TEXT,
    start_date DATE,
    end_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- source_uid = identifiant du mandat a la source (PM...), cle naturelle stable.
-- Une appartenance close n'est jamais supprimee: elle porte les actes de sa periode.
CREATE TABLE group_memberships (
    source_uid TEXT PRIMARY KEY,
    actor_uid TEXT NOT NULL REFERENCES actors(uid) ON DELETE CASCADE,
    group_uid TEXT NOT NULL REFERENCES parliamentary_groups(uid) ON DELETE CASCADE,
    legislature SMALLINT NOT NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    quality TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT group_memberships_period_ordered CHECK (end_date IS NULL OR end_date >= start_date)
);

CREATE INDEX idx_group_memberships_actor ON group_memberships(actor_uid, start_date, end_date);
CREATE INDEX idx_group_memberships_group ON group_memberships(group_uid);
CREATE INDEX idx_parliamentary_groups_legislature ON parliamentary_groups(legislature);

-- Date de depot du dossier: date de reference du rattachement des initiateurs (RM-01).
ALTER TABLE legislative_dossiers
    ADD COLUMN deposit_date DATE;

CREATE INDEX idx_legislative_dossiers_deposit ON legislative_dossiers(deposit_date);

-- Initiateurs: le groupe devient date et trace jusqu'a la source.
-- group_sigle provenait de nosdeputes.fr (source tierce, interdite par RM-05)
-- et portait le groupe COURANT, pas celui de la date de l'acte (RM-01).
ALTER TABLE dossier_initiators
    DROP COLUMN group_sigle,
    ADD COLUMN actor_uid TEXT,
    ADD COLUMN group_uid TEXT,
    ADD COLUMN group_abbrev TEXT,
    ADD COLUMN group_label TEXT,
    ADD COLUMN membership_quality TEXT,
    ADD COLUMN reference_date DATE,
    ADD COLUMN official_url TEXT,
    ADD COLUMN actor_role TEXT;

-- RM-01: aucun groupe affiche sans sa date de reference.
ALTER TABLE dossier_initiators
    ADD CONSTRAINT dossier_initiators_group_requires_reference_date
    CHECK (group_uid IS NULL OR reference_date IS NOT NULL);
