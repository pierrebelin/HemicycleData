use crate::application::ports::actor_repository::ActorRepository;
use crate::application::ports::amendment_repository::{
    AmendmentPageRequest, AmendmentRepository, AmendmentSummary, DossierAmendmentCoverage,
};
use crate::application::ports::RepositoryError;
use crate::domain::actor::{ActorDirectory, ActorUid, GroupUid};

/// Un amendement pret a afficher: la ligne stockee, plus ce que le referentiel
/// permet de nommer.
///
/// Les noms ne sont pas denormalises en base, ils se resolvent ici — comme les
/// positions nominales d'un scrutin. Un acteur absent du referentiel garde son
/// identifiant et rien n'est devine (ACTEURS RM-04).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmendmentView {
    pub summary: AmendmentSummary,
    pub author_name: Option<String>,
    pub author_official_url: Option<String>,
    pub author_group_label: Option<String>,
    pub author_group_abbrev: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DossierAmendments {
    pub items: Vec<AmendmentView>,
    pub total: i64,
    pub coverage: DossierAmendmentCoverage,
}

/// CU-02 — Consulter les amendements d'un dossier.
pub struct BrowseDossierAmendments<'a> {
    amendments: &'a dyn AmendmentRepository,
    actors: &'a dyn ActorRepository,
}

impl<'a> BrowseDossierAmendments<'a> {
    pub fn new(amendments: &'a dyn AmendmentRepository, actors: &'a dyn ActorRepository) -> Self {
        Self { amendments, actors }
    }

    pub async fn execute(
        &self,
        dossier_uid: &str,
        page: &AmendmentPageRequest,
    ) -> Result<DossierAmendments, RepositoryError> {
        let page = self.amendments.by_dossier(dossier_uid, page).await?;
        let coverage = self.amendments.dossier_coverage(dossier_uid).await?;

        // Le referentiel n'est charge que pour les auteurs de la page affichee.
        let uids: Vec<ActorUid> = page
            .items
            .iter()
            .filter_map(|item| item.author_actor_uid.as_ref())
            .filter_map(|raw| ActorUid::new(raw.clone()).ok())
            .collect();
        let directory = self.actors.load_directory_for(&uids).await?;

        let items = page
            .items
            .into_iter()
            .map(|summary| view_of(summary, &directory))
            .collect();

        Ok(DossierAmendments {
            items,
            total: page.total,
            coverage,
        })
    }
}

fn view_of(summary: AmendmentSummary, directory: &ActorDirectory) -> AmendmentView {
    let actor = summary
        .author_actor_uid
        .as_ref()
        .and_then(|raw| ActorUid::new(raw.clone()).ok())
        .and_then(|uid| directory.actor(&uid).cloned());

    // Le groupe affiche est celui stocke — donc celui de la date de depot
    // (RM-02). Le referentiel ne sert qu'a lui donner son libelle, jamais a le
    // recalculer sur l'appartenance courante.
    let group = summary
        .author_group_uid
        .as_ref()
        .and_then(|raw| GroupUid::new(raw.clone()).ok())
        .and_then(|uid| directory.group(&uid));

    AmendmentView {
        author_name: actor.as_ref().map(|a| a.full_name()),
        author_official_url: actor.as_ref().and_then(|a| a.official_url()),
        author_group_label: group.map(|g| g.label().to_string()),
        author_group_abbrev: group.map(|g| g.abbrev().to_string()),
        summary,
    }
}
