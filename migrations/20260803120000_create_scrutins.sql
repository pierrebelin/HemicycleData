-- Scrutins publics, repartition par groupe et positions nominales.
-- Source: archive officielle AN des scrutins. Voir todo/SPEC-scrutins.md.

CREATE TABLE scrutins (
    uid TEXT PRIMARY KEY,
    number TEXT NOT NULL,
    legislature SMALLINT NOT NULL,
    scrutin_date DATE NOT NULL,
    session_ref TEXT,
    sitting_ref TEXT,
    place TEXT,
    ballot_type_code TEXT NOT NULL,
    ballot_type_label TEXT NOT NULL,
    majority_label TEXT,
    outcome_code TEXT NOT NULL,
    outcome_label TEXT NOT NULL,
    requester TEXT,
    subject TEXT NOT NULL,
    -- Synthese officielle, publiee telle quelle (RM-02).
    voters SMALLINT NOT NULL,
    expressed SMALLINT NOT NULL,
    required SMALLINT NOT NULL,
    announcement TEXT NOT NULL,
    votes_for SMALLINT NOT NULL,
    votes_against SMALLINT NOT NULL,
    abstentions SMALLINT NOT NULL,
    not_voting SMALLINT NOT NULL,
    voluntary_not_voting SMALLINT NOT NULL,
    -- RM-10: aucune cle etrangere vers legislative_dossiers. 69 % des scrutins
    -- n'ont pas de dossier, et un dossier reference n'est pas forcement ingere:
    -- une contrainte referentielle rejetterait des scrutins que le site doit
    -- exposer. Le libelle publie est conserve pour rester lisible sans jointure.
    dossier_uid TEXT,
    dossier_label TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT scrutins_dossier_pair CHECK ((dossier_uid IS NULL) = (dossier_label IS NULL))
);

-- Une ligne par groupe. origin = 'published' quand la source la publie,
-- 'reconstructed' quand elle est recalculee depuis les positions nominales
-- (RM-03): l'affichage doit alors porter la mention de methode.
CREATE TABLE scrutin_group_tallies (
    scrutin_uid TEXT NOT NULL REFERENCES scrutins(uid) ON DELETE CASCADE,
    group_uid TEXT NOT NULL,
    member_count SMALLINT,
    majority_position TEXT,
    votes_for SMALLINT NOT NULL,
    votes_against SMALLINT NOT NULL,
    abstentions SMALLINT NOT NULL,
    not_voting SMALLINT NOT NULL,
    voluntary_not_voting SMALLINT NOT NULL,
    origin TEXT NOT NULL,
    PRIMARY KEY (scrutin_uid, group_uid),
    CONSTRAINT scrutin_group_tallies_origin CHECK (origin IN ('published', 'reconstructed'))
);

-- RM-04: group_uid est le groupe sous lequel la source range le votant dans ce
-- scrutin. Date par construction, aucune jointure sur l'appartenance courante.
-- Pas de cle etrangere vers actors ni parliamentary_groups: une position ne
-- doit jamais disparaitre parce que le referentiel est en retard.
CREATE TABLE scrutin_votes (
    scrutin_uid TEXT NOT NULL REFERENCES scrutins(uid) ON DELETE CASCADE,
    actor_uid TEXT NOT NULL,
    group_uid TEXT,
    position TEXT NOT NULL,
    cause_code TEXT,
    by_delegation BOOLEAN NOT NULL,
    seat SMALLINT,
    PRIMARY KEY (scrutin_uid, actor_uid),
    CONSTRAINT scrutin_votes_position
        CHECK (position IN ('for', 'against', 'abstention', 'not_voting'))
);

-- Mises au point: RM-05, aucun effet sur les decomptes ci-dessus.
CREATE TABLE scrutin_vote_corrections (
    scrutin_uid TEXT NOT NULL REFERENCES scrutins(uid) ON DELETE CASCADE,
    actor_uid TEXT NOT NULL,
    claimed_position TEXT NOT NULL,
    malfunction BOOLEAN NOT NULL,
    PRIMARY KEY (scrutin_uid, actor_uid),
    CONSTRAINT scrutin_vote_corrections_position
        CHECK (claimed_position IN ('for', 'against', 'abstention', 'not_voting'))
);

CREATE INDEX idx_scrutins_date ON scrutins(scrutin_date DESC, number DESC);
CREATE INDEX idx_scrutins_dossier ON scrutins(dossier_uid) WHERE dossier_uid IS NOT NULL;
CREATE INDEX idx_scrutin_group_tallies_group ON scrutin_group_tallies(group_uid);
CREATE INDEX idx_scrutin_votes_actor ON scrutin_votes(actor_uid);
CREATE INDEX idx_scrutin_votes_group ON scrutin_votes(scrutin_uid, group_uid);
