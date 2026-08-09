-- Amendements: prises de position ecrites et signees, rattachees a un texte
-- legislatif et rapprochees d'un scrutin quand l'objet le designe.
-- Source: archive officielle AN des amendements. Voir todo/SPEC-amendements.md.

CREATE TABLE amendments (
    uid TEXT PRIMARY KEY,
    legislature SMALLINT NOT NULL,
    -- Numero publie: une chaine, jamais un entier (« 45 rect. », « CF120 »).
    number TEXT NOT NULL,
    -- Forme normalisee du numero. Ecrite des maintenant, lue par le lot qui
    -- rapprochera l'amendement de l'objet d'un scrutin (RM-06): la calculer
    -- apres coup demanderait de reparcourir la table entiere.
    number_key TEXT NOT NULL,
    -- RM-05: identifiant du texte legislatif publie par la source. Aucune cle
    -- etrangere vers dossier_documents: un amendement dont le texte n'est pas
    -- encore ingere doit entrer quand meme (RM-01), exactement comme un scrutin
    -- sans dossier (voir 20260803120000_create_scrutins.sql). Le rattachement au
    -- dossier se fait par jointure a la lecture, pas par une colonne figee a
    -- l'ingestion: les dossiers arrivent en incremental, une jointure se repare
    -- toute seule a la passe suivante.
    text_ref TEXT,
    examination_ref TEXT,
    target_title TEXT NOT NULL,
    target_kind TEXT,
    -- Auteur. 'deputy': author_actor_uid renseigne. 'institutional':
    -- Gouvernement ou commission, aucun acteur, le libelle publie fait foi.
    author_kind TEXT NOT NULL,
    author_actor_uid TEXT,
    -- Renseigne pour un auteur institutionnel seulement. Le nom d'un depute
    -- n'est pas denormalise ici: il se resout a la lecture depuis le referentiel
    -- des acteurs, comme pour les positions nominales d'un scrutin. Une copie du
    -- nom vieillirait sans que rien ne la rafraichisse.
    author_label TEXT,
    author_group_uid TEXT,
    author_group_origin TEXT NOT NULL,
    author_group_ambiguous BOOLEAN NOT NULL DEFAULT FALSE,
    -- RM-04: `code` sert aux filtres, `label` s'affiche tel quel. 'other' = sort
    -- publie hors referentiel, conserve et compte, jamais range d'office dans la
    -- categorie voisine.
    fate_code TEXT NOT NULL,
    fate_label TEXT NOT NULL,
    state_label TEXT,
    deposited_on DATE,
    parent_uid TEXT,
    -- RM-03: expose sommaire verbatim, sans troncature ni resume. Le dispositif
    -- de l'amendement n'est pas stocke: il tient sur la page officielle, et le
    -- porteur de sens pour le lecteur est l'expose.
    summary TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT amendments_fate_code CHECK (fate_code IN (
        'adopted', 'rejected', 'withdrawn', 'fallen', 'not_supported',
        'inadmissible', 'not_discussed', 'unspecified', 'other')),
    CONSTRAINT amendments_author_kind CHECK (author_kind IN ('deputy', 'institutional')),
    CONSTRAINT amendments_author_pair
        CHECK ((author_kind = 'deputy') = (author_actor_uid IS NOT NULL)),
    CONSTRAINT amendments_author_label
        CHECK ((author_kind = 'institutional') = (author_label IS NOT NULL)),
    CONSTRAINT amendments_author_group_origin
        CHECK (author_group_origin IN ('published', 'resolved_at_deposit', 'unknown')),
    CONSTRAINT amendments_author_group_pair
        CHECK ((author_group_origin = 'unknown') = (author_group_uid IS NULL))
);

-- RM-02: group_uid est le groupe du signataire A LA DATE DE DEPOT.
-- group_origin = 'published' quand la source le nomme dans l'amendement (date
-- par construction), 'resolved_at_deposit' quand le site le reconstitue depuis
-- l'appartenance datee, 'unknown' quand rien n'est resoluble. Un groupe courant
-- n'entre jamais ici.
--
-- Pas de cle etrangere vers actors: une signature ne doit pas disparaitre parce
-- que le referentiel est en retard (meme motif que scrutin_votes).
CREATE TABLE amendment_signatories (
    amendment_uid TEXT NOT NULL REFERENCES amendments(uid) ON DELETE CASCADE,
    actor_uid TEXT NOT NULL,
    role TEXT NOT NULL,
    -- Rang publie. Restitue l'ordre de la source, ne classe rien.
    rank SMALLINT NOT NULL,
    group_uid TEXT,
    group_origin TEXT NOT NULL,
    group_ambiguous BOOLEAN NOT NULL DEFAULT FALSE,
    -- Denormalisee depuis amendments: la page d'un depute lit une suite datee,
    -- l'index (actor_uid, deposited_on DESC) la sert sans tri a la volee.
    deposited_on DATE,
    PRIMARY KEY (amendment_uid, actor_uid),
    CONSTRAINT amendment_signatories_role CHECK (role IN ('author', 'cosignatory')),
    CONSTRAINT amendment_signatories_group_origin
        CHECK (group_origin IN ('published', 'resolved_at_deposit', 'unknown')),
    CONSTRAINT amendment_signatories_group_pair
        CHECK ((group_origin = 'unknown') = (group_uid IS NULL))
);

-- Sert la liste des amendements d'un dossier, ordonnee par depot (RM-07):
-- l'ordre de depot est mecanique, la ou un tri sur le numero publie melerait
-- « 100 » et « 99 » et la ou un tri par nombre de cosignataires serait un
-- classement.
CREATE INDEX idx_amendments_text_deposit
    ON amendments(text_ref, deposited_on, uid) WHERE text_ref IS NOT NULL;
CREATE INDEX idx_amendments_author
    ON amendments(author_actor_uid) WHERE author_actor_uid IS NOT NULL;
-- Page depute: une suite datee, sans tri a la volee.
CREATE INDEX idx_amendment_signatories_actor
    ON amendment_signatories(actor_uid, deposited_on DESC, amendment_uid DESC);
-- Manquant jusqu'ici, et indispensable a la jointure amendement -> dossier
-- (RM-05): dossier_documents n'etait indexe que par dossier_uid.
CREATE INDEX idx_dossier_documents_document ON dossier_documents(document_uid);

-- Empreinte de la derniere archive entierement ingeree, par source.
--
-- Une passe d'amendements reparse plusieurs centaines de milliers de fichiers.
-- La cadence de deux heures ne tient que si une archive inchangee est reconnue
-- comme telle: ArchiveFetcher evite deja le retelechargement (GET conditionnel),
-- cette table evite le reparsing.
CREATE TABLE source_archives (
    label TEXT PRIMARY KEY,
    digest TEXT NOT NULL,
    ingested_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
