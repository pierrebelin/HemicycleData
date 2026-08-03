# SCRUTINS — Scrutins et votes des dossiers

> ⚠️ **À réviser avant implémentation** — le pivot vers le site de transparence (PROJECT.md, août 2026) invalide plusieurs choix de cette spec :
> - **RM-01** (rattachement obligatoire à un dossier) écarte les scrutins orphelins. Mesuré sur les données réelles le 02/08/2026 : **5 826 scrutins sur 8 434 (69 %) n'ont aucun dossier rattaché**. Cette règle viderait le site des deux tiers des votes. PROJECT.md §7 impose de les conserver.
> - **RM-02** (deux régimes de vote) est faux : les 8 434 scrutins portent tous le décompte nominatif. Les votes à main levée sont absents du jeu, pas présents sans répartition — c'est un trou de couverture à afficher, pas un mode d'affichage.
> - **Bonne nouvelle** : chaque votant porte la référence de son mandat, donc son groupe **au moment du vote**, sans recherche par date. La répartition par groupe ne dépend pas de la phase acteurs — seul l'affichage des libellés en dépend.
> - **Position nominale par député** est en hors périmètre §9 ; les données la fournissent pour tous les scrutins et elle devient le niveau de preuve central (PROJECT.md §7).
> - Les justifications « matière des contenus générés » tombent : la couche éditoriale est abandonnée.

> Ingérer les scrutins publics de l'Assemblée nationale rattachés aux dossiers suivis. Afficher la répartition des votes par groupe politique en chiffres bruts. Matière des futurs contenus (« rejeté par 280 voix contre 250 ») et du critère importance du scoring.

## 1. Contexte

Un dossier montre aujourd'hui ses actes (« Vote solennel ») sans le résultat du vote. Impossible de dire qui a voté quoi — cœur de la promesse produit (« voté par qui »). Les scrutins publics AN portent la position de chaque député ; il faut les rattacher aux dossiers.

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Scrutin public | Vote enregistré nominalement, position de chaque député connue |
| Vote à main levée | Vote non enregistré : seul le résultat (adopté/rejeté) existe |
| Répartition par groupe | Pour / contre / abstention / non-votants, comptés par groupe politique |
| Scrutin orphelin | Scrutin non rattachable à un dossier suivi (motion de censure, résolution) |

## 3. Cas d'usage

### CU-01 — Ingérer les scrutins au rafraîchissement
**Acteur** : système · **Intention** : chaque dossier porte ses scrutins à jour · **Fréquence** : chaque rafraîchissement

**Scenario nominal :**
1. Rafraîchissement récupère les scrutins publics AN de la période.
2. Système rattache chaque scrutin à son dossier (RM-01).
3. Système stocke : date, objet, résultat, totaux, répartition par groupe.

**Variantes :** scrutin orphelin → ignoré (RM-01). **Erreurs :** source scrutins indisponible → rafraîchissement des dossiers continue sans scrutins, signalé.

### CU-02 — Consulter les votes d'un dossier
**Acteur** : utilisateur · **Intention** : savoir qui a voté quoi · **Fréquence** : chaque consultation de dossier candidat à un contenu

**Scenario nominal :**
1. Utilisateur ouvre le détail d'un dossier.
2. Système liste les scrutins : date, objet (ex. « Ensemble du texte, 1ère lecture »), résultat, totaux.
3. Utilisateur déplie un scrutin → répartition par groupe, chiffres bruts, lien vers la page officielle du scrutin.

**Variantes :** vote à main levée → résultat sec, sans répartition (RM-02). Dossier sans scrutin → section absente.

## 4. Règles métier

### RM-01 — Rattachement obligatoire
- **Énoncé** : un scrutin n'entre dans l'application que rattaché à un dossier suivi. Scrutin orphelin ignoré. · **Origine** : choix produit (périmètre centré dossier) · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-02 — Deux régimes de vote
- **Énoncé** : scrutin public → répartition par groupe affichée. Main levée → résultat sec « adopté » / « rejeté », aucune répartition. Absence de répartition ≠ zéro voix. · **Origine** : PROJECT.md §7 (limite des données AN) · **Sévérité** : bloquant · **Applies to** : CU-01, CU-02

### RM-03 — Chiffres bruts, jamais inventés
- **Énoncé** : tout chiffre de vote affiché provient tel quel des données officielles. Aucun chiffre calculé, estimé ou reformulé. · **Origine** : PROJECT.md §6-7 (neutralité) · **Sévérité** : bloquant · **Applies to** : transverse
- **Exemple conforme** : « Pour : 280 — Contre : 250 ». **Non conforme** : « adopté à une large majorité ».

### RM-04 — Lien source officielle
- **Énoncé** : chaque scrutin affiché porte un lien vers sa page officielle AN. · **Origine** : PROJECT.md §6 · **Sévérité** : bloquant · **Applies to** : CU-02

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Scrutin (date, objet, type) | Identité du vote | Importée (data AN) | Essentiel |
| Résultat | Adopté / rejeté | Importée | Essentiel |
| Totaux | Pour / contre / abstention / non-votants | Importée | Essentiel |
| Répartition par groupe | Totaux ventilés par groupe politique | Importée | Essentiel |
| Lien officiel | Page du scrutin sur le site AN | Importée | Essentiel |

## 8. Relations

| Amont | Aval |
|---|---|
| Données scrutins AN | Scrutins rattachés aux dossiers |
| Scrutins d'un dossier | Contenus générés (chiffres de vote, spec GEN-CONTENU) |
| Participation des groupes | Critère importance du scoring (SCORE-LLM RM-07) |

## 9. Hors périmètre

| Exclusion | Raison |
|---|---|
| Scrutins orphelins (motions de censure, résolutions) | Reviendront avec les éditions spéciales |
| Position nominale par député | La répartition par groupe suffit au format court |
| Scrutins du Sénat | Sénat entièrement hors périmètre actuel |
| Votes en commission | Données distinctes, valeur faible pour le format |

## 10. Hypothèses

| # | Hypothèse | À valider par |
|---|---|---|
| H1 | Les données scrutins AN référencent le dossier (ou le texte) permettant le rattachement automatique | Vérification données AN |
| H2 | La répartition par groupe est directement présente ou déductible des positions individuelles fournies | Vérification données AN |

→ Étape suivante : /plan-implementation
