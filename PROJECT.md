# hémicycle.data — Veille parlementaire neutre, format court

Outil de veille sur l'activité du Parlement français (Assemblée nationale + Sénat)
qui sélectionne automatiquement les sujets pertinents et prépare la matière de
vidéos courtes (Instagram / TikTok), avec validation manuelle avant production.

## 1. Intention

Informer un public de **25-40 ans** qui veut suivre la vie politique sans y connaître
grand-chose, en racontant **ce qui s'est passé au Parlement** de façon claire et accessible.

Principe éditorial directeur : on **priorise les sujets qui comptent déjà pour les gens
afin de les informer** — jamais pour faire le buzz ou attiser le clivage. C'est un service
d'information neutre, pas un média d'engagement par la polémique.

- **Public** : actifs 25-40 ans, curieux, peu experts.
- **Ton** : léger sur la forme, rigoureux sur le fond.
- **Promesse** : « voici ce qui a été voté / discuté, par qui, et où ça en est ».
- **Usage** : outil d'abord personnel ; ouvert si ça intéresse d'autres.

## 2. Format & rythme

- **Récap hebdomadaire**, créneau fixe (vendredi / week-end), 3 à 5 sujets.
- **Éditions spéciales** ponctuelles, déclenchées par un gros texte (budget, réforme
  majeure, motion de censure).
- **Pas de quota quotidien public** : on publie quand il y a de la matière (le Parlement
  ne siège pas en continu — session d'octobre à fin juin, surtout mardi-jeudi).
- **Suivi quotidien interne** : l'ingestion tourne en continu et alimente un journal
  de la semaine (digest perso possible), distinct du produit publié.

## 3. Architecture (vue d'ensemble)

Ingestion continue → Journal interne de la semaine → Scoring → Short-list (3-5)
→ [VALIDATION MANUELLE] → Génération scripts → Production vidéo

- L'ingestion et le scoring tournent **au fil de l'eau**.
- La **publication** est hebdomadaire.
- Le **point de contrôle humain** porte sur deux choses : le *choix des sujets* et
  l'*exactitude factuelle*. Pas sur la rédaction de chaque post.

## 4. Sélection des sujets — grille de scoring

But : passer du flux brut (dizaines de textes/votes/amendements par semaine) à une
short-list. Score pondéré + un filtre.

| Critère              | Mesure                                                                 | Poids | Type     |
|----------------------|------------------------------------------------------------------------|-------|----------|
| Proximité thématique | Appartenance aux familles cibles (voir §5)                             | ×3    | LLM      |
| Impact concret       | Change-t-il quelque chose de tangible dans une vie de trentenaire ?   | ×3    | LLM      |
| Résonance actu       | Sujet déjà présent dans la conversation publique (→ contexte acquis)  | ×3    | LLM      |
| Ampleur              | Nb de personnes concernées, poids budgétaire (PLF/PLFSS = max)        | ×2    | mixte    |
| Avancement           | Vote solennel adopté > 1ʳᵉ lecture > amendement commission > dépôt    | ×2    | objectif |
| Importance (signal)  | Participation des groupes, ampleur du débat — **comme proxy d'importance, pas de clivage** | ×1 | objectif |
| **Vulgarisabilité**  | Explicable en 60-90 s ? Sinon → mention courte ou simplification      | *filtre* | LLM   |

Règles :

- **Critères objectifs** = lus directement dans les données structurées (fiables, non
  contestables).
- **Critères LLM** = jugement → ce sont eux que la validation manuelle surveille en priorité.
- **Vulgarisabilité = filtre, pas poids** : un sujet pertinent mais imbitable en court
  est simplifié à l'extrême ou rétrogradé en mention.
- **Pondérations à calibrer sur données réelles** (quelques semaines de tournée à blanc),
  pas à figer dans l'abstrait.
- **Seuils à définir** : score minimal pour « sujet développé » vs « mention rapide » vs
  « édition spéciale ».

## 5. Familles thématiques cibles

- Pouvoir d'achat / fiscalité (le sujet roi)
- Logement (loyers, accès propriété, locations courte durée)
- Travail / emploi (droit du travail, chômage, indépendants, télétravail)
- Environnement / énergie (prix de l'énergie, transition, transports)
- Numérique (données, IA, réseaux sociaux, fraude en ligne)
- Santé / social (remboursements, accès aux soins, congés)
- Société / libertés (**terrain sensible : factuel uniquement**)

À écarter : procédure interne, ratifications techniques, ajustements juridiques obscurs.

## 6. Neutralité — règles dures

La neutralité est une contrainte de génération, vérifiée à la validation manuelle.

- **Faits uniquement** : quel texte, ce qu'il prévoit, qui l'a voté, résultat, étape.
- **Aucune évaluation** du texte (bon/mauvais, juste/injuste, efficace/non).
- **Répartition des votes brute** plutôt que synthèse (voir §7).
- **Des nombres, pas d'adverbes** : « rejeté par 280 voix contre 250 » ✅ ;
  « massivement rejeté » ❌.
- **Positions attribuées et on-record uniquement** : exposé des motifs, interventions
  en séance (comptes rendus), position de vote par groupe. Jamais de commentaire média
  ni de synthèse libre. Format « les défenseurs avancent X / les opposants Y », attribué.
- **Le LLM rapporte et reformule fidèlement, il ne conclut jamais.**
- **Lien vers la source officielle sur chaque sujet** (scrutin / compte rendu / dossier).
- **Test qualité** : « une personne de droite ET de gauche trouveraient-elles ça juste ? »

## 7. Traitement des votes

- Afficher la **répartition par groupe** (pour / contre / abstention / non-votants),
  chiffres bruts.
- **Limite technique à gérer** : la répartition nominale n'existe que pour les
  **scrutins publics**. Beaucoup de textes passent à main levée → seulement « adopté » /
  « rejeté », sans répartition.
- L'outil gère les **deux cas** : répartition quand elle existe, résultat sec sinon.
- **Ne jamais inventer de chiffres** : le LLM ne lit que la donnée structurée sur ce point.

## 8. Sources de données

### Socle Assemblée nationale — `data.assemblee-nationale.fr`
Colonne vertébrale. XML / JSON. Jeux clés :
- Dossiers législatifs (textes en cours, adoptés, promulgués ; titres, dépôts,
  commissions, rapporteurs, dates).
- Scrutins : position de vote de chaque député (→ répartitions §7).
- Comptes rendus de séance (dates, thèmes, orateurs, débats → positions attribuées).
- Amendements (auteur, objet, sort) → signal « importance ».
- Agenda des travaux en séance.

### Vie publique — `vie-publique.fr`
- « Panorama des lois » : résumés en langage clair et neutre → **graine de vulgarisation**.
- **Depuis le 1ᵉʳ avril 2026, l'accès aux dossiers législatifs passe par Vie publique**
  (redirections depuis Légifrance). À intégrer dans le routage des sources.

### Légifrance via PISTE — API officielle
- Pour les textes promulgués / le droit en vigueur / le Journal officiel.
- Gratuite après inscription, auth OAuth (token Bearer), sandbox + prod.

### Couches de simplification (tierces — vérifier la pérennité avant d'en dépendre)
- **CIVIX** (API publique read-only sur data.gouv.fr) : agrège scrutins, votes
  individuels, groupes, dossiers, sans interprétation.
- **Tricoteuses** (`@tricoteuses/assemblee`) : données AN nettoyées et réorganisées.
- **NosDéputés.fr / ParlAPI.fr** (Regards Citoyens) : recherche plein texte sur débats,
  amendements, questions, rapports → idéal pour le contexte attribué.

### Sénat — `data.senat.fr`
- Pour couvrir la navette complète. **Vérifier les formats actuels** au moment d'intégrer.

## 9. Stack pressentie

- **.NET / C#** (profil porteur ; toutes les sources sont language-agnostic via REST/XML/JSON).
- LLM en BYOK (reformulation/scoring contraints, pas de génération libre du fond).
- Stockage du journal interne + historique des scores (calibration des poids).

## 10. Feuille de route

- [ ] **Phase 0** — Repo, choix des sources de départ (AN + Vie publique).
- [ ] **Phase 1** — Ingestion AN (scrutins + dossiers) → journal interne.
- [ ] **Phase 2** — Scoring + short-list hebdo (poids provisoires).
- [ ] **Phase 3** — Mécanique de neutralité + génération de scripts.
- [ ] **Phase 4** — Production vidéo + interface de validation manuelle.
- [ ] **Calibration** — Ajuster les poids et seuils sur données réelles.

## Non-objectifs (à garder en tête)

- Pas de recherche de viralité par la polémique.
- Pas d'opinion, pas de conclusion éditoriale.
- Pas de publication forcée les jours sans matière.
