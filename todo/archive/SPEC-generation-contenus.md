# GEN-CONTENU — Génération de scripts et posts

> Pour un dossier sélectionné, générer la matière publiable : script vidéo 60-90 s (livraison 1), puis déclinaison post texte légende + carrousel (livraison 2). Brouillon éditable, validation manuelle obligatoire, règles dures de neutralité.

## 1. Contexte

La curation s'arrête aujourd'hui à « Selected » : la rédaction du contenu est entièrement manuelle. La génération transforme un dossier riche (titre, étape, initiateurs, votes) en brouillon fidèle et neutre ; l'humain contrôle le choix du sujet et l'exactitude factuelle, pas la rédaction (README.md §3).

## 2. Vocabulaire

| Terme | Définition |
|---|---|
| Script | Texte à dire en vidéo, 60-90 s de lecture à voix haute |
| Post | Légende Instagram + découpage en slides de carrousel |
| Brouillon | Contenu généré ou édité, pas encore validé |
| Contenu validé | Brouillon relu et approuvé : faits exacts, neutralité respectée |
| File de production | Dossiers Selected sans contenu publié — la « short-list » hebdo est une convention, pas une entité |

## 3. Cas d'usage

### CU-01 — Générer un brouillon
**Acteur** : utilisateur · **Intention** : obtenir la matière d'un contenu sur un sujet choisi · **Fréquence** : 3-5 fois par semaine

**Scenario nominal :**
1. Utilisateur ouvre un dossier Selected.
2. Utilisateur demande la génération (script, ou post en livraison 2).
3. Système génère un brouillon depuis les données du dossier (RM-01 à RM-06).
4. Système affiche le brouillon, éditable.

**Variantes :** brouillon existant → régénération le remplace après confirmation. **Erreurs :** génération en échec → dossier inchangé, erreur affichée, aucun brouillon partiel.

### CU-02 — Éditer et valider un brouillon
**Acteur** : utilisateur · **Intention** : corriger puis approuver le contenu · **Fréquence** : chaque contenu

**Scenario nominal :**
1. Utilisateur relit le brouillon : exactitude des faits, neutralité (test : une personne de droite ET de gauche le trouveraient-elles juste ?).
2. Utilisateur édite le texte à la main si besoin.
3. Utilisateur marque le contenu validé.

**Variantes :** édition d'un contenu déjà validé → repasse brouillon (RM-07).

### CU-03 — Marquer publié
**Acteur** : utilisateur · **Intention** : tracer ce qui est parti sur les réseaux · **Fréquence** : chaque publication

**Scenario nominal :**
1. Utilisateur a publié le contenu validé sur le réseau (hors application).
2. Utilisateur marque le contenu publié → le dossier passe Published.

## 4. Règles métier

### RM-01 — Faits uniquement
- **Énoncé** : le contenu rapporte : quel texte, ce qu'il prévoit, qui le porte, résultat des votes, étape courante. Aucune évaluation (bon/mauvais, juste/injuste, efficace/non). · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-02 — Des nombres, pas d'adverbes
- **Énoncé** : résultats de vote en chiffres bruts, jamais qualifiés. · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-01
- **Exemple conforme** : « rejeté par 280 voix contre 250 ». **Non conforme** : « massivement rejeté ».

### RM-03 — Positions attribuées et on-record uniquement
- **Énoncé** : toute position rapportée est attribuée et issue de sources officielles (exposé des motifs, position de vote par groupe). Format « les défenseurs avancent X / les opposants Y ». Jamais de commentaire média ni de synthèse libre. · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-04 — Chiffres lus, jamais générés
- **Énoncé** : chaque chiffre de vote du contenu provient des scrutins ingérés (spec SCRUTINS). Dossier sans scrutin → le contenu dit « adopté » / « rejeté » sans chiffres. · **Origine** : README.md §7 · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-05 — Lien source officielle
- **Énoncé** : chaque contenu inclut le lien vers la source officielle (dossier, scrutin ou compte rendu). · **Origine** : README.md §6 · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-06 — Format du script
- **Énoncé** : script structuré accroche → faits → votes → où ça en est → source, durée de lecture 60-90 s. · **Origine** : choix produit (format vidéo courte) · **Sévérité** : bloquant · **Applies to** : CU-01

### RM-07 — Validation manuelle obligatoire
- **Énoncé** : aucun contenu n'est validé sans action humaine explicite. Toute édition ou régénération d'un contenu validé le repasse brouillon. · **Origine** : README.md §3 (point de contrôle humain) · **Sévérité** : bloquant · **Applies to** : CU-02

### RM-08 — Génération réservée aux dossiers Selected
- **Énoncé** : la génération n'est proposée que sur un dossier au statut Selected. · **Origine** : choix produit (la sélection précède la production) · **Sévérité** : bloquant · **Applies to** : CU-01

## 5. Données

| Donnée | Description | Source | Importance |
|---|---|---|---|
| Script (texte) | Contenu vidéo 60-90 s | Calculée (LLM) puis saisie (éditions) | Essentiel |
| Post (légende + slides) | Déclinaison texte — livraison 2 | Calculée (LLM) puis saisie | Essentiel |
| Statut du contenu | Brouillon / validé / publié | Saisie | Essentiel |
| Dates génération / validation / publication | Traçabilité | Calculée | Secondaire |

## 6. États & transitions

| État | Événement | État suivant | Condition |
|---|---|---|---|
| (aucun contenu) | Génération | Brouillon | Dossier Selected |
| Brouillon | Régénération / édition | Brouillon | — |
| Brouillon | Validation | Validé | Action humaine |
| Validé | Édition ou régénération | Brouillon | RM-07 |
| Validé | Marquage publié | Publié | Dossier passe Published |

## 7. Comportements transverses

### File de production
Pas d'entité « édition hebdomadaire ». La short-list du récap = dossiers Selected sans contenu publié, visibles ensemble. Le rythme (vendredi, 3-5 sujets) reste une discipline d'usage.

## 8. Relations

| Amont | Aval |
|---|---|
| Dossier enrichi (étape, initiateurs, commission) | Corps du contenu |
| Scrutins du dossier (spec SCRUTINS) | Chiffres de vote du contenu |
| Curation (Selected) | Droit de générer ; publication → Published |

## 9. Hors périmètre

| Exclusion | Raison |
|---|---|
| Production vidéo (montage, voix, habillage) | Hors application, outil dédié |
| Publication automatique vers Instagram/TikTok | Publication manuelle assumée ; le marquage suffit |
| Entité « édition hebdomadaire » et éditions spéciales | Convention d'usage retenue à la place |
| Vérification automatique de neutralité | La validation humaine porte ce contrôle |

## 10. Hypothèses

| # | Hypothèse | À valider par |
|---|---|---|
| H1 | Les données dossier + scrutins suffisent à un script complet sans les comptes rendus de séance (positions détaillées des groupes) | Premiers scripts réels |
| H2 | 60-90 s de lecture ≈ longueur de texte stable, contrôlable à la génération | Premiers scripts réels |

## 11. Questions ouvertes

| # | Question | Impact | Options |
|---|---|---|---|
| Q1 | Ordre des livraisons interne : le post texte (livraison 2) attend-il le retour d'expérience des premiers scripts ? | Planning | Oui (recommandé lors du grill) / non |
| Q2 | Faut-il ingérer les exposés des motifs pour nourrir RM-03 (positions des défenseurs) dès la livraison 1 ? | Richesse du contenu | Titre + actes seuls d'abord, ou exposé des motifs inclus |

→ Étape suivante : /plan-implementation
