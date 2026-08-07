# SCRUTINS — Votes publics de l'Assemblée nationale

> Ingérer tous les scrutins publics de la législature en cours, leur répartition par groupe et la position de chaque député. Exposer liste, détail et section scrutins d'un dossier. Niveau de preuve du site : chaque chiffre remonte à sa source officielle.

## 1. Contexte

Le site montre aujourd'hui des dossiers sans leurs votes. La promesse — « voici comment chaque groupe a voté » — n'est pas tenue.

La source officielle publie 8 434 scrutins pour la législature 17, tous avec décompte nominatif. 69 % ne portent aucun dossier : les écarter viderait le site des deux tiers des votes (README.md §7).

Le référentiel des acteurs (spec ACTEURS, livrée) fournit noms et libellés de groupe. Sans lui, un scrutin n'affiche que des identifiants.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Scrutin public | Vote enregistré, position de chaque votant connue |
| Vote à main levée | Vote non enregistré. Absent de la source : le site n'en sait rien |
| Position de vote | Pour, contre, abstention, non-votant |
| Non-votant | Votant présent au scrutin qui ne prend pas part au vote pour une cause publiée (fonction) |
| Non-votant volontaire | Compte publié par groupe, sans nom. La synthèse officielle ne le totalise pas |
| Répartition par groupe | Décompte pour / contre / abstention / non-votants / non-votants volontaires, groupe par groupe |
| Position nominale | Position d'un votant identifié, avec le groupe sous lequel la source le range |
| Synthèse | Décompte global publié par l'Assemblée : votants, suffrages exprimés, suffrages requis, annonce du sort |
| Mise au point | Déclaration postérieure d'un député : son vote enregistré ne correspond pas à son intention |
| Sort | Résultat publié : adopté / rejeté |
| Scrutin sans dossier | Scrutin que la source ne rattache à aucun dossier législatif |
| Répartition reconstituée | Répartition recalculée depuis les positions nominales quand la source ne publie pas les groupes |

## 3. Cas d'usage

### CU-01 — Ingérer les scrutins
**Acteur** : système · **Intention** : base à jour, exhaustive · **Fréquence** : chaque rafraîchissement

**Scénario nominal :**
1. Système récupère l'archive officielle des scrutins de la législature en cours.
2. Retient chaque scrutin : identité, date, séance, type, sort, objet, synthèse, répartition par groupe, positions nominales, mises au point.
3. Rattache le scrutin au dossier quand la source en désigne un (RM-10).
4. Reconstruit la répartition des groupes que la source ne nomme pas (RM-03).
5. Écrit l'ensemble, aucun scrutin écarté (RM-01).

**Erreurs :** source indisponible → état précédent conservé, rafraîchissement des dossiers continue, anomalie signalée. Acteur absent du référentiel → position conservée avec l'identifiant brut, aucun nom deviné (ACTEURS RM-04).

**Résultat attendu :** nombre de scrutins en base = nombre publié par la source.

### CU-02 — Consulter la liste des scrutins
**Acteur** : visiteur · **Intention** : trouver un vote · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur ouvre la liste, triée du plus récent au plus ancien.
2. Chaque ligne porte : date, numéro, objet, sort, décompte pour / contre / abstention.
3. Visiteur filtre par période, sort, type de vote, présence d'un dossier.
4. Visiteur ouvre un scrutin.

**Variantes :** filtre sans résultat → liste vide annoncée, aucun repli sur un résultat approchant.

### CU-03 — Consulter un scrutin
**Acteur** : visiteur · **Intention** : savoir qui a voté quoi · **Fréquence** : courante

**Scénario nominal :**
1. Visiteur ouvre un scrutin : date, séance, type de vote, majorité requise, demandeur, objet, sort.
2. Système affiche la synthèse officielle : votants, suffrages exprimés, suffrages requis, décompte.
3. Système affiche la répartition par groupe, un groupe par ligne, libellé officiel (ACTEURS RM-06).
4. Visiteur déplie un groupe → positions nominales, nom et position de chaque votant.
5. Système affiche les mises au point, à part, sans toucher aux décomptes (RM-05).
6. Système affiche le lien vers la page officielle du scrutin (RM-07) et vers le dossier s'il existe.

**Variantes :** répartition reconstituée → mention de méthode affichée (RM-03). Scrutin sans dossier → aucun lien dossier, aucune mention d'anomalie : c'est le cas majoritaire.

### CU-04 — Consulter les scrutins d'un dossier
**Acteur** : visiteur · **Intention** : voir les votes d'un texte · **Fréquence** : chaque consultation de dossier

**Scénario nominal :**
1. Visiteur ouvre un dossier.
2. Système liste les scrutins que la source rattache à ce dossier : date, objet, sort, décompte.
3. Système affiche la lacune des votes à main levée (RM-06).
4. Visiteur ouvre un scrutin.

**Variantes :** dossier sans scrutin rattaché → section présente, mention explicite que la source ne rattache aucun scrutin à ce dossier, jamais une section absente.

## 4. Règles métier

### RM-01 — Exhaustivité
- **Énoncé** : tout scrutin publié par la source entre en base et reste consultable. Aucun filtre sur le sort, le type, le lieu ou la présence d'un dossier. · **Origine** : README.md §2 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02

### RM-02 — Chiffres de la source, jamais recalculés
- **Énoncé** : synthèse et répartition par groupe sont affichées telles que publiées. Aucun total recalculé, arrondi ou corrigé, même quand la source se contredit. · **Origine** : README.md §6-7 · **Sévérité** : bloquant · **Applies to** : transverse
- **Non conforme** : remplacer un décompte publié par la somme des positions nominales.

### RM-03 — Répartition reconstituée signalée
- **Énoncé** : quand la source ne nomme pas les groupes d'un scrutin, la répartition est reconstruite depuis les positions nominales et le référentiel, et porte la mention « répartition reconstituée à partir du décompte nominatif — la source ne publie pas les groupes sur ce scrutin ». · **Origine** : ACTEURS Q4, README.md §2 et §9 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03
- **Conforme** : chiffre affiché avec mention. **Non conforme** : chiffre reconstruit présenté comme publié par l'Assemblée.

### RM-04 — Groupe = ligne publiée du scrutin
- **Énoncé** : le groupe d'une position nominale est celui sous lequel la source range le votant dans ce scrutin. Jamais l'appartenance courante. · **Origine** : README.md §3.2, choix produit · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03
- La ligne de groupe est datée par construction : elle appartient au scrutin. L'appartenance datée ne sert qu'à la reconstruction (RM-03).

### RM-05 — Mise au point sans effet sur les décomptes
- **Énoncé** : une mise au point est affichée à part, attribuée et datée. Elle ne modifie ni la synthèse, ni la répartition, ni les positions nominales. · **Origine** : convention Assemblée nationale · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03

### RM-06 — Lacune des votes à main levée affichée
- **Énoncé** : les pages de scrutins portent la mention que les votes à main levée ne figurent pas dans la source et que le site n'en rend pas compte. · **Origine** : README.md §2 et §7 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-04

### RM-07 — Lien source sur chaque scrutin
- **Énoncé** : chaque scrutin affiché porte le lien vers sa page officielle. · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-03, CU-04

### RM-08 — Aucun agrégat comparatif entre groupes
- **Énoncé** : aucun cumul, taux ou classement qui compare les groupes entre eux. Les chiffres restent attachés à un scrutin. · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : transverse
- **Non conforme** : « taux de participation du groupe X sur la législature », « cohérence de vote ».

### RM-09 — Libellés de la source conservés
- **Énoncé** : sort, type de vote, majorité requise, annonce, cause de non-vote sont affichés avec le libellé publié. Aucune reformulation, aucun code traduit sans libellé officiel. · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-03

### RM-10 — Scrutin sans dossier conservé
- **Énoncé** : l'absence de dossier ne retire ni n'altère un scrutin. Le rattachement est enregistré quand la source le publie, ignoré sinon. · **Origine** : README.md §7 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03

### RM-11 — Ordre de rafraîchissement
- **Énoncé** : référentiel des acteurs rafraîchi avant les scrutins. Sinon la reconstruction (RM-03) et les noms affichés portent sur des données périmées. · **Origine** : ACTEURS §7 · **Sévérité** : bloquant · **Applies to** : CU-01

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Scrutin | Identifiant, numéro, législature, date, session, séance, lieu | Importée (AN) | Essentiel |
| Type de vote | Code, libellé, majorité requise | Importée | Essentiel |
| Sort | Code, libellé | Importée | Essentiel |
| Objet | Libellé du texte soumis au vote | Importée | Essentiel |
| Demandeur | Qui a demandé le scrutin | Importée | Secondaire |
| Synthèse | Votants, suffrages exprimés, suffrages requis, annonce, décompte | Importée | Essentiel |
| Répartition par groupe | Groupe, effectif, position majoritaire, décompte | Importée ou reconstruite | Essentiel |
| Origine de la répartition | Publiée / reconstituée | Calculée | Essentiel |
| Position nominale | Acteur, groupe, position, cause de non-vote, vote par délégation, place | Importée | Essentiel |
| Mise au point | Acteur, position revendiquée, signalée comme dysfonctionnement ou non | Importée | Secondaire |
| Dossier rattaché | Identifiant et libellé du dossier, quand la source en désigne un | Importée | Essentiel |
| Lien officiel | Page du scrutin sur le site de l'Assemblée | Calculée | Essentiel |

## 6. Comportements transverses

**Position majoritaire d'un groupe** — publiée par la source, affichée telle quelle. Jamais recalculée, jamais présentée comme une consigne ni comme un engagement du groupe.

**Effectif du groupe** — publié scrutin par scrutin. Il donne la portée du décompte, il ne sert à aucun taux (RM-08).

**Cause de non-vote** — publiée sous forme de code. Affichée telle quelle tant que le libellé officiel n'est pas sourcé (Q2).

## 7. Relations

| Amont | Aval |
|---|---|
| Archive officielle des scrutins AN | Scrutins, répartitions, positions nominales |
| Référentiel acteurs et groupes (spec ACTEURS) | Noms et libellés affichés, reconstruction RM-03 |
| Scrutins | Section scrutins d'un dossier |
| Scrutins | Pages thème × groupe × période (README.md §8.1, Phase 5) |

## 8. Hors périmètre

| Exclusion | Raison |
|---|---|
| Votes à main levée | Absents de la source (RM-06) |
| Votes en commission | Jeu de données distinct |
| Scrutins du Sénat | Sénat hors périmètre (README.md §10) |
| Législatures antérieures | Choix produit (ACTEURS RM-07) |
| Fiche par député, historique de vote d'une personne | README.md §3.3 : le site présente les votes d'un groupe, pas la position d'une personne |
| Agrégats multi-scrutins par groupe | RM-08 |
| Thématisation des scrutins | Phase 4 |

## 9. Hypothèses

Vérifiées sur les données réelles le 3 août 2026 (archive officielle des scrutins, législature 17).

| # | Hypothèse | Statut |
|---|---|---|
| H1 | Tous les scrutins portent le décompte nominatif | **Confirmée.** 8 434 / 8 434, mode de publication unique |
| H2 | La position de vote de chaque votant permet de reconstruire les groupes | **Confirmée.** 1 270 476 positions, 645 acteurs, zéro identifiant non résolu par le référentiel |
| H3 | La référence de mandat du votant donne son groupe sans recherche par date | **Infirmée.** Les 1 270 476 références pointent vers le mandat d'Assemblée — le siège, pas le groupe. Le raccourci annoncé dans ACTEURS H2 n'existe pas ; la ligne de groupe du scrutin le remplace (RM-04) |
| H4 | La ligne de groupe publiée coïncide avec l'appartenance datée | **Confirmée à 99,97 %.** 371 lignes divergent sur 1 270 476, 25 acteurs, 175 scrutins, toutes des bornes à un jour (l'Assemblée clôt les appartenances la veille d'un renouvellement mais rattache le vote au groupe). Écart journalisé, jamais affiché |
| H5 | La sentinelle de groupe factice se reconstruit sans perte | **Confirmée.** 146 lignes sur 14 scrutins, dont 12 intégralement perdues. Reconstruction depuis le référentiel : totaux identiques au déclaré 14 fois sur 14, zéro acteur non résolu. Sur les 2 scrutins partiels, les 11 lignes publiées coïncident ligne à ligne |
| H6 | Le rattachement à un dossier est publié dans le scrutin | **Confirmée, partielle.** 2 608 scrutins sur 8 434 (31 %) portent un dossier, 75 dossiers distincts, jusqu'à 422 scrutins pour un même dossier. Les 5 826 autres n'en portent aucun (RM-10) |
| H7 | Le sort et le type de vote ont un jeu de valeurs fermé | **Confirmée.** Sort : adopté (2 849), rejeté (5 585). Type : scrutin public ordinaire (8 339), solennel (72), motion de censure (23). Lieu : Hémicycle (8 409), Salons (25) |
| H8 | Tous les scrutins portent la même assemblée | **Confirmée.** 8 434 / 8 434 sur le même organe, législature 17, dates du 08/10/2024 au 21/07/2026 |
| H9 | La mise au point ne modifie pas le décompte officiel | **Confirmée.** 3 043 déclarations sur 1 442 scrutins, publiées hors décompte. La source publie en plus 163 dysfonctionnements du matériel de vote sur 145 scrutins, de même nature. Total 3 206 déclarations sur 1 544 scrutins ; aucun acteur ne figure dans les deux blocs d'un même scrutin |
| H10 | La synthèse officielle égale la somme des groupes | **Confirmée pour pour / contre / abstention** : 8 434 / 8 434. **Infirmée ailleurs** : la synthèse totalise toujours 0 non-votant volontaire alors que les groupes en comptent (6 204 scrutins, jusqu'à 257). Un scrutin diverge aussi sur les non-votants (le premier de la législature : 10 en synthèse, 21 en groupes). Les deux chiffres sont affichés tels que publiés (RM-02) |
| H11 | La répartition compte toujours les mêmes groupes | **Confirmée.** 12 lignes de groupe par scrutin, sans exception. 13 groupes distincts sur la législature, plus la sentinelle |

## 10. Questions ouvertes

| # | Question | Impact | Options |
|---|---|---|---|
| Q1 | Le premier scrutin de la législature publie 10 non-votants en synthèse et 21 dans les groupes. Anomalie de source isolée. | 1 scrutin sur 8 434. Affiché tel quel (RM-02), mais rien ne le signale au visiteur | Laisser tel quel · signaler l'écart sur ce scrutin · journal des corrections (Phase 7) |
| Q2 | Les causes de non-vote sont publiées en codes (3 valeurs, 23 383 positions). Aucun libellé officiel dans le jeu de données. | Le visiteur lit un code brut | Sourcer les libellés dans un autre jeu AN · afficher le code seul · publier la table sur la page méthodologie |
| Q3 | ~~Le rattachement d'un scrutin à un dossier ne vaut que si le dossier est ingéré. 75 dossiers sont référencés ; la couverture réelle n'est pas mesurée.~~ **Répondu le 03/08/2026** : les 75 dossiers référencés sont tous ingérés, **zéro lien mort** sur les 2 608 scrutins concernés. Aucune action. | — | — |
| Q4 | Les non-votants volontaires sont comptés sans être nommés, et absents de la synthèse officielle. Leur définition n'est pas publiée. | Chiffre affiché sans définition, sur 6 204 scrutins | Afficher le libellé source seul (retenu) · sourcer la définition et la publier en méthodologie |

→ Étape suivante : /plan-implementation
