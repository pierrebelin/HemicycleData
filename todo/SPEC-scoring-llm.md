# SCORE-LLM — Scoring par grille complète

> Remplace le score heuristique 3 dimensions (progress, magnitude, momentum) par la grille cible de PROJECT.md §4 : 6 critères pondérés + 1 filtre. 3 critères notés par jugement automatisé (LLM), 3 lus dans les données structurées. But : short-list crédible, sujets pertinents pour le public 25-40 ans jamais ratés.

## 1. Contexte

Score actuel = mots-clés dans titre et libellé d'acte. Rate les sujets à fort impact concret sans mot-clé, surnote les textes techniques bien nommés. La curation manuelle compense en relisant tout le flux — coût humain que le scoring devait éviter.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Critère LLM | Critère noté par jugement automatisé : proximité thématique, impact concret, résonance actu |
| Critère objectif | Critère calculé depuis données structurées : ampleur, avancement, importance |
| Vulgarisabilité | Sujet explicable en 60-90 s ? Filtre, pas un poids |
| Famille thématique | Une des 7 familles cibles de PROJECT.md §5 (pouvoir d'achat, logement, travail, environnement, numérique, santé, société) |
| Sujet développé | Dossier candidat à un contenu complet |
| Mention courte | Dossier pertinent mais inexplicable en court : cité en une phrase, pas développé |
| Notation | Passage d'un dossier dans la grille complète, produit notes + justifications |

## 3. Cas d'usage

### CU-01 — Noter le flux au rafraîchissement
**Acteur** : système · **Intention** : chaque dossier ingéré porte un score grille complète · **Fréquence** : chaque rafraîchissement

**Scenario nominal :**
1. Rafraîchissement ingère les dossiers.
2. Système recalcule les critères objectifs de tous les dossiers.
3. Système note via LLM les dossiers jamais notés ou avec nouvel acte depuis dernière notation (RM-02).
4. Système calcule le score total pondéré, applique le filtre vulgarisabilité (RM-03).

**Erreurs :** LLM indisponible → RM-05.

### CU-02 — Consulter la décomposition du score
**Acteur** : utilisateur · **Intention** : comprendre pourquoi ce score, surveiller les jugements LLM · **Fréquence** : chaque consultation de dossier

**Scenario nominal :**
1. Utilisateur ouvre le détail d'un dossier.
2. Système affiche : note par critère, poids, justification pour chaque critère LLM (RM-04), famille thématique détectée, catégorie (développé / mention courte), date de notation.

**Variantes :** dossier jamais noté LLM → critères objectifs seuls, mention « notation en attente ».

### CU-03 — Obtenir les suggestions classées
**Acteur** : utilisateur · **Intention** : voir les meilleurs sujets de la période · **Fréquence** : hebdomadaire

**Scenario nominal :**
1. Utilisateur ouvre les suggestions.
2. Système classe les dossiers non curatés par score total décroissant, catégorie visible.

**Variantes :** mentions courtes listées à part des sujets développés.

## 4. Règles métier

### RM-01 — Grille de pondération
- **Énoncé** : score total = somme pondérée des 6 critères, normalisée 0-100. Grille : · **Origine** : PROJECT.md §4 · **Sévérité** : bloquant · **Applies to** : CU-01

| Critère | Mesure | Poids | Type |
|---|---|---|---|
| Proximité thématique | Appartenance aux 7 familles cibles | ×3 | LLM |
| Impact concret | Change quelque chose de tangible dans une vie de trentenaire ? | ×3 | LLM |
| Résonance actu | Sujet déjà dans la conversation publique | ×3 | LLM |
| Ampleur | Nb de personnes concernées, poids budgétaire (PLF/PLFSS = max) | ×2 | objectif |
| Avancement | Étape courante de la navette (promulgation > vote solennel > … > dépôt) | ×2 | objectif |
| Importance (signal) | Ampleur de l'activité parlementaire — proxy d'importance, pas de clivage | ×1 | objectif |

### RM-02 — Déclenchement de la notation LLM
- **Énoncé** : un dossier passe au LLM si jamais noté, ou si nouvel acte depuis la dernière notation. Critères objectifs recalculés à chaque rafraîchissement. · **Origine** : choix produit (maîtrise du coût) · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-03 — Vulgarisabilité = filtre
- **Énoncé** : dossier jugé non explicable en 60-90 s → catégorie « mention courte ». Jamais exclu du flux, jamais d'impact sur le score total. · **Origine** : PROJECT.md §4 · **Sévérité** : bloquant · **Applies to** : CU-01, CU-03

### RM-04 — Justification obligatoire
- **Énoncé** : chaque critère LLM porte une justification d'une à deux phrases. Critère sans justification = notation invalide. · **Origine** : PROJECT.md §3 (la validation manuelle surveille les jugements LLM en priorité) · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02

### RM-05 — Échec LLM non bloquant
- **Énoncé** : notation LLM en échec → dossier conserve ses critères objectifs, marqué « notation en attente », re-tenté au prochain rafraîchissement. Le rafraîchissement n'échoue jamais pour cause de LLM. · **Origine** : choix produit · **Sévérité** : warning · **Applies to** : CU-01

### RM-06 — Le LLM ne lit que le dossier
- **Énoncé** : la notation s'appuie exclusivement sur les données du dossier (titre, procédure, actes, étape, initiateurs, commission). Aucun fait externe injecté dans les justifications. · **Origine** : PROJECT.md §6 (neutralité) · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-07 — Importance dégradée sans scrutins
- **Énoncé** : tant que les scrutins ne sont pas ingérés (spec SCRUTINS), le critère importance s'appuie sur le volume d'actes du dossier. Enrichi par la participation aux scrutins ensuite. · **Origine** : choix produit (ordre de livraison) · **Sévérité** : informatif · **Applies to** : CU-01

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Note par critère (×6) | 0-10 par critère | Calculée (LLM ou objectif) | Essentiel |
| Justification (×3 critères LLM) | 1-2 phrases par critère | Calculée (LLM) | Essentiel |
| Famille thématique | Famille cible détectée, ou aucune | Calculée (LLM) | Essentiel |
| Catégorie | Sujet développé / mention courte | Calculée (filtre vulgarisabilité) | Essentiel |
| Date de notation | Dernier passage LLM | Calculée | Secondaire |
| Score total | 0-100 normalisé | Calculée | Essentiel |

## 8. Relations

| Amont | Aval |
|---|---|
| Dossiers ingérés (titre, actes, étape, initiateurs, commission) | Notation |
| Score total + catégorie | Suggestions, sélection des sujets |
| Spec SCRUTINS (participation des groupes) | Enrichit le critère importance (RM-07) |

## 9. Hors périmètre

| Exclusion | Raison |
|---|---|
| Calibration des poids et seuils | Se fait sur données réelles après quelques semaines (PROJECT.md §4) |
| Seuil score « édition spéciale » | Dépend de la calibration |
| Veille médias externe pour la résonance actu | Le jugement s'appuie sur la culture générale du modèle, pas sur un flux d'actualité |
| Historique des scores pour calibration | Itération séparée |

## 10. Hypothèses

| # | Hypothèse | À valider par |
|---|---|---|
| H1 | La résonance actu jugée sans flux d'actualité externe reste utile (connaissances générales du modèle) | Tournée à blanc |
| H2 | Volume AN (dizaines de dossiers nouveaux/modifiés par semaine) → coût LLM acceptable | Suivi du premier mois |

## 11. Questions ouvertes

| # | Question | Impact | Options |
|---|---|---|---|
| Q1 | Seuil de score séparant sujet fort / sujet faible dans les suggestions | Affichage suggestions | TBD après calibration |
| Q2 | L'ampleur (nb personnes concernées) est-elle notée LLM en attendant des données budgétaires structurées ? PROJECT.md la dit « mixte » | Fiabilité du critère | LLM seul au départ, ou volet objectif dès maintenant |

→ Étape suivante : /plan-implementation
