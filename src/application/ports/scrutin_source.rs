use async_trait::async_trait;

use crate::domain::scrutin::Scrutin;

pub use super::SourceError;

/// Source des scrutins publics.
///
/// La source publie une archive complete par legislature: il n'existe pas de
/// requete par scrutin ni de flux incremental. RM-01 impose de tout prendre,
/// donc de tout charger.
#[async_trait]
pub trait ScrutinSource: Send + Sync {
    async fn fetch_scrutins(&self, legislature: u16) -> Result<Vec<Scrutin>, SourceError>;
}
