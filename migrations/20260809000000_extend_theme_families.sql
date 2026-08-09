-- Referentiel thematique porte de 8 a 13 familles, et rattachement par regle
-- publiee (todo/SPEC-thematisation.md, README.md §5).
--
-- Trois manques dans les huit familles precedentes:
--   1. « societe / libertes » melait justice, securite, immigration, education
--      et culture — les sujets les plus disputes de la legislature dans un seul
--      bac, celui qu'on avait soi-meme marque « terrain sensible »;
--   2. aucune famille n'accueillait l'international ni la defense: les
--      ratifications de traites tombaient dans « institutions » ou nulle part;
--   3. l'agriculture etait rangee sous l'environnement, ce qui est deja une
--      prise de position sur le sujet le plus tendu entre les deux.
--
-- Le decoupage porte sur l'objet des textes, jamais sur leur orientation
-- (README.md §6).

-- 1. Perimetres resserres des familles qui perdent de la matiere -------------

UPDATE theme_families
   SET label = 'Santé / social',
       scope = 'Remboursements, accès aux soins, hôpital, handicap, action sociale, politique familiale.',
       display_order = 4
 WHERE code = 'sante-social';

UPDATE theme_families
   SET scope = 'Prix de l''énergie, transition, transports, eau, biodiversité, déchets.',
       display_order = 5
 WHERE code = 'environnement-energie';

UPDATE theme_families
   SET scope = 'Égalité, droits des personnes, fin de vie, bioéthique, laïcité, libertés publiques. Rattachement sur l''objet du texte, jamais sur son orientation.',
       display_order = 11
 WHERE code = 'societe-libertes';

UPDATE theme_families SET display_order = 7 WHERE code = 'numerique';
UPDATE theme_families SET display_order = 13 WHERE code = 'institutions-procedure';

-- 2. Les cinq nouvelles familles ---------------------------------------------

INSERT INTO theme_families (code, label, scope, display_order) VALUES
    ('agriculture-alimentation', 'Agriculture / alimentation',
     'Revenu agricole, pêche, produits phytosanitaires, alimentation, foncier agricole.', 6),
    ('justice-securite', 'Justice / sécurité',
     'Droit pénal, police, gendarmerie, prisons, terrorisme, procédure judiciaire.', 8),
    ('immigration', 'Immigration',
     'Entrée et séjour des étrangers, asile, éloignement, nationalité. Rattachement sur l''objet du texte, jamais sur son orientation.', 9),
    ('education-culture', 'Éducation / culture',
     'École, université, recherche, sport, culture, médias, audiovisuel.', 10),
    ('international-defense', 'International / défense',
     'Ratification de traités, armées, aide au développement, affaires européennes.', 12);

-- 3. L'origine du rattachement disparait --------------------------------------
--
-- `origin` rangeait chaque rattachement en « proposition automatique » ou
-- « arbitrage humain », et l'affichait a cote de la famille. La distinction
-- n'apprend rien au lecteur: un texte est rattache au logement ou il ne l'est
-- pas, savoir qui a ouvert la ligne ne change pas le vote qu'il va lire.
--
-- L'audit ne disparait pas pour autant: `author` reste renseigne en clair —
-- nom de la regle, nom du modele, nom du mainteneur — et l'historique conserve
-- chaque ligne close (RM-07, README.md §9). Ce qui est retire, c'est la
-- taxonomie affichee, pas la tracabilite.
--
-- La contrainte `theme_assignments_origin_check` tombe avec la colonne.

ALTER TABLE theme_assignments DROP COLUMN origin;

ALTER TABLE debated_texts DROP CONSTRAINT debated_texts_last_attempt_outcome_check;
ALTER TABLE debated_texts ADD CONSTRAINT debated_texts_last_attempt_outcome_check
    CHECK (last_attempt_outcome IN ('ruled', 'proposed', 'no_family', 'failed'));

-- 4. Suivi de tentative pour les dossiers sans scrutin ------------------------
--
-- Un dossier relie a un texte herite de ses familles (RM-06) et ne passe jamais
-- par le modele. Ceux qu'aucun scrutin ne relie sont classes sur leur titre; il
-- leur faut le meme suivi qu'aux textes pour ne jamais etre resoumis deux fois.

CREATE TABLE dossier_theme_attempts (
    dossier_uid TEXT PRIMARY KEY,
    last_attempt_on DATE NOT NULL,
    last_attempt_outcome TEXT NOT NULL
        CHECK (last_attempt_outcome IN ('ruled', 'proposed', 'no_family', 'failed'))
);

-- 5. Reprise des rattachements ecrits sous l'ancien referentiel ---------------
--
-- « societe / libertes » et « environnement / energie » se repartissent
-- desormais entre plusieurs familles: leurs rattachements ne peuvent pas etre
-- transposes par correspondance. On les clot — rien n'est supprime, l'etat
-- passe reste reconstituable (RM-07) — et on rend les objets concernes a la
-- passe de rattachement.
--
-- La cloture porte sur **tous** les rattachements courants d'un objet touche,
-- pas seulement sur celui des deux familles redecoupees: un texte garde en
-- « logement » ne serait plus jamais repris, et perdrait sa part « societe »
-- pour de bon. Les rattachements des onze autres familles, sur des objets que
-- le redecoupage ne touche pas, restent ouverts tels quels.

CREATE TEMPORARY TABLE reclassified_subjects ON COMMIT DROP AS
SELECT DISTINCT subject_kind, subject_id
  FROM theme_assignments
 WHERE closed_on IS NULL
   AND family_code IN ('societe-libertes', 'environnement-energie');

UPDATE theme_assignments a
   SET closed_on = CURRENT_DATE
  FROM reclassified_subjects r
 WHERE a.closed_on IS NULL
   AND a.subject_kind = r.subject_kind
   AND a.subject_id = r.subject_id
   -- Une ligne ouverte le jour meme ne peut pas etre close la veille.
   AND a.opened_on <= CURRENT_DATE;

-- Rendus eligibles a la passe suivante: sans cela, `last_attempt_outcome`
-- valant « proposed » les tiendrait hors de la file d'attente pour toujours.
UPDATE debated_texts t
   SET last_attempt_on = NULL, last_attempt_outcome = NULL, updated_at = NOW()
  FROM reclassified_subjects r
 WHERE r.subject_kind = 'text' AND r.subject_id = t.text_key;
