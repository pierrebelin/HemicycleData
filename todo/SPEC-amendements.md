# AMENDEMENTS — Prises de position écrites des députés

> Ingérer tous les amendements de la législature en cours, les rattacher au dossier par l'identifiant de texte publié, et les rapprocher du scrutin quand l'objet du scrutin en cite le numéro. Exposer l'exposé sommaire *verbatim*, signé et daté. Objectif : donner au lecteur les raisons on-record d'un vote, sans qu'aucune phrase ne soit écrite par le site.

## 1. Contexte

Le site montre *comment* chaque groupe a voté, jamais *ce que les députés ont écrit* en votant. La promesse de vérification s'arrête au décompte.

**L'open data de l'Assemblée ne publie aucun champ « raison du vote ».** Le jeu des scrutins porte, par député, la position, la cause de non-participation, la délégation et le siège — rien d'autre. La position est un fait structuré ; le motif ne l'est jamais. Les motifs existent, mais ailleurs :

| Gisement | Contenu | État |
|---|---|---|
| Mises au point | Le député déclare que son vote enregistré ne reflète pas son intention | En base (`scrutin_vote_corrections`), non affiché |
| **Amendements** | Auteur, cosignataires, dispositif, **exposé sommaire**, sort | **Cette spec** |
| Comptes rendus de séance (Syceron) | Verbatim de séance, orateur identifié, explications de vote | Lot ultérieur, borné aux séances qu'un scrutin référence via `scrutins.sitting_ref` |
| Comptes rendus de commission | Idem, en amont de la séance | Non planifié |

Le README l'anticipe : §6 autorise « exposé des motifs, interventions en séance, position de vote » comme positions on-record, et §10 liste les amendements parmi les jeux clés — non exploité à ce jour.

L'amendement est le premier gisement traité parce qu'il est le seul **structuré** : un identifiant propre, un auteur identifié par son `PA…`, un rattachement au texte publié par la source, et un texte que son signataire a écrit et assumé. Aucune extraction, aucune interprétation.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Amendement | Modification proposée à un texte en discussion, déposée par un ou plusieurs signataires |
| Sous-amendement | Amendement portant sur un autre amendement |
| Exposé sommaire | Texte bref par lequel le signataire justifie son amendement. Rédigé par lui, publié tel quel |
| Dispositif | Le corps de la modification proposée. Non stocké : il tient sur la page officielle |
| Auteur | Premier signataire. Peut être un député, le Gouvernement ou une commission |
| Cosignataire | Signataire suivant, dans l'ordre publié |
| Sort | Résultat publié : adopté, rejeté, retiré, tombé, non soutenu, irrecevable |
| Sort inconnu | Valeur publiée hors du référentiel de la spec. Conservée avec son libellé, comptée, jamais rangée d'office |
| Texte législatif | Le document que l'amendement modifie, désigné par un identifiant publié (`PRJLANR5L17B0324`) |
| Rattachement | Lien amendement → dossier, par identifiant des deux côtés. Un fait |
| Rapprochement | Lien amendement → scrutin, déduit du numéro cité dans l'objet du scrutin. Pas un fait : une méthode |
| Groupe au dépôt | Groupe du signataire à la date de dépôt de l'amendement, jamais son groupe courant |

## 3. Cas d'usage

### CU-01 — Ingérer les amendements
**Acteur** : système · **Intention** : base à jour, exhaustive · **Fréquence** : chaque rafraîchissement

**Scénario nominal :**
1. Système récupère l'archive officielle des amendements de la législature en cours.
2. Retient chaque amendement : identité, numéro, texte visé, cible, auteur, cosignataires, exposé sommaire, sort, état, date de dépôt, amendement parent.
3. Résout le groupe de chaque signataire **à la date de dépôt** (RM-02).
4. Écrit l'ensemble, aucun amendement écarté (RM-01).
5. Recalcule intégralement les rapprochements avec les scrutins (RM-06).

**Erreurs :** source indisponible → état précédent conservé, le reste du rafraîchissement continue, anomalie signalée (RM-10). Signataire absent du référentiel → signature conservée sans groupe, aucun nom ni groupe deviné. Fichier illisible → compté, journalisé, la passe continue.

**Résultat attendu :** nombre d'amendements en base = nombre de fichiers de l'archive, moins les illisibles — eux-mêmes comptés et remontés.

### CU-02 — Consulter les amendements d'un dossier
**Acteur** : visiteur · **Intention** : lire ce que les députés ont écrit sur ce texte · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur ouvre un dossier, section « amendements ».
2. Système affiche le total, puis une page d'amendements dans l'ordre publié (RM-07).
3. Chaque ligne porte : numéro, cible, auteur avec son groupe au dépôt, nombre de cosignataires, sort avec son libellé publié, lien source (RM-09).
4. Visiteur déplie un amendement → exposé sommaire *verbatim* (RM-03), liste complète des signataires.

**Variantes :** dossier sans amendement → section présente, mention explicite. Amendements dont le texte n'est relié à aucun dossier ingéré → comptés et annoncés en note de couverture, jamais tus.

### CU-03 — Consulter l'amendement mis aux voix d'un scrutin
**Acteur** : visiteur · **Intention** : lire ce sur quoi porte le vote · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur ouvre un scrutin dont l'objet cite un numéro d'amendement.
2. Système affiche l'amendement rapproché, avec la mention de méthode (RM-06).
3. Visiteur lit l'exposé sommaire, puis la répartition des votes.

**Variantes :** l'objet ne cite aucun numéro → mention « l'objet de ce scrutin ne cite aucun numéro d'amendement ». L'objet cite un numéro qu'aucun amendement ne permet d'identifier sans ambiguïté → mention distincte, aucun lien affiché. Deux lacunes de natures différentes, dites différemment.

### CU-04 — Consulter le journal d'un député
**Acteur** : visiteur · **Intention** : voir les actes publics d'un député croisé sur un vote · **Fréquence** : occasionnelle

**Scénario nominal :**
1. Visiteur clique le nom d'un votant depuis un scrutin, ou d'un signataire depuis un amendement.
2. Système affiche l'identité, la frise des appartenances datées, et la mention de cadrage en tête de page (RM-08).
3. Système liste les actes on-record, du plus récent au plus ancien : votes nominaux, amendements signés ou cosignés, mises au point.
4. Visiteur charge la suite. Aucun total n'est affiché (RM-08).

**Variantes :** aucun acte sur la période → mention que l'absence d'acte n'est pas une information sur la personne.

> **Dépendance de charte** : CU-04 suppose l'amendement du README §3.3 acté. Tant qu'il ne l'est pas, CU-04 n'est pas implémentable.

## 4. Règles métier

### RM-01 — Exhaustivité
- **Énoncé** : tout amendement publié par la source entre en base et reste consultable. Aucun filtre sur le sort, l'auteur, le lieu d'examen ou la présence d'un texte rattachable. · **Origine** : README.md §2 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02
- **Non conforme** : n'ingérer que les amendements adoptés, ou que ceux d'un dossier connu.

### RM-02 — Groupe du signataire à la date de dépôt
- **Énoncé** : le groupe affiché à côté d'un signataire est celui qu'il détenait à la date de dépôt. Un groupe publié par la source dans l'amendement est daté par construction et conservé tel quel ; sinon il est reconstitué depuis l'appartenance datée. Sans date de dépôt, aucun groupe n'est affiché. · **Origine** : README.md §3.2 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02, CU-03
- **Non conforme** : joindre sur l'appartenance courante « faute de mieux ». Deux groupes concurrents à cette date → aucun groupe affiché, jamais un choix arbitraire.

### RM-03 — Exposé sommaire verbatim ou rien
- **Énoncé** : l'exposé sommaire est reproduit mot pour mot, attribué à son signataire. Aucun résumé, aucune reformulation, aucun extrait choisi par le site, aucun appel à un modèle. · **Origine** : README.md §6 et §8 · **Sévérité** : bloquant · **Applies to** : transverse
- Un aperçu tronqué n'est admis que s'il est strictement les N premiers caractères, N affiché, avec un lien vers le texte complet. Les 200 premiers caractères d'un exposé peuvent être une formule de politesse ou une attaque : le choix du site doit être mécanique et annoncé.
- Le rendu HTML passe par une liste blanche de balises. Aucun mot ajouté, retiré ni réordonné : seules des balises tombent.

### RM-04 — Libellés de la source conservés, sorts inconnus compris
- **Énoncé** : le sort et l'état sont affichés avec le libellé publié. Une valeur hors référentiel prend le code `other`, garde son libellé intact et est comptée au rafraîchissement. · **Origine** : README.md §6, SPEC-scrutins RM-09 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02
- **Non conforme** : ranger « Réservé jusqu'au vote » dans « rejeté » parce que c'est la catégorie la plus proche.

### RM-05 — Rattachement au dossier par identifiant seulement
- **Énoncé** : le lien amendement → dossier passe par l'identifiant de texte législatif publié, joint aux documents du dossier. Un identifiant des deux côtés, aucun rapprochement par similarité de libellé. · **Origine** : README.md §9 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02
- Un amendement dont le texte n'est relié à aucun dossier ingéré reste consultable et sa lacune est comptée.

### RM-06 — Rapprochement du scrutin : méthode stockée, abandon sur ambiguïté
- **Énoncé** : le lien amendement → scrutin est déduit du numéro cité dans l'objet du scrutin. Il porte sa méthode en base, l'affichage porte la mention correspondante, et la méthode est publiée. Un couple (scrutin, numéro) qui résout zéro ou plusieurs amendements ne produit **aucun** lien. · **Origine** : README.md §9 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03
- **Non conforme** : rapprocher par nom d'auteur cité, par proximité de date, par similarité de libellé, ou retenir « le plus probable » des deux candidats.
- Les compteurs établis / non résolus / ambigus sont publiés sur la page méthode.

### RM-07 — Pagination mécanique et annoncée
- **Énoncé** : un dossier peut porter des milliers d'amendements. L'affichage est borné par page, le total est toujours rendu, l'ordre par défaut est celui de la source, et la borne est annoncée. · **Origine** : README.md §2 · **Sévérité** : bloquant · **Applies to** : CU-02
- **Non conforme** : « les amendements les plus significatifs », un tri par nombre de cosignataires, une sélection par pertinence. Paginer n'est pas filtrer ; hiérarchiser l'est.

### RM-08 — Aucun agrégat sur une personne
- **Énoncé** : la page d'un député liste des actes datés. Aucun total par sens de vote, aucun taux de présence, aucun score, aucun classement, aucune comparaison à un autre député ni à son groupe. La pagination annonce qu'il y a d'autres actes, jamais combien. · **Origine** : README.md §3.3 amendé, §6 · **Sévérité** : bloquant · **Applies to** : CU-04
- Un décompte d'actes se lit comme une mesure d'activité parlementaire, que le site refuse de produire.
- Verrouillé par un test : la réponse de la page député ne doit porter aucune clé `total`, `count`, `rate`, `score`, `rank`, `average`, `percent`.

### RM-09 — Lien source sur chaque élément affiché
- **Énoncé** : chaque amendement affiché porte le lien vers sa page officielle. · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-03, CU-04

### RM-10 — Ingestion non bloquante
- **Énoncé** : l'échec du rafraîchissement des amendements ne fait échouer ni les dossiers, ni les scrutins, ni la thématisation. Il remonte comme anomalie et les amendements déjà en base sont conservés. · **Origine** : SPEC-scrutins RM-11, pratique établie de `RefreshAll` · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-11 — Ordre de rafraîchissement
- **Énoncé** : le référentiel des acteurs est rafraîchi avant les amendements (RM-02 en dépend), et l'extraction des textes débattus avant le rapprochement des scrutins (RM-06 en dépend). · **Origine** : ACTEURS §7, SPEC-thematisation · **Sévérité** : bloquant · **Applies to** : CU-01

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Amendement | Identifiant, législature, numéro, texte visé, lieu d'examen | Importée (AN) | Essentiel |
| Cible | Article ou division visée, libellé publié | Importée | Essentiel |
| Auteur | Député (`PA…`) ou institution (Gouvernement, commission) | Importée | Essentiel |
| Cosignataires | Acteurs, dans l'ordre publié | Importée | Essentiel |
| Groupe au dépôt | Groupe du signataire à la date de dépôt, avec son origine | Importée ou reconstituée | Essentiel |
| Exposé sommaire | Texte du signataire, verbatim | Importée | Essentiel |
| Sort | Code normalisé + libellé publié | Importée | Essentiel |
| État | Libellé de traitement publié, distinct du sort | Importée | Secondaire |
| Date de dépôt | Date publiée. Absente → aucun groupe daté calculable | Importée | Essentiel |
| Amendement parent | Amendement visé par un sous-amendement | Importée | Secondaire |
| Rattachement dossier | Par identifiant de texte, joint aux documents du dossier | Calculée (jointure) | Essentiel |
| Rapprochement scrutin | Amendement, scrutin, méthode, numéro cité | Calculée (méthode publiée) | Essentiel |
| Lien officiel | Page de l'amendement sur le site de l'Assemblée | Calculée | Essentiel |

**Non stocké, délibérément** : le dispositif de l'amendement. C'est le gros morceau en volume, il tient à un clic sur la page officielle, et le porteur de sens pour le lecteur est l'exposé sommaire.

## 6. Hypothèses à mesurer

L'archive n'est pas joignable depuis l'environnement de développement web : la politique d'egress de l'organisation refuse `data.assemblee-nationale.fr` (403 sur le CONNECT). Le parseur est donc écrit contre le schéma documenté, testé sur des fixtures écrites à la main, puis **validé contre l'archive réelle** par un test ignoré, sur le modèle de `scrutin_client.rs::parses_the_official_archive` (`SCRUTINS_ZIP`) et `actor_client.rs::parses_the_official_archive` (`AMO30_ZIP`) :

```bash
AMENDEMENTS_ZIP=/chemin/Amendements.json.zip cargo test -- --ignored
```

Chaque hypothèse ci-dessous reste **non mesurée** tant que ce test n'a pas tourné. Elles ne conditionnent pas la justesse du domaine ; elles conditionnent le mapping, le dimensionnement des lots et la fenêtre cron.

| # | À mesurer | Ce que ça décide |
|---|---|---|
| H1 | Taille de l'archive, `ETag` / `Last-Modified` | `ArchiveFetcher` garde le ZIP en mémoire **et le clone** à chaque `fetch` : au-delà de ~200 Mo, il faut une variante sur disque |
| H2 | Arborescence interne du ZIP | Diagnostic seulement — le parseur ne doit pas en dépendre |
| H3 | Nombre d'entrées, taille décompressée | Taille de base, lots, durée de passe |
| H4 | Noms exacts des champs | Le mapping `Raw*` → domaine |
| H5 | **Valeurs distinctes de sort et d'état** | Le référentiel de `FateCode` ; toute valeur non listée tombe en `other` (RM-04) |
| H6 | Un identifiant de scrutin est-il publié dans l'amendement ? | Si oui, RM-06 devient un fait publié et non une méthode |
| H7 | Un identifiant de dossier est-il publié, en plus du texte ? | Dispense ou non de la jointure par les documents (RM-05) |
| H8 | Format de l'URL publique d'un amendement | RM-09. Non implémenté tant qu'il n'est pas confirmé : un lien mort vaut moins que pas de lien |
| H9 | Longueur des exposés, nombre de cosignataires, balises HTML présentes | Volume, liste blanche de balises, troncature d'affichage |
| H10 | Le numéro est-il réutilisé entre commission et séance ? | Décide si (texte, numéro) est une clé unique — cœur de l'ambiguïté de RM-06 |

## 7. Périmètre

**Dans le périmètre** : ingestion, rattachement au dossier, rapprochement du scrutin, affichage sur les pages dossier, scrutin et député.

**Hors périmètre** : les comptes rendus de séance (lot suivant), les comptes rendus de commission, le dispositif des amendements, la recherche plein texte dans les exposés — cette dernière ferait remonter les formulations les plus tranchées, ce qui est un tri éditorial déguisé (§2, §6).

**Aucun appel à un modèle dans ce lot.** La thématisation reste le seul poste du produit qui en appelle un (README §5).
