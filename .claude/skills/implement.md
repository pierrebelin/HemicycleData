# Skill : Implémenter du code

Skill à utiliser pour toute implémentation de fonctionnalité, correction de bug, ou ajout de code dans le projet. Encapsule l'architecture Clean Architecture + DDD en idiomes Rust.

## Structure du projet

```
src/
  main.rs                      // bootstrap + wiring des dépendances
  domain/
    {aggregate}/
      mod.rs
      {aggregate}.rs           // aggregate root
      value_objects.rs         // newtypes (champ privé + constructeur validant)
      errors.rs                // DomainError
  application/
    ports/
      {port}_repository.rs     // trait Repository (port)
    use_cases/
      {use_case}/
        mod.rs
        command.rs             // input DTO
        handler.rs             // struct UseCase<R> + tests
  infrastructure/
    persistence/
      pg_{entity}_repository.rs  // impl du port (sqlx)
    config.rs
  api/
    routes.rs
    handlers/{entity}_handlers.rs  // handlers Axum → use cases
    dto.rs                     // request/response models
```

## Patterns obligatoires

### Value objects → newtypes

Champ privé, constructeur validant, invariant garanti par le système de types.

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

### Aggregate root → struct à champs privés

L'encapsulation passe par la frontière de module. Les invariants vivent dans les méthodes.

### Ports → traits côté application

Définis dans `application/ports/`, implémentés dans `infrastructure/`. C'est le seam de test principal.

```rust
#[async_trait]
pub trait PostRepository: Send + Sync {
    async fn save(&self, post: &Post) -> Result<(), RepoError>;
    async fn by_id(&self, id: &PostId) -> Result<Option<Post>, RepoError>;
}
```

### Use cases → struct avec dépendances + méthode `execute`

Appelé directement depuis le handler Axum. Pas de mediator.

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

`AppState` clonable avec dépendances en `Arc<dyn Trait>` (dispatch dynamique, acceptable pour du code DB-bound).

## Anti-patterns — NE PAS faire

- **Pas de mediator** (type MediatR). Le use case est appelé directement depuis le handler.
- **Pas de trait par réflexe.** Trait seulement s'il y a polymorphisme ou seam de test (repositories, services externes LLM/image). Sinon, struct concrète.
- **Pas de couche de mapping type AutoMapper.** Mapping DTO ↔ domaine explicite et local (`impl From<...>`).

## Stratégie de tests

### Domaine : tests purs, zéro mock

Les entités et value objects n'ont aucune dépendance infra → tests synchrones, module `#[cfg(test)]` inline.

Si tester le domaine oblige à mocker quelque chose, c'est que de l'infra a fuité dedans.

### Use cases : fake in-memory, pas mockall

Tester le résultat observable (état persisté), pas la séquence d'appels.

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    struct InMemoryPostRepository {
        posts: Mutex<HashMap<PostId, Post>>,
    }

    #[async_trait]
    impl PostRepository for InMemoryPostRepository {
        async fn save(&self, post: &Post) -> Result<(), RepoError> {
            self.posts.lock().unwrap().insert(post.id().clone(), post.clone());
            Ok(())
        }
        async fn by_id(&self, id: &PostId) -> Result<Option<Post>, RepoError> {
            Ok(self.posts.lock().unwrap().get(id).cloned())
        }
    }
}
```

**Pattern clé** : garder un handle typé `Arc<InMemoryPostRepository>` pour les assertions, passer une copie coercée `Arc<dyn PostRepository>` au use case.

### Règle fake vs mock

- **Fake in-memory** pour les **ports d'état** (repositories) → on teste l'état résultant.
- **mockall** (`#[automock]`) uniquement pour les **ports d'effet** (`EventPublisher`, `NotificationSender`) où l'interaction est le contrat.

### Tests paramétrés

Utiliser `rstest` pour les tests paramétrés :

```rust
#[rstest]
#[case("", true)]
#[case("a", false)]
#[case(&"a".repeat(2201), true)]
fn validation_caption(#[case] input: &str, #[case] doit_echouer: bool) {
    assert_eq!(Caption::new(input.into()).is_err(), doit_echouer);
}
```

### Test data builders

Module `tests/support` avec des builders (`PostBuilder::draft().with_hashtags(...).build()`) pour des tests lisibles quand les invariants se complexifient.

## Frictions Rust à anticiper

- **Async traits sur `dyn`** : pour `Arc<dyn Repository>`, utiliser le crate `async_trait` (ou `trait-variant`).
- **Génériques vs `dyn`** : démarrer en `dyn` (câblage simple, un seul `AppState`). Réintroduire des génériques seulement si profilage le justifie.
- **`&self` + interior mutability** : les fakes qui mutent leur état nécessitent `Mutex<HashMap>` (`Mutex` plutôt que `RefCell` pour rester `Send + Sync`).
- **`Post` doit dériver `Clone`** pour les fakes (store renvoie des copies). Si on ne veut pas exposer en prod : `#[cfg_attr(test, derive(Clone))]`.
- **CORS** : back et front sont deux serveurs séparés → `CorsLayer` de tower-http dès le départ.

## Checklist avant de livrer du code

1. Le nouveau code respecte la structure de dossiers ci-dessus
2. Les value objects utilisent le pattern newtype avec validation
3. Les invariants métier sont dans le domaine, pas dans les handlers
4. Les use cases sont testés avec des fakes in-memory
5. Pas de trait inutile, pas de mediator, pas de mapping implicite
6. `cargo build` et `cargo test` passent
