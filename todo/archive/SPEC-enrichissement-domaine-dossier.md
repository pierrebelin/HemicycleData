# ENRICH-DOSSIER — Enrichissement du domaine dossier législatif

> Ajouter 3 données au dossier législatif : étape courante dans la navette parlementaire, initiateurs (députés/ministres + groupe politique), commission saisie au fond. Objectif : nourrir la génération de posts Instagram avec du contexte politique exploitable.

## 1. Contexte

Dossier législatif aujourd'hui = titre, procédure, actes (date+label), score. Pas assez riche pour un post IG percutant. Manque : qui porte le texte, où il en est dans la navette, quelle commission l'examine. Ces 3 infos sont présentes dans les données brutes AN mais ignorées au parsing.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Navette parlementaire | Aller-retour d'un texte entre Assemblée nationale et Sénat jusqu'à adoption |
| Étape | Phase macro du parcours : 1ère lecture AN, 1ère lecture Sénat, CMP, nouvelle lecture, lecture définitive, Conseil constitutionnel, promulgation |
| Initiateur | Personne (député, sénateur, ministre) à l'origine du dépôt du texte |
| Groupe parlementaire | Groupe politique d'appartenance d'un député (ex. LFI, RN, Renaissance) |
| Commission saisie au fond | Commission permanente responsable de l'examen du texte |

## 3. Cas d'usage

### CU-01 — Consulter l'étape courante d'un dossier

**Acteur** : utilisateur · **Intention** : savoir où en est un texte dans la navette · **Fréquence** : chaque consultation de dossier

**Scenario nominal :**
1. Utilisateur ouvre le détail d'un dossier.
2. Système affiche l'étape courante (ex. "1ère lecture — Assemblée nationale") et la chambre concernée.

**Variantes :** dossier sans acte législatif → étape = inconnue.

### CU-02 — Consulter les initiateurs d'un dossier

**Acteur** : utilisateur · **Intention** : savoir qui porte le texte · **Fréquence** : chaque consultation de dossier

**Scenario nominal :**
1. Utilisateur ouvre le détail d'un dossier.
2. Système affiche la liste des initiateurs : nom complet + groupe parlementaire (sigle).

**Variantes :**
- Initiateur non trouvé dans le référentiel des députés → affiche seulement l'identifiant brut (acteurRef).
- Dossier gouvernemental sans acteurRef → initiateur = "Gouvernement" (déduit du type de procédure "Projet de loi").

### CU-03 — Consulter la commission saisie au fond

**Acteur** : utilisateur · **Intention** : savoir quelle commission examine le texte · **Fréquence** : chaque consultation de dossier

**Scenario nominal :**
1. Utilisateur ouvre le détail d'un dossier.
2. Système affiche le nom de la commission (ex. "Commission des lois").

**Variantes :** dossier sans renvoi en commission → commission = absente.

## 4. Règles métier

### RM-01 — Détermination de l'étape courante
- **Énoncé** : étape courante = acte de plus haut niveau (codeActe top-level) ayant la date la plus récente parmi ses descendants. · **Origine** : structure données AN · **Sévérité** : bloquant
- **Applies to** : CU-01

Les étapes ordonnées de la navette :

| Code acte | Étape | Chambre |
|---|---|---|
| AN1 | 1ère lecture | Assemblée nationale |
| SN1 | 1ère lecture | Sénat |
| AN2 / SN2 | 2ème lecture | AN / Sénat |
| CMP | Commission mixte paritaire | Conjointe |
| ANNLEC / SNNLEC | Nouvelle lecture | AN / Sénat |
| ANLDEF | Lecture définitive | Assemblée nationale |
| CC | Conseil constitutionnel | — |
| PROM | Promulgation | — |

### RM-02 — Résolution des initiateurs
- **Énoncé** : chaque acteurRef du champ `initiateur` est résolu vers un nom+prénom+groupe via le référentiel des députés en exercice. Résolution best-effort : échec silencieux, on garde l'acteurRef brut. · **Origine** : choix produit · **Sévérité** : warning
- **Applies to** : CU-02

### RM-03 — Initiateur implicite pour les projets de loi
- **Énoncé** : si procédure = "Projet de loi *" et aucun acteurRef présent → initiateur = "Gouvernement" sans groupe. · **Origine** : convention parlementaire · **Sévérité** : informatif
- **Applies to** : CU-02

### RM-04 — Commission par mapping statique
- **Énoncé** : l'organeRef extrait de l'acte `*-COM-FOND-SAISIE` est résolu via un catalogue fixe des 8 commissions permanentes de l'AN. · **Origine** : choix produit (les commissions changent rarement) · **Sévérité** : bloquant
- **Applies to** : CU-03

Les 8 commissions permanentes :

| Libellé |
|---|
| Commission des affaires culturelles et de l'éducation |
| Commission des affaires économiques |
| Commission des affaires étrangères |
| Commission des affaires sociales |
| Commission de la défense nationale et des forces armées |
| Commission du développement durable et de l'aménagement du territoire |
| Commission des finances, de l'économie générale et du contrôle budgétaire |
| Commission des lois constitutionnelles, de la législation et de l'administration générale de la République |

### RM-05 — Tous les initiateurs conservés
- **Énoncé** : un dossier peut avoir 1 à N initiateurs. Tous sont stockés (pas de troncature au premier). · **Origine** : choix produit · **Sévérité** : bloquant
- **Applies to** : CU-02

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Étape courante (code + libellé + chambre) | Phase dans la navette | Calculée depuis codeActe top-level | Essentiel |
| Initiateurs (nom, prénom, groupe) | Auteurs du texte | Importée (ZIP AN + API nosdeputes.fr) | Essentiel |
| Commission saisie au fond (libellé) | Commission examinatrice | Importée (ZIP AN) + catalogue statique | Secondaire |

## 6. États & transitions

Pas de nouveau cycle de vie. L'étape courante est une donnée calculée, pas un état géré.

## 7. Comportements transverses

### Résolution des députés
Référentiel des députés chargé depuis nosdeputes.fr. Cache en mémoire (même logique que le ZIP). Correspondance par identifiant AN (acteurRef type "PA######"). Si l'API est indisponible, les initiateurs restent sous forme d'identifiants bruts — pas bloquant.

### Impact sur le scoring
L'étape courante pourrait remplacer le `score_progress` actuel (basé sur le libellé du dernier acte) par un calcul sur le codeActe. Hors périmètre de cette spec — amélioration séparée.

## 8. Relations

| Amont | Aval |
|---|---|
| ZIP dossiers AN (actesLegislatifs, initiateur) | Étape, commission, identifiants initiateurs |
| API nosdeputes.fr | Noms + groupes parlementaires |

## 9. Hors périmètre

| Exclusion | Raison |
|---|---|
| Amendements | Source de données distincte, complexité élevée |
| Votes / scrutins | Nécessite un dataset séparé |
| Refonte du scoring avec codeActe | Amélioration séparée |
| Persistance BDD des nouvelles données | Itération suivante — enrichissement d'abord côté source/parsing |

## 10. Hypothèses

| # | Hypothèse | À valider par |
|---|---|---|
| H1 | L'API nosdeputes.fr expose un champ permettant la correspondance avec les acteurRef AN (format PA######) | Vérification technique |
| H2 | Les 8 commissions permanentes suffisent — les commissions spéciales/ad hoc sont ignorées | PO |
| H3 | Les codes acte top-level (AN1, SN1, CMP…) sont stables entre législatures | Vérification données AN |

## 11. Questions ouvertes

| # | Question | Impact | Options |
|---|---|---|---|
| Q1 | Le référentiel nosdeputes.fr utilise un `id` propre, pas l'acteurRef AN — comment faire la correspondance ? | Résolution initiateurs | Scraper le slug nosdeputes, enrichir le ZIP AN avec les noms directs, ou utiliser un autre référentiel |

→ Étape suivante : /implement
