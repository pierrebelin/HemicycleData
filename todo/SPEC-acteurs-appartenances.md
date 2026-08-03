# ACTEURS — Référentiel des députés et groupes

> Constituer référentiel officiel des députés et groupes parlementaires de la législature en cours, avec appartenances datées. Rattacher chaque initiateur de dossier au groupe qu'il avait à la date de dépôt. Socle des noms et des groupes affichés dans les scrutins.

## 1. Contexte

Site affiche aujourd'hui le groupe **courant** d'un initiateur, lu chez un tiers. Député change de groupe : ses dépôts passés changent d'étiquette rétroactivement. Faute vérifiable par n'importe qui, sur un site dont la valeur est la vérifiabilité.

Scrutins à venir désignent députés et groupes par identifiants. Sans référentiel, aucun nom ni libellé affichable.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Acteur | Personne enregistrée par l'Assemblée : député, ministre, sénateur |
| Député | Acteur titulaire d'un mandat à l'Assemblée |
| Groupe parlementaire | Formation constituée de députés ; effectif = membres + apparentés |
| Apparenté | Député rattaché à un groupe sans en être membre |
| Non-inscrit | Député sans groupe |
| Appartenance | Lien député–groupe, borné par date de début et date de fin |
| Législature en cours | Mandature actuelle de l'Assemblée |
| Initiateur | Acteur à l'origine du dépôt d'un dossier |
| Date de référence | Date de l'acte servant à choisir l'appartenance à afficher |

## 3. Cas d'usage

### CU-01 — Ingérer le référentiel
**Acteur** : système · **Intention** : référentiel à jour · **Fréquence** : chaque rafraîchissement

**Scénario nominal :**
1. Système récupère acteurs, mandats et organes depuis le jeu historique officiel (RM-05).
2. Retient députés, groupes, appartenances avec leurs dates et leur qualité.
3. Filtre sur la législature en cours (RM-07).
4. Met à jour référentiel, conserve appartenances closes.

**Erreurs :** source indisponible → référentiel précédent conservé, rafraîchissement des dossiers continue, anomalie signalée.

### CU-02 — Rattacher un initiateur à son groupe de dépôt
**Acteur** : système · **Intention** : étiquette fidèle à la date de l'acte · **Fréquence** : chaque ingestion de dossier

**Scénario nominal :**
1. Système lit initiateurs du dossier et date de dépôt.
2. Cherche pour chaque initiateur l'appartenance active à cette date.
3. Enregistre le groupe trouvé avec la date de référence.

**Variantes :** aucune appartenance active à cette date → non-inscrit. Initiateur non député (ministre) → aucun groupe, qualité affichée.
**Erreurs :** acteur absent du référentiel → nom conservé, groupe non affiché (RM-04).

### CU-03 — Consulter les initiateurs d'un dossier
**Acteur** : visiteur · **Intention** : savoir qui a déposé le texte, sous quelle étiquette, à quelle date · **Fréquence** : chaque consultation

**Scénario nominal :**
1. Visiteur ouvre un dossier.
2. Système affiche pour chaque initiateur : nom, qualité, groupe à la date de dépôt.
3. Système affiche la date de référence à côté du groupe.
4. Système affiche lien vers page officielle de l'acteur.

**Variantes :** non-inscrit → mention « non-inscrit ». Ministre → fonction, aucun groupe.

**Résultat attendu :** aucun groupe affiché sans sa date de référence.

## 4. Règles métier

### RM-01 — Appartenance à la date de l'acte
- **Énoncé** : groupe affiché à côté d'un acte = groupe du député à la date de cet acte. Jamais le groupe courant. · **Origine** : PROJECT.md §3.2 · **Sévérité** : bloquant
- **Applies to** : transverse
- **Conforme** : « déposé en 2024 — groupe X (au 12/09/2024) ». **Non conforme** : « groupe X » sans date, lu sur l'appartenance actuelle.

### RM-02 — Toutes les qualités d'appartenance comptent
- **Énoncé** : une appartenance à un groupe compte quelle que soit sa qualité — membre, membre apparenté, président. Aucune qualité n'est écartée du rattachement. · **Origine** : convention Assemblée nationale · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02
- **Non conforme** : ne retenir que la qualité « membre » — les présidents de groupe disparaîtraient de leur propre groupe.

### RM-03 — Non-inscrits traités comme un groupe
- **Énoncé** : les non-inscrits forment un groupe à part entière, nommé par son libellé officiel. Jamais masqués, jamais rattachés d'office à un autre groupe. · **Origine** : PROJECT.md §2 (exhaustivité) · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03

### RM-04 — Aucune identité déduite
- **Énoncé** : acteur absent du référentiel → nom brut conservé, groupe non affiché. Aucun groupe deviné par homonymie ou proximité. · **Origine** : PROJECT.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02, CU-03

### RM-05 — Source officielle historique
- **Énoncé** : référentiel provient du jeu officiel **historique** de l'Assemblée nationale (tous acteurs, tous mandats), pas du jeu limité aux députés en exercice. Aucune source tierce. · **Origine** : PROJECT.md §9 (traçabilité) + vérification données · **Sévérité** : bloquant · **Applies to** : CU-01
- **Justification mesurée** : 645 députés ont voté sous la législature en cours pour 577 sièges. Le jeu « en exercice » laisse 69 votants et 97 mandats non résolus ; le jeu historique en laisse zéro.

### RM-06 — Groupe nommé tel quel
- **Énoncé** : libellé affiché = libellé officiel du groupe. Aucune traduction en parti politique. · **Origine** : PROJECT.md §3.1 · **Sévérité** : bloquant · **Applies to** : CU-03

### RM-07 — Législature en cours
- **Énoncé** : seuls acteurs et appartenances de la législature en cours entrent dans le référentiel. · **Origine** : choix produit · **Sévérité** : bloquant · **Applies to** : CU-01

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Acteur | Identifiant, nom, prénom | Importée (AN) | Essentiel |
| Qualité | Député, ministre, autre | Importée | Essentiel |
| Groupe | Identifiant, sigle, libellé officiel | Importée | Essentiel |
| Appartenance | Député, groupe, date début, date fin, qualité (membre, membre apparenté, président, non-inscrit) | Importée | Essentiel |
| Législature | Mandature de rattachement | Importée | Essentiel |
| Lien page officielle acteur | Source vérifiable | Importée | Secondaire |

## 6. États & transitions

| État | Événement | État suivant | Condition |
|---|---|---|---|
| — | Adhésion à un groupe | Active | Date de début renseignée |
| Active | Départ du groupe | Close | Date de fin renseignée |

Appartenance close reste consultable : elle porte les actes de sa période.

## 7. Comportements transverses

**Ordre de rafraîchissement** — référentiel rafraîchi avant les dossiers. Sinon rattachements calculés sur données périmées.

**Appartenance sans date de fin** — considérée active.

## 8. Relations

| Amont | Aval |
|---|---|
| Jeu officiel acteurs et organes AN | Référentiel députés et groupes |
| Référentiel | Initiateurs datés des dossiers |
| Référentiel | Noms et libellés affichés dans les scrutins (spec SCRUTINS) |
| Référentiel | Pages par groupe (PROJECT.md §8.1) |

## 9. Hors périmètre

| Exclusion | Raison |
|---|---|
| Fiche par député | Choix produit : position nominale visible au détail du scrutin seulement |
| Composition historique complète des groupes | Besoin non établi |
| Législatures antérieures | Choix produit (RM-07) |
| Sénateurs | Sénat hors périmètre (PROJECT.md §10) |
| Mandats locaux, fonctions exécutives | Sans usage produit |
| Correspondance groupe → parti | Interdite (RM-06) |

## 10. Hypothèses

Vérifiées sur les données réelles le 2 août 2026 (jeux officiels AN, législature 17).

| # | Hypothèse | Statut |
|---|---|---|
| H1 | Appartenances portent dates de début et de fin, et une qualité | **Confirmée.** Qualités observées : membre, membre apparenté, président, député non-inscrit |
| H2 | Scrutins désignent acteurs et groupes par les identifiants du référentiel | **Confirmée au-delà.** Chaque votant porte aussi la référence de son mandat : le groupe se lit sans recherche par date. Zéro référence non résolue sur l'ensemble des scrutins |
| H3 | Les non-inscrits sont une absence d'appartenance | **Infirmée.** Les non-inscrits constituent un groupe officiel à part entière (voir RM-03) |
| H4 | Date de dépôt présente à la source mais non conservée aujourd'hui : elle devra l'être pour appliquer RM-01 | **Confirmée.** Gap côté application — comblé le 3 août 2026 : plus ancien acte `*-DEPOT`, repli sur la date de dépôt du plus ancien document. Couverture 2 810 / 2 833 dossiers de la législature 17 ; les deux sources concordent sur les 2 559 cas où elles coexistent |
| H5 | RM-07 filtre les acteurs autant que les appartenances | **Infirmée en pratique.** Un filtrage strict des acteurs laisse 3 687 des 6 028 entrées initiateur (61 %, surtout des sénateurs) affichées en identifiant brut, et contredit CU-02 qui suppose le ministre connu. Retenu : identité ingérée sans filtre, groupes et appartenances restreints à la législature en cours |

## 11. Questions ouvertes — tranchées

Mesures faites sur les jeux officiels le 3 août 2026 (législature 17).

### Q1 — Plusieurs appartenances actives à une même date
**Sans objet.** 18 chevauchements observés, **tous sur le même groupe**, la qualité seule
diffère (Membre + Président). **Zéro chevauchement entre groupes distincts**, y compris en
traitant la date de fin comme inclusive.

**Règle retenue** : à qualité multiple sur un même groupe, l'appartenance la plus récemment
ouverte l'emporte — le groupe est identique, seule la qualité affichée change. Deux groupes
distincts à la même date restent traités comme une donnée incohérente : rien n'est affiché,
l'anomalie est signalée. Le cas est implémenté bien qu'il ne se présente pas.

### Q2 — Groupe dissous en cours de législature
**Libellé historique seul.** La date de référence affichée à côté du groupe porte déjà
l'information ; une mention « dissous » alourdit sans rien ajouter de vérifiable sur l'acte.

Portée mesurée : 60 entrées initiateur sur 60 dossiers, toutes UDR (dissous le 04/09/2025).
AD (dissous le 11/09/2024) n'a déposé aucun texte. Les dates de vie des groupes restent
stockées (`parliamentary_groups.end_date`) pour un usage ultérieur.

### Q3 — Ministre initiateur ayant siégé par le passé
**Fonction seule.** Lecture stricte de RM-01 : on n'affiche que ce qui était vrai à la date
de l'acte. Rappeler un groupe qu'il n'occupait pas ce jour-là placerait une étiquette
politique à côté d'un acte qu'elle ne qualifie pas.

Portée mesurée : 357 entrées initiateur pour 33 ministres, dont 16 ont eu un groupe
parlementaire par le passé (161 entrées). Cas typique : Michel Barnier, dépôts d'octobre 2024
comme Premier ministre, député seulement depuis le 29/09/2025.

### Q4 — Référence de groupe factice dans les scrutins
**Répartition reconstruite, avec mention de méthode.**

Chiffre corrigé : la sentinelle `PO0` occupe **146 lignes de groupe réparties sur 14 scrutins**,
et non 146 scrutins. Dans 12 d'entre eux, *toutes* les lignes sont `PO0` : la ventilation par
groupe publiée est intégralement perdue.

Le décompte nominatif, lui, survit — chaque votant porte son `acteurRef`. La répartition a été
reconstruite depuis le référentiel sur les 14 scrutins : **totaux identiques au déclaré 14 fois
sur 14, zéro acteur non résolu**. Les deux options initiales supposaient l'information perdue ;
elle ne l'est pas.

L'affichage porte une mention du type « répartition reconstituée à partir du décompte nominatif —
la source ne publie pas les groupes sur ce scrutin », plus le lien source. Le chiffre est produit
par le site, pas par l'Assemblée : le dire relève de PROJECT.md §2 (lacune affichée, jamais
silencieuse) et §9 (méthode publiée).

→ À reprendre dans la spec SCRUTINS (Phase 3).
