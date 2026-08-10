# hémicycle.data — Transparence des votes parlementaires

Site de consultation des votes de l'Assemblée nationale, par thème et par groupe parlementaire, en vue de 2027. Backend Rust (API REST), frontend SPA React/TypeScript.

**Lire `README.md` avant toute décision produit** — il porte les règles non négociables : exhaustivité (§2), groupe ≠ parti et appartenance datée (§3), neutralité (§6), aucun chiffre produit par un LLM (§8).

Le produit a pivoté en août 2026 : l'ancienne cible « génération de posts Instagram / TikTok » est abandonnée. Le socle d'ingestion et le domaine législatif sont conservés. Du code et de l'interface portent encore le vocabulaire éditorial d'avant (curation, suggestions, short-list) — c'est de la dette, pas une intention.

## Stack

### Backend
- **Rust** — Axum 0.8.x, Tokio, sqlx (requêtes vérifiées à la compilation), serde, tower-http
- **Base de données** : PostgreSQL local sur le VPS — connexion par connection string, pas de couche BaaS

### Frontend
- **Vite + React + TypeScript** — SPA sans SSR (le backend est en Rust, pas de couche backend JS)
- **TanStack Query** pour la communication API
- **Tailwind** pour le style
- Types TS à terme générés depuis les structs Rust via `ts-rs`

## Architecture

Clean Architecture + DDD. Dépendances pointant vers l'intérieur. Couche application découpée par use case (vertical slicing).

```
src/
  main.rs
  domain/{aggregate}/          → aggregate roots, value objects, erreurs domaine
  application/ports/           → traits (ports)
  application/use_cases/{uc}/  → command, handler, tests
  infrastructure/persistence/  → impls sqlx des ports
  api/                         → routes Axum, handlers, DTOs
```

Le skill `.claude/skills/implement.md` contient les patterns détaillés et exemples de code.

## Commandes

```bash
cargo build                    # compilation
cargo test                     # tests
cargo run                      # lancer le serveur
```

## Dépôt public — règles de confidentialité

Ce dépôt a vocation à être public. Tout commit est définitif : réécrire l'historique après une fuite ne suffit pas, un secret poussé est un secret à révoquer.

**Ne jamais versionner :**
- Secrets et identifiants : `DATABASE_URL` réel, `ANTHROPIC_API_KEY`, `ADMIN_TOKEN_SECRET`, clés SSH/TLS, tokens GitHub ou npm. Ils vivent dans `.env` en local (ignoré), dans `/home/hemicycle/shared/.env` en production, dans les *GitHub Secrets* pour la CI.
- Infrastructure nominative : IP du VPS, empreintes d'hôtes, chemins absolus de la machine de dev (`/Users/...`). La documentation utilise des placeholders — `<IP_DU_VPS>` — et rien d'autre.
- Données personnelles : adresses e-mail privées, dumps de base, exports contenant autre chose que de la donnée publique de l'Assemblée nationale.
- Artefacts locaux : `.DS_Store`, `target/`, `node_modules/`, `dist/`.

**Toujours :**
- Un nouveau paramètre sensible s'ajoute à `.env.example` avec une valeur factice, jamais avec la vraie.
- Les valeurs de configuration se lisent via `std::env::var`, jamais en dur dans le code ni dans un test.
- Avant d'ouvrir une PR, vérifier `git diff` : pas de `sk-ant-`, pas de `postgresql://user:motdepasse@`, pas d'IP, pas de fichier `.env`.
- La documentation de déploiement (`todo/SPEC-DEPLOIEMENT.md`) décrit la procédure, jamais l'instance : commandes et placeholders, pas d'hôte réel.

## Conventions

- Langue du code : anglais
- Langue de la documentation et des communications : français
- Pas de mediator, pas de trait par réflexe, pas de mapping implicite type AutoMapper
- Tests : fakes in-memory pour les ports d'état, mockall uniquement pour les ports d'effet
- Value objects = newtypes avec constructeur validant
