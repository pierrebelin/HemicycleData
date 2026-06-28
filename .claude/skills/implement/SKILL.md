---
name: implement
description: Implémenter du code dans le projet (fonctionnalité, correction, refacto). Applique les patterns Clean Architecture + DDD en Rust idiomatique. Utiliser pour toute demande d'implémentation, ajout de code, ou correction de bug.
allowed-tools: Bash(cargo *) Read Edit Write
---

# Implémenter du code

## Structure du projet

```
src/
  main.rs                         # bootstrap + wiring
  domain/{aggregate}/
    mod.rs
    {aggregate}.rs                # aggregate root
    value_objects.rs              # newtypes (champ privé + constructeur validant)
    errors.rs                     # DomainError
  application/
    ports/{port}_repository.rs    # traits (ports)
    use_cases/{use_case}/
      mod.rs
      command.rs                  # input DTO
      handler.rs                  # struct UseCase<R> + tests
  infrastructure/
    persistence/pg_{entity}_repository.rs
    config.rs
  api/
    routes.rs
    handlers/{entity}_handlers.rs
    dto.rs
```

## Patterns obligatoires

### Value objects — newtypes

```rust
pub struct Caption(String);

impl Caption {
    pub fn new(raw: String) -> Result<Self, DomainError> {
        if raw.chars().count() > 2200 {
            return Err(DomainError::CaptionTooLong);
        }
        Ok(Self(raw))
    }
    pub fn as_str(&self) -> &str { &self.0 }
}
```

### Ports — traits côté application

```rust
#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn save(&self, post: &Post) -> Result<(), RepoError>;
    async fn by_id(&self, id: &PostId) -> Result<Option<Post>, RepoError>;
}
```

### Use cases — struct + `execute`, pas de mediator

```rust
pub struct CreatePost<R: PostRepository> {
    repo: R,
}

impl<R: PostRepository> CreatePost<R> {
    pub async fn execute(&self, cmd: CreatePostCommand) -> Result<PostId, AppError> {
        let caption = Caption::new(cmd.caption)?;
        let post = Post::draft(caption);
        self.repo.save(&post).await?;
        Ok(post.id().clone())
    }
}
```

### Wiring

`AppState` clonable avec dépendances en `Arc<dyn Trait>`.

## Anti-patterns — NE PAS faire

- Pas de mediator (type MediatR)
- Pas de trait par réflexe — seulement si polymorphisme ou seam de test
- Pas de couche de mapping implicite — mapping DTO ↔ domaine explicite (`impl From<...>`)

## Tests

- **Domaine** : tests purs inline `#[cfg(test)]`, zéro mock. Si mocker = infra qui fuite.
- **Use cases** : fake in-memory (`Mutex<HashMap>`) pour les ports d'état. Tester l'état persisté, pas la séquence d'appels.
- **mockall** uniquement pour les ports d'effet (`EventPublisher`, `NotificationSender`).
- **rstest** pour les tests paramétrés.
- Garder un handle typé `Arc<InMemoryRepo>` pour les assertions, passer `Arc<dyn Repo>` au use case.

```rust
#[cfg(test)]
mod tests {
    struct InMemoryPostRepository {
        posts: Mutex<HashMap<PostId, Post>>,
    }

    #[async_trait]
    impl PostRepository for InMemoryPostRepository { /* ... */ }
}
```

## Frictions Rust

- Async traits sur `dyn` : utiliser `async_trait` ou `trait-variant`
- Démarrer en `dyn`, génériques seulement si profilage le justifie
- Fakes : `Mutex<HashMap>` (pas `RefCell`) pour rester `Send + Sync`
- `Post` : `#[cfg_attr(test, derive(Clone))]` si besoin pour les fakes
- CORS : `CorsLayer` de tower-http dès le départ

## Checklist

1. Structure de dossiers respectée
2. Value objects = newtypes avec validation
3. Invariants métier dans le domaine, pas dans les handlers
4. Use cases testés avec fakes in-memory
5. Pas de trait inutile, pas de mediator, pas de mapping implicite
6. `cargo build` et `cargo test` passent
