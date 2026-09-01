-- Candidatures présidentielles déclarées et programmes sourcés.
--
-- Cette table ne contient que des candidatures dont une déclaration publique
-- primaire est conservée. Les partis et les groupes parlementaires sont deux
-- référentiels distincts : aucun lien n'est déduit de leur seul nom.
CREATE TABLE presidential_candidates (
    id TEXT PRIMARY KEY CHECK (id ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
    display_name TEXT NOT NULL CHECK (length(trim(display_name)) > 0),
    declared_on DATE NOT NULL,
    declaration_source_url TEXT NOT NULL CHECK (declaration_source_url LIKE 'https://%'),
    declaration_source_label TEXT NOT NULL CHECK (length(trim(declaration_source_label)) > 0),
    official_site_url TEXT CHECK (official_site_url IS NULL OR official_site_url LIKE 'https://%'),
    program_url TEXT CHECK (program_url IS NULL OR program_url LIKE 'https://%'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Un candidat peut être soutenu par plusieurs organisations. Ce référentiel
-- est volontairement séparé des groupes de l'Assemblée nationale.
CREATE TABLE political_organizations (
    id TEXT PRIMARY KEY CHECK (id ~ '^[a-z0-9]+(?:-[a-z0-9]+)*$'),
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    official_url TEXT CHECK (official_url IS NULL OR official_url LIKE 'https://%')
);

CREATE TABLE candidate_political_organizations (
    candidate_id TEXT NOT NULL REFERENCES presidential_candidates(id) ON DELETE CASCADE,
    organization_id TEXT NOT NULL REFERENCES political_organizations(id) ON DELETE RESTRICT,
    source_url TEXT NOT NULL CHECK (source_url LIKE 'https://%'),
    source_label TEXT NOT NULL CHECK (length(trim(source_label)) > 0),
    PRIMARY KEY (candidate_id, organization_id)
);

-- Le rapprochement avec un groupe n'existe que lorsqu'une source l'établit.
-- `linked_on` est la date de la source, pas une datation artificielle du
-- comportement électoral : les votes gardent leur propre date de scrutin.
CREATE TABLE candidate_parliamentary_groups (
    candidate_id TEXT NOT NULL REFERENCES presidential_candidates(id) ON DELETE CASCADE,
    group_uid TEXT NOT NULL REFERENCES parliamentary_groups(uid) ON DELETE RESTRICT,
    linked_on DATE NOT NULL,
    source_url TEXT NOT NULL CHECK (source_url LIKE 'https://%'),
    source_label TEXT NOT NULL CHECK (length(trim(source_label)) > 0),
    PRIMARY KEY (candidate_id, group_uid)
);

-- Chaque proposition est un extrait attribué à une source du programme. Aucun
-- résumé libre ni indicateur d'alignement n'est stocké ou calculé ici.
CREATE TABLE candidate_program_proposals (
    id BIGSERIAL PRIMARY KEY,
    candidate_id TEXT NOT NULL REFERENCES presidential_candidates(id) ON DELETE CASCADE,
    family_code TEXT NOT NULL REFERENCES theme_families(code),
    excerpt TEXT NOT NULL CHECK (length(trim(excerpt)) > 0),
    source_url TEXT NOT NULL CHECK (source_url LIKE 'https://%'),
    source_label TEXT NOT NULL CHECK (length(trim(source_label)) > 0),
    source_published_on DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_candidate_program_proposals_candidate_theme
    ON candidate_program_proposals(candidate_id, family_code, id);
CREATE INDEX idx_candidate_parliamentary_groups_candidate
    ON candidate_parliamentary_groups(candidate_id);
