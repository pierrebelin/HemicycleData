# hémicycle.data — Transparence des votes parlementaires

Site de consultation des votes de l'Assemblée nationale, organisés par thème et par
groupe parlementaire. Objectif : permettre à quiconque de vérifier, avant l'élection
présidentielle de 2027, **ce que les forces politiques ont réellement voté** — sur pièces,
sans interprétation.

## 1. Intention

Le débat électoral se joue sur des déclarations. Les votes, eux, sont publics, horodatés
et opposables — mais dispersés dans des jeux de données que personne ne consulte.
L'outil rapproche les deux : *voici les textes sur le logement, voici comment chaque
groupe a voté, voici la source officielle*.

- **Public** : électeurs qui veulent vérifier un discours contre un acte. Curieux, peu experts.
- **Promesse** : « sur ce sujet, voici les votes — chiffres bruts, source cliquable ».
- **Anti-promesse** : aucune note, aucun classement, aucun jugement sur un groupe.
- **Posture** : instrument de vérification, pas média d'opinion.

Le test qualité reste le même : *une personne de droite et une personne de gauche
trouveraient-elles la page juste ?*

## 2. Règle structurante — exhaustivité

C'est la contrainte la plus lourde du produit, et elle est non négociable.

Sur un site de transparence en période électorale, **le choix de ce qu'on montre est
déjà un acte éditorial**. Afficher 30 votes sur le logement et en taire 12 est
indéfendable, quelle que soit la raison technique.

Conséquences directes :

- On ingère et on expose **tous** les scrutins de la législature, sans sélection.
- Le scoring **ordonne** l'affichage (pertinence, ampleur), il ne **filtre** jamais.
- Tout critère de classement est **public et consultable** sur le site.
- Toute lacune connue (source indisponible, vote à main levée) est **affichée comme telle**,
  jamais silencieuse.

Ce point renverse l'ancienne logique de short-list : on ne jette plus rien.

## 3. Modèle politique — les trois pièges

### 3.1 Groupe parlementaire ≠ parti

L'Assemblée publie des **groupes**. Le public raisonne en **partis**. Ce ne sont pas les
mêmes objets : certains groupes rassemblent plusieurs partis, des députés y sont
rattachés sans en être membres, et certains partis n'ont aucun groupe.

**Règle** : on affiche le groupe, nommé comme tel, avec une page de composition.
Aucune traduction silencieuse groupe → parti. Une équivalence approximative présentée
comme un fait est une fausse information.

### 3.2 Appartenance datée

Un député change de groupe en cours de législature. Un vote de 2024 doit être compté
avec le groupe du député **à la date du vote**, jamais avec son groupe actuel.

**Règle** : l'appartenance est stockée avec ses dates de validité ; toute agrégation de
votes joint sur l'appartenance à la date du scrutin. Une jointure sur l'appartenance
courante réécrit l'histoire des votes.

### 3.3 Candidat ≠ député

Beaucoup de personnalités en lice n'ont jamais siégé à l'Assemblée, ou plus depuis
plusieurs législatures. Leur absence de votes n'est pas une information sur elles.

**Règle** : le site ne présente jamais « la position de X ». Il présente « les votes du
groupe Y ». La limite est affichée sur les pages concernées, pas enfouie dans une FAQ.

## 4. Architecture (vue d'ensemble)

```
Ingestion continue AN (dossiers + scrutins + députés + appartenances)
  → Base de faits horodatés
  → Thématisation (rattachement texte débattu → familles, hérité par scrutins et dossiers)
  → Pages structurées : thème × groupe × période
  → Chat (couche de routage, sans production de chiffres)
```

- L'ingestion tourne **au fil de l'eau**, sans validation humaine préalable :
  la donnée est brute et sourcée, il n'y a rien à valider éditorialement.
- Le point de contrôle humain porte sur **la thématisation** (§5) et sur la
  **qualité de l'ingestion**, pas sur le contenu affiché.

## 5. Thématisation

Rattacher un texte à un ou plusieurs thèmes est **le seul endroit où un jugement
entre dans le produit**. Il est donc traité comme tel : critères explicites, résultat
inspectable, correction possible.

**Le porteur du thème est le texte débattu, pas le dossier** (décidé le 03/08/2026).
La source publie le lien vers le dossier de façon irrégulière *à l'intérieur d'un même
texte* : la proposition de loi relative au droit à l'aide à mourir porte 297 scrutins
avec dossier et 541 sans. Thématiser le dossier laisserait 541 votes du même texte hors
thème — la sélection éditoriale que §2 interdit. Le texte est extrait de l'objet du
scrutin par une règle déterministe : 8 428 scrutins sur 8 434 se regroupent en
322 textes. Un scrutin hérite des familles de son texte ; un dossier hérite du texte que
ses propres scrutins nomment, et n'est classé directement que s'il n'a aucun scrutin.

Familles cibles :

- Pouvoir d'achat / fiscalité
- Logement (loyers, accès propriété, locations courte durée)
- Travail / emploi (droit du travail, chômage, indépendants)
- Environnement / énergie (prix de l'énergie, transition, transports)
- Numérique (données, IA, réseaux sociaux, fraude en ligne)
- Santé / social (remboursements, accès aux soins, congés)
- Société / libertés (**terrain sensible : factuel uniquement**)
- Institutions / procédure (motions de censure, révisions, budget)

Règles :

- Un texte peut appartenir à **plusieurs** familles, trois au plus ; on ne force pas l'unicité.
- Le rattachement est **révisable** et son historique conservé.
- Un texte non rattaché reste **consultable** (liste dédiée), il n'est pas perdu.
- La méthode de rattachement est publiée sur le site.
- Le modèle **propose** et sa proposition est publiée avec la mention de son origine ;
  un humain peut arbitrer après coup. Le modèle ne voit que le libellé du texte.

## 6. Neutralité — règles dures

- **Faits uniquement** : quel texte, ce qu'il prévoit, qui a voté quoi, résultat, étape.
- **Aucune évaluation** du texte (bon/mauvais, juste/injuste, efficace/non).
- **Aucune évaluation d'un groupe** : pas de score de cohérence, de taux de présence
  présenté comme un mérite, ni d'agrégat qui se lit comme un classement.
- **Des nombres, pas d'adverbes** : « rejeté par 280 voix contre 250 » ✅ ;
  « massivement rejeté » ❌.
- **Positions attribuées et on-record uniquement** : exposé des motifs, interventions en
  séance, position de vote. Jamais de commentaire média ni de synthèse libre.
- **Lien vers la source officielle sur chaque chiffre affiché.**
- **Le LLM ne produit jamais un chiffre** (§8).

## 7. Traitement des votes

- **Répartition par groupe** (pour / contre / abstention / non-votants), chiffres bruts.
- **Position nominale par député** disponible au détail du scrutin — c'est le niveau de
  preuve qui rend le site vérifiable.
- **Trou de couverture, pas second régime** (vérifié le 02/08/2026) : tous les scrutins
  publiés par l'Assemblée portent le décompte nominatif. Les votes à main levée ne sont
  pas publiés *sans* répartition — ils sont **absents du jeu de données**. Le site ne peut
  donc rien dire d'eux. Cette lacune est **affichée comme telle** sur les pages concernées ;
  la silencer laisserait croire à une exhaustivité que la source ne permet pas.
- **Scrutins sans dossier** : conservés et exposés. **69 % des scrutins de la législature
  en cours ne portent aucun rattachement à un dossier** — les écarter viderait le site
  des deux tiers des votes.
- **Ne jamais inventer de chiffres.**

## 8. Interfaces

### 8.1 Pages structurées — socle

URL stables et partageables : thème, groupe, dossier, scrutin, député.
Servies directement depuis la base : aucun risque d'invention, indexables, citables.

### 8.2 Chat — surcouche

Le chat **route et cite**, il ne rédige pas les faits.

- Le LLM interprète la demande (thème + groupe + période) et sélectionne les données.
- Les chiffres affichés proviennent de la base, jamais du texte généré.
- Chaque réponse renvoie vers les pages et les sources officielles.

Motif : une seule hallucination sur un chiffre de vote détruit la crédibilité de
l'ensemble, et c'est irrattrapable en période électorale.

## 9. Défense méthodologique

Un site qui rapproche votes et partis avant une présidentielle sera contesté sur sa
méthode. La défense se construit en amont, pas après :

- Méthode de thématisation et de tri **publiée**.
- Chaque chiffre **traçable** jusqu'à la source officielle.
- Données sous **Licence Ouverte** (open data AN), réutilisation licite.
- Corrections **journalisées** publiquement.
- Aucun agrégat comparatif entre groupes qui ressemble à une note.

## 10. Sources de données

### Socle Assemblée nationale — `data.assemblee-nationale.fr`
Colonne vertébrale. XML / JSON. Jeux clés :
- **Scrutins** : position de vote de chaque député (→ §7). Priorité absolue.
- **Acteurs et organes** : députés, groupes, **mandats et appartenances datées** (→ §3.2).
- Dossiers législatifs (titres, dépôts, commissions, rapporteurs, dates).
- Comptes rendus de séance (positions attribuées).
- Amendements (auteur, objet, sort).

### Vie publique — `vie-publique.fr`
- « Panorama des lois » : résumés en langage clair et neutre → aide à la thématisation.
- **Depuis le 1ᵉʳ avril 2026, l'accès aux dossiers législatifs passe par Vie publique**
  (redirections depuis Légifrance). À intégrer dans le routage des sources.

### Légifrance via PISTE — API officielle
- Textes promulgués, droit en vigueur, Journal officiel. Auth OAuth, sandbox + prod.

### Couches tierces (vérifier la pérennité avant d'en dépendre)
- **CIVIX** : agrège scrutins, votes individuels, groupes, dossiers, sans interprétation.
- **Tricoteuses** (`@tricoteuses/assemblee`) : données AN nettoyées.
- **NosDéputés.fr / ParlAPI.fr** (Regards Citoyens) : recherche plein texte.

### Sénat — `data.senat.fr`
**Hors périmètre initial**, choix assumé : l'Assemblée est élue au suffrage universel
direct, ses votes sont les plus lisibles pour l'usage visé. À rouvrir si besoin.

## 11. Stack

- **Backend Rust** — Axum, Tokio, sqlx, serde. **Frontend React/TypeScript** (Vite, SPA).
- **Base** : Neon (Postgres serverless).
- Clean Architecture + DDD, découpage par use case. Voir `CLAUDE.md`.
- LLM en BYOK, cantonné à la thématisation et au routage du chat — jamais à la
  production de chiffres.

## 12. Feuille de route

- [x] **Phase 0** — Repo, ingestion des dossiers législatifs AN.
- [x] **Phase 1** — Enrichissement du domaine dossier (étape, initiateurs, commission,
      documents, résumé) + curation.
- [x] **Phase 2** — **Acteurs** : référentiel députés et groupes, appartenances datées (§3.2).
- [x] **Phase 3** — **Scrutins** : ingestion, positions nominales, répartition par groupe.
- [x] **Phase 4** — **Thématisation** : textes débattus, familles, méthode publiée (§5).
- [ ] **Phase 5** — Pages publiques thème × groupe × période.
- [ ] **Phase 6** — Chat de routage (§8.2).
- [ ] **Phase 7** — Page méthodologie, journal des corrections (§9).

## Non-objectifs

- Pas de note, de classement ni de score attribué à un groupe ou à une personne.
- Pas d'opinion, pas de conclusion éditoriale, pas de recherche de viralité.
- Pas de traduction groupe → parti sans base factuelle.
- Pas de prédiction électorale, pas de comparaison de programmes.
- Pas de chiffre produit par un LLM.

---

*Note de pivot (août 2026) : ce document remplace la version « générateur de contenus
courts Instagram/TikTok ». Le socle d'ingestion et le domaine législatif sont conservés ;
la couche de production éditoriale (short-list hebdomadaire, scripts, posts) est
abandonnée. Voir `todo/archive/` pour les spécifications devenues caduques — dont
`SPEC-scoring-llm.md`, archivée le 03/08/2026 : sa grille faisait noter le LLM de 0 à 10,
ce que §8 interdit, et ses critères (impact concret, résonance actu, vulgarisabilité)
appartenaient au produit d'avant le pivot.*
