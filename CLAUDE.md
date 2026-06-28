# Générateur de posts Instagram

Application web de génération de posts Instagram. Backend Rust (API REST), frontend SPA React/TypeScript.

## Stack

### Backend
- **Rust** — Axum 0.8.x, Tokio, sqlx (requêtes vérifiées à la compilation), serde, tower-http
- **Base de données** : Neon (serverless Postgres) — connexion par connection string, pas de couche BaaS

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

## Conventions

- Langue du code : anglais
- Langue de la documentation et des communications : français
- Pas de mediator, pas de trait par réflexe, pas de mapping implicite type AutoMapper
- Tests : fakes in-memory pour les ports d'état, mockall uniquement pour les ports d'effet
- Value objects = newtypes avec constructeur validant
