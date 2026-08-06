# PAGES-THEME-GROUPE — Pages publiques thème × groupe × période

> Chemin de lecture par sujet : le visiteur ouvre un thème, restreint à un groupe et à une période, et lit scrutin par scrutin la répartition publiée de ce groupe. La page **liste**, elle ne **résume** pas : aucun total, aucun taux, aucun classement.

## 1. Contexte

Le site porte 8 434 scrutins et 1 270 476 positions nominales. On les atteint par date, par dossier ou par numéro — jamais par sujet. La promesse de PROJECT.md §1, « voici les textes sur le logement, voici comment chaque groupe a voté », n'a aucune page.

Le piège tient en une phrase : « le groupe X sur le logement » est un agrégat multi-scrutins par groupe, que SPEC-scrutins RM-08 et PROJECT.md §6 interdisent. La page existe quand même — comme **liste filtrée**, pas comme synthèse.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Page de thème | Liste des scrutins portant une famille thématique |
| Filtre groupe | Restriction d'une page de thème à un groupe. Ne crée pas d'objet nouveau |
| Ligne de vote | Répartition publiée d'un groupe sur **un** scrutin : pour, contre, abstention, non-votants, non-votants volontaires |
| Fenêtre d'existence | Premier et dernier scrutin où la source publie une ligne pour ce groupe |
| Chiffre dépendant du groupe | Nombre dont la valeur change selon le sens des votes du groupe |
| Chiffre indépendant du groupe | Nombre identique quel que soit le groupe consulté |
| Nature de l'objet | Ce que le scrutin met aux voix : amendement, article, ensemble du texte, motion |
| Couverture thématique | Part des textes portant au moins une famille |

## 3. Cas d'usage

### CU-01 — Parcourir un thème
**Acteur** : visiteur · **Intention** : voir ce qui a été voté sur un sujet · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur ouvre une famille.
2. Système liste ses textes, du plus récemment voté au plus ancien, avec le nombre de scrutins de chacun et l'origine du rattachement.
3. Système affiche la couverture thématique et le lien vers les non rattachés (RM-11).
4. Visiteur ouvre un texte, ou bascule sur la liste des scrutins du thème.

**Variantes :** famille sans texte rattaché → page présente, mention explicite qu'aucun texte ne porte cette famille et renvoi vers la méthode. Jamais une famille masquée (RM-07).

### CU-02 — Restreindre un thème à un groupe
**Acteur** : visiteur · **Intention** : lire les votes d'un groupe sur un sujet · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur choisit un groupe sur une page de thème.
2. Système liste les scrutins du thème, du plus récent au plus ancien.
3. Chaque ligne porte : date, objet, nature, sort publié, **et la seule ligne de vote de ce groupe** — pour, contre, abstention, non-votants, non-votants volontaires, effectif publié, position majoritaire publiée.
4. Système affiche le nombre de scrutins listés et les bornes de dates (RM-01).
5. Visiteur ouvre un scrutin pour voir tous les groupes et les positions nominales.

**Variantes :** répartition reconstituée → mention de méthode sur la ligne (RM-09).
**Erreurs :** groupe inconnu → page refusée, aucun repli sur un groupe approchant.

**Résultat attendu :** aucun nombre affiché ne cumule plusieurs scrutins.

### CU-03 — Restreindre à une période
**Acteur** : visiteur · **Intention** : cadrer la lecture dans le temps · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur choisit une session parlementaire.
2. Système restreint la liste aux scrutins de cette session, ordre inchangé.
3. Système nomme la session par ses dates, pas par sa référence source (RM-13).
4. Le compte de scrutins listés et les bornes suivent le filtre.

**Variantes :** session sans scrutin sur ce thème → liste vide annoncée, la session reste proposée.

### CU-04 — Consulter un groupe hors de sa fenêtre d'existence
**Acteur** : visiteur · **Intention** : comprendre une liste vide · **Fréquence** : occasionnelle

**Scénario nominal :**
1. Visiteur filtre sur un groupe et une période qui ne se recouvrent pas.
2. Système affiche : « la source ne publie aucune ligne pour ce groupe entre le … et le … », avec la fenêtre d'existence du groupe.
3. Système ne présente jamais l'absence comme une abstention ni comme un non-vote (RM-05).

### CU-05 — Ouvrir un thème pendant que la thématisation est incomplète
**Acteur** : visiteur · **Intention** : savoir ce que la page ne montre pas encore · **Fréquence** : transitoire

**Scénario nominal :**
1. Visiteur ouvre une famille alors que des textes n'ont pas été soumis au modèle.
2. Système affiche le nombre de textes rattachés, le nombre non rattachés, le nombre jamais soumis.
3. Système renvoie vers la liste des non rattachés et vers la méthode (RM-11).

### CU-06 — Atteindre la preuve
**Acteur** : visiteur · **Intention** : vérifier un chiffre · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur clique un scrutin depuis n'importe quelle liste.
2. Système ouvre le détail : tous les groupes, positions nominales, synthèse officielle, mises au point.
3. Système affiche le lien vers la page officielle du scrutin (RM-12).

## 4. Règles métier

### RM-01 — Critère d'admissibilité d'un chiffre
- **Énoncé** : sur une page filtrée par groupe, un nombre n'est affichable que si sa valeur ne dépend pas du sens des votes du groupe. Admis : nombre de scrutins listés, bornes de dates, effectif publié au scrutin, nombre de textes du thème. Refusé : compte de « pour », de « contre », d'abstentions ou de non-votes sur plus d'un scrutin, taux de participation, part d'abstention, compte de positions majoritaires. · **Origine** : PROJECT.md §6, SPEC-scrutins RM-08 · **Sévérité** : bloquant · **Applies to** : transverse
- **Conforme** : « 34 scrutins listés, du 12/03/2025 au 04/06/2026 » — identique pour chaque groupe.
- **Non conforme** : « le groupe a voté pour 21 fois sur ce thème », « 78 % de participation ».

### RM-02 — Aucun cumul multi-scrutins par groupe
- **Énoncé** : les chiffres de vote restent attachés à un scrutin. Aucune somme, moyenne, taux ni classement portant sur plusieurs scrutins d'un groupe. · **Origine** : SPEC-scrutins RM-08, SPEC-thematisation RM-12 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-03

### RM-03 — Deux groupes côte à côte seulement au scrutin
- **Énoncé** : la mise en regard de plusieurs groupes n'existe qu'au détail d'un scrutin, où la source publie leurs lignes ensemble. Aucune page ne juxtapose deux groupes sur un thème ou une période. · **Origine** : PROJECT.md §6, §9 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-06

### RM-04 — Groupe de la ligne = groupe publié au scrutin
- **Énoncé** : le filtre groupe porte sur le groupe sous lequel la source range le vote **dans ce scrutin**, jamais sur l'appartenance courante d'un député. · **Origine** : PROJECT.md §3.2, SPEC-scrutins RM-04 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-03

### RM-05 — Fenêtre d'existence affichée
- **Énoncé** : une liste vide par absence du groupe sur la période porte la mention de sa fenêtre d'existence. L'absence n'est jamais présentée comme une position de vote. · **Origine** : PROJECT.md §2, mesure du 05/08/2026 (H3) · **Sévérité** : bloquant · **Applies to** : CU-04

### RM-06 — Le thème vient du texte
- **Énoncé** : un scrutin entre dans un thème par les familles de son texte débattu. Aucun rattachement direct sur un scrutin, aucun sur un amendement isolé. · **Origine** : SPEC-thematisation RM-06 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02

### RM-07 — Le tri ordonne, il ne filtre pas
- **Énoncé** : tout scrutin d'un texte rattaché est atteignable depuis la page du thème. Les filtres de nature et de période sont explicites, réversibles et affichés ; aucun retrait implicite. · **Origine** : PROJECT.md §2 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02, CU-03
- **Non conforme** : n'afficher que les scrutins « l'ensemble du texte » parce que les amendements sont bruyants.

### RM-08 — Chiffres de la source, jamais recalculés
- **Énoncé** : répartition, effectif, position majoritaire et sort sont affichés tels que publiés. Aucun total recalculé depuis les positions nominales. · **Origine** : SPEC-scrutins RM-02 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-06

### RM-09 — Répartition reconstituée signalée
- **Énoncé** : une ligne reconstruite depuis le décompte nominatif porte sa mention de méthode partout où elle s'affiche, liste comprise. · **Origine** : SPEC-scrutins RM-03 · **Sévérité** : bloquant · **Applies to** : CU-02

### RM-10 — Position majoritaire sans lecture d'intention
- **Énoncé** : la position majoritaire publiée s'affiche comme un fait de la source. Jamais présentée comme consigne, discipline ou engagement du groupe. · **Origine** : SPEC-scrutins §6, PROJECT.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02

### RM-11 — Couverture thématique affichée
- **Énoncé** : chaque page de thème porte le nombre de textes rattachés, non rattachés et jamais soumis, et le lien vers les non rattachés. · **Origine** : PROJECT.md §2, SPEC-thematisation RM-01 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-05

### RM-12 — Lien source sur chaque scrutin listé
- **Énoncé** : chaque scrutin affiché porte le lien vers sa page officielle, depuis la liste comme depuis le détail. · **Origine** : PROJECT.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-06

### RM-13 — Période nommée par ses dates
- **Énoncé** : une session est présentée par ses dates de premier et dernier scrutin. La référence de la source est conservée dans l'adresse, jamais seule à l'écran. · **Origine** : choix produit · **Sévérité** : warning · **Applies to** : CU-03

### RM-14 — Lacune des votes à main levée affichée
- **Énoncé** : les pages de thème portent la mention que les votes à main levée ne figurent pas dans la source. · **Origine** : PROJECT.md §7, SPEC-scrutins RM-06 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02

### RM-15 — Adresses stables et partageables
- **Énoncé** : thème, thème × groupe, thème × groupe × période ont chacun une adresse stable, partageable et indexable. · **Origine** : PROJECT.md §8.1 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02, CU-03

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Famille et sa couverture | Libellé, périmètre, comptes de textes rattachés / non rattachés / jamais soumis | Calculée | Essentiel |
| Texte du thème | Libellé, nombre de scrutins, dates de premier et dernier vote, origine du rattachement | Calculée | Essentiel |
| Scrutin listé | Date, numéro, objet, nature, type de vote, sort publié, lien officiel | Importée | Essentiel |
| Ligne de vote du groupe | Pour, contre, abstention, non-votants, non-votants volontaires, effectif, position majoritaire, origine publiée ou reconstituée | Importée ou reconstruite | Essentiel |
| Groupe | Libellé officiel, sigle, fenêtre d'existence | Importée et calculée | Essentiel |
| Session | Référence source, date de premier et de dernier scrutin, nombre de scrutins | Importée et calculée | Essentiel |
| Compte de scrutins listés | Nombre de lignes après filtres, bornes de dates | Calculée | Essentiel |

## 6. Comportements transverses

**Volumétrie** — un thème peut porter plus de mille scrutins : 17 textes concentrent 5 699 des 8 434 scrutins. Les listes sont paginées, l'ordre est stable, et le compte total est affiché avant pagination : une page tronquée sans son total laisserait croire à une sélection (PROJECT.md §2).

**Ordre** — chronologique décroissant partout, par défaut. Aucun tri par « importance » : il ordonnerait ce que le visiteur croirait filtré.

**Nature de l'objet** — affichée sur chaque ligne (amendement, article, ensemble du texte, motion). 86 % des scrutins sont des amendements ; sans cette mention, un vote sur un amendement de procédure se lit comme une position sur le texte.

**Groupe sans votant sur un scrutin** — la source publie parfois une ligne à zéro. Affichée telle quelle, jamais convertie en absence de ligne.

## 7. Relations

| Amont | Aval |
|---|---|
| Rattachements et propositions (spec THEMATISATION) | Contenu des pages de thème |
| Textes débattus et liens scrutin → texte | Appartenance d'un scrutin à un thème |
| Scrutins, répartitions par groupe (spec SCRUTINS) | Lignes de vote listées |
| Référentiel des groupes (spec ACTEURS) | Libellés, sigles, fenêtres d'existence |
| Pages livrées ici | Chat de routage (Phase 6), page méthodologie (Phase 7) |

## 8. Hors périmètre

| Exclusion | Raison |
|---|---|
| Tout total, taux ou classement par groupe | RM-01, RM-02 |
| Comparaison de deux groupes hors détail d'un scrutin | RM-03 |
| Fiche par député, historique de vote d'une personne | PROJECT.md §3.3, SPEC-scrutins §8. PROJECT.md §8.1 annonce pourtant une adresse « député » — contradiction, voir Q4 |
| Traduction d'un groupe en parti | PROJECT.md §3.1 |
| Rattachement des ~2 758 dossiers sans scrutin | SPEC-thematisation §9, mécanique existante non lancée |
| Export, graphiques, séries temporelles | Une courbe de votes par groupe est un cumul déguisé (RM-02) |
| Législatures antérieures | SPEC-acteurs RM-07 |

## 9. Hypothèses

Mesurées sur la base réelle le **5 août 2026**, législature 17, sauf mention contraire.

| # | Hypothèse | Statut |
|---|---|---|
| H1 | Chaque scrutin porte une ligne pour chaque groupe | **Infirmée, marginale.** 12 lignes sur 8 424 scrutins, 11 sur 8, 10 sur 2. GDR manque sur 2 scrutins, NI sur 10. Une page de groupe saute donc quelques scrutins du thème sans que ce soit une position |
| H2 | Les groupes de la législature sont connus et stables | **Confirmée en nombre.** 13 groupes dans les répartitions, 101 196 lignes, zéro groupe hors référentiel. 1 270 476 positions nominales, **zéro sans groupe** |
| H3 | Les groupes coexistent sur toute la législature | **Infirmée. Fait structurant.** UDR : 3 053 scrutins, 08/10/2024 → 10/07/2025. UDDPLR : 5 381 scrutins, 08/09/2025 → 21/07/2026. Zéro recouvrement, et 3 053 + 5 381 = 8 434 : ils partitionnent exactement la législature. Aucun cumul « sur la législature » ne peut avoir de dénominateur commun entre ces deux groupes (fonde RM-01, RM-05) |
| H4 | L'effectif d'un groupe est stable, donc utilisable comme dénominateur | **Infirmée.** RN 121 à 125, EPR 91 à 95, NI 8 à 12, DR 47 à 50. Seuls ECOS, GDR et UDR sont constants. Aucun taux n'a de base fixe |
| H5 | Une ligne de groupe porte toujours des votants | **Infirmée.** 8 834 lignes à zéro sur les cinq positions. Un cumul les avalerait en silence |
| H6 | Les objets soumis au vote sont majoritairement des textes entiers | **Infirmée, fait structurant.** Amendement 7 222, article 917, **ensemble du texte 211**, motion 56, autre 28. Un total par thème serait dominé par des votes d'amendement, pas par des positions sur un texte (fonde RM-01, comportement « nature de l'objet ») |
| H7 | Les répartitions affichées sont publiées par la source | **Confirmée à 99,87 %.** 101 062 lignes publiées, 134 reconstituées. Les 134 sont exactement les lignes sans position majoritaire publiée (fonde RM-09) |
| H8 | La session est un découpage utilisable comme période | **Confirmée, déséquilibrée.** 5 sessions, zéro scrutin sans session, 21 mois couverts : SCR5A2025O1 2 873 (08/10/2024 → 30/06/2025), SCR5A2025E1 180 (01→10/07/2025), SCR5A2025E2 **1** (08/09/2025), SCR5A2026O1 4 849 (15/10/2025 → 30/06/2026), SCR5A2026E1 531 (01→21/07/2026) |
| H9 | Un thème tient dans une page | **Infirmée.** 322 textes, médiane 2 scrutins, moyenne 26,2, maximum 931. 17 textes ≥ 100 scrutins portent 5 699 scrutins (68 %). Pagination obligatoire (comportement « volumétrie ») |
| H10 | Le groupe d'un vote ne se déduit pas de l'appartenance courante | **Confirmée.** 23 acteurs sur 645 ont voté sous plus d'un groupe. Joindre sur l'appartenance courante réécrirait leurs votes (fonde RM-04) |
| H11 | La thématisation alimente ces pages | **Non vérifiable au 05/08/2026.** 322 textes, **0 rattachement courant, 0 proposition, 0 texte soumis au modèle** : `ANTHROPIC_API_KEY` absente de l'environnement. 8 428 scrutins sur 8 434 portent un texte, le porteur existe donc ; seul le rattachement manque. Toute mesure de volumétrie **par famille** reste à faire (Q1) |

Mesures reproductibles : `cargo run --bin measure_phase5`.

## 10. Questions ouvertes

| # | Question | Impact | Options |
|---|---|---|---|
| Q1 | La volumétrie par famille n'est pas mesurée : couverture nulle faute de clé d'API. Une famille pourrait porter 4 000 scrutins comme 12. | Dimensionnement des listes, pertinence du filtre de nature | Mesurer après la première passe de thématisation (retenu) · dimensionner sur le pire cas mesuré aujourd'hui (931 scrutins pour un texte) |
| Q2 | La session SCR5A2025E2 porte 1 scrutin. Proposée comme les autres, elle donnera presque toujours une liste vide. | Un filtre qui ne rend rien 8 fois sur 8 se lit comme un défaut | Proposer toutes les sessions (retenu) · fondre les sessions extraordinaires dans l'ordinaire voisine · masquer les sessions vides pour le thème consulté |
| Q3 | La justification produite par le modèle est-elle affichée au visiteur ? Reprend SPEC-thematisation Q3, que ces pages rendent concrète. | Transparence contre bruit sur la page de thème | Sur la fiche du texte seulement · sur chaque ligne de la liste · réservée à l'arbitrage |
| Q4 | PROJECT.md §8.1 annonce une adresse « député » ; §3.3 et SPEC-scrutins §8 excluent la fiche par personne. | Contradiction non tranchée dans les sources | Trancher pour l'exclusion et corriger §8.1 · ouvrir une page député sans historique agrégé · laisser en l'état |
| Q5 | Le filtre de nature (amendement / article / ensemble du texte / motion) n'existe dans aucune spec livrée : il est introduit ici. | Dérive possible vers un filtre par défaut, que RM-07 interdit | Filtre explicite, jamais actif par défaut (retenu) · nature affichée sans filtre · TBD |
| Q6 | Une proposition non arbitrée alimente ces pages sans limite de temps. Reprend SPEC-thematisation Q4. | Le visiteur lit un rattachement que personne n'a validé | Mention d'origine sur chaque ligne (déjà RM-09 de THEMATISATION) · compteur d'attente en méthode · priorisation par nombre de scrutins portés |

→ Étape suivante : /plan-implementation
