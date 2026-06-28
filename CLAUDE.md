# hémicycle.data

Outil de veille sur l'activité du Parlement français (Assemblée nationale + Sénat)
qui sélectionne automatiquement les sujets pertinents et prépare la matière de
vidéos courtes (Instagram / TikTok), avec validation manuelle avant production.

## Stack

- .NET / C# (toutes les sources sont language-agnostic via REST/XML/JSON)
- LLM en BYOK (reformulation/scoring contraints, pas de génération libre du fond)

## Architecture

Ingestion continue → Journal interne de la semaine → Scoring → Short-list (3-5)
→ [VALIDATION MANUELLE] → Génération scripts → Production vidéo

## Sources de données

- data.assemblee-nationale.fr (scrutins, dossiers, comptes rendus, amendements, agenda)
- vie-publique.fr (panorama des lois, résumés vulgarisés)
- Légifrance via PISTE (textes promulgués, JO)
- data.senat.fr (navette complète)
- Couches tierces : CIVIX, Tricoteuses, NosDéputés.fr / ParlAPI.fr

## Conventions

- Langue du code : anglais
- Langue de la documentation et des communications : français
- Neutralité stricte dans tout contenu généré (voir PROJECT.md §6)
