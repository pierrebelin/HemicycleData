//! Binaire jetable: enregistre des propositions de familles produites hors API.
//!
//! Chemin d'exception, pas de remplacement de CU-02. `ANTHROPIC_API_KEY` etant
//! absente, les familles sont produites par un modele dans une session d'outil,
//! puis recopiees ici. L'origine reste `proposal` et le modele est nomme tel
//! qu'il est: enregistrer cela en `human_arbitration` ferait mentir le site sur
//! sa propre methode (README.md §5, §9).
//!
//! Meme sequence de persistance que ProposeThemeFamilies: save_proposal,
//! replace_assignments, record_attempt.
//!
//! Usage:
//!   cargo run --bin seed_theme_proposals            # liste les textes cibles
//!   cargo run --bin seed_theme_proposals -- --apply # ecrit les propositions

use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use hemicycle_data::application::ports::theme_repository::{AttemptOutcome, ThemeRepository};
use hemicycle_data::domain::theme::{
    FamilyCode, ProposedFamily, SubjectRef, TextKey, ThemeProposal,
};
use hemicycle_data::infrastructure::persistence::pg_theme_repository::PgThemeRepository;
use sqlx::postgres::PgPoolOptions;
use sqlx::Row;

/// Nomme le producteur sans ambiguite. Ce n'est pas le chemin BYOK.
const MODEL: &str = "claude-opus-5 (session Claude Code, hors API)";
const PROMPT_VERSION: &str = "thematisation-v1-hors-api";

/// Nombre de textes cibles, les plus recemment votes.
const TARGET: i64 = 20;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let apply = std::env::args().any(|a| a == "--apply");
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new().max_connections(2).connect(&url).await?;

    let targets = most_recent_unassigned(&pool, TARGET).await?;
    println!("{} textes cibles (dernier vote decroissant)\n", targets.len());

    let table = classifications();
    let today = Utc::now().date_naive();
    let repository = PgThemeRepository::new(pool.clone());

    let mut written = 0;
    for (key, label, last_vote, scrutins) in &targets {
        if NO_FAMILY.contains(&key.as_str()) {
            println!("  [aucune famille] {last_vote} {scrutins:>4} sc. {label}");
            if apply {
                repository
                    .record_attempt(key, today, AttemptOutcome::NoFamily)
                    .await?;
                written += 1;
            }
            continue;
        }
        match table.get(key.as_str()) {
            None => println!("  [manquant] {last_vote} {scrutins:>4} sc. {label}"),
            Some(families) => {
                let codes: Vec<&str> = families.iter().map(|(c, _)| c.as_str()).collect();
                println!(
                    "  [{}] {last_vote} {scrutins:>4} sc. {label}",
                    codes.join(", ")
                );
                if apply {
                    write_proposal(&repository, key, families, today).await?;
                    written += 1;
                }
            }
        }
    }

    if apply {
        println!("\n{written} propositions ecrites, origine `proposal`, modele « {MODEL} ».");
    } else {
        println!("\nRelancer avec --apply pour ecrire.");
    }
    Ok(())
}

/// Textes sans rattachement courant, du plus recemment vote au plus ancien.
async fn most_recent_unassigned(
    pool: &sqlx::PgPool,
    limit: i64,
) -> Result<Vec<(TextKey, String, NaiveDate, i64)>, Box<dyn std::error::Error>> {
    let rows = sqlx::query(
        "SELECT t.text_key, t.label,
                max(s.scrutin_date) AS last_vote,
                count(*) AS scrutins
         FROM debated_texts t
         JOIN scrutin_debated_texts l ON l.text_key = t.text_key
         JOIN scrutins s ON s.uid = l.scrutin_uid
         WHERE NOT EXISTS (
             SELECT 1 FROM theme_assignments a
             WHERE a.subject_kind = 'text' AND a.subject_id = t.text_key
               AND a.closed_on IS NULL
         )
         GROUP BY t.text_key, t.label
         ORDER BY last_vote DESC, scrutins DESC
         LIMIT $1",
    )
    .bind(limit)
    .persistent(false)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| {
            (
                TextKey::from_raw(&r.get::<String, _>("text_key")),
                r.get::<String, _>("label"),
                r.get::<NaiveDate, _>("last_vote"),
                r.get::<i64, _>("scrutins"),
            )
        })
        .collect())
}

async fn write_proposal(
    repository: &PgThemeRepository,
    key: &TextKey,
    families: &[(FamilyCode, &'static str)],
    today: NaiveDate,
) -> Result<(), Box<dyn std::error::Error>> {
    let subject = SubjectRef::Text(key.clone());
    let proposed = families
        .iter()
        .map(|(family, justification)| ProposedFamily::new(*family, justification.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    // Le domaine applique RM-03 (trois au plus) et RM-05 (justification).
    let proposal = ThemeProposal::new(
        subject.clone(),
        proposed,
        MODEL.to_string(),
        PROMPT_VERSION.to_string(),
        today,
    )?;
    let assignments = proposal.into_assignments()?;

    repository.save_proposal(&proposal).await?;
    repository
        .replace_assignments(&subject, today, &assignments)
        .await?;
    repository
        .record_attempt(key, today, AttemptOutcome::Proposed)
        .await?;
    Ok(())
}

/// Textes qu'aucune famille du referentiel ne couvre. Enregistres en
/// `no_family`, consultables, distingues d'un echec (RM-01).
const NO_FAMILY: [&str; 1] = [
    "proposition de loi relative à l'organisation, à la gestion et au financement du sport professionnel (texte de la commission mixte paritaire)",
];

/// Rattachements produits a partir du seul libelle du texte (RM-04): aucun
/// resultat de vote, aucun groupe, aucun decompte consulte.
/// Justifications sans jugement ni chiffre (RM-05, RM-10, RM-11).
fn classifications() -> HashMap<&'static str, Vec<(FamilyCode, &'static str)>> {
    use FamilyCode::*;
    HashMap::from([
        (
            "projet de loi relatif à la protection des enfants",
            vec![
                (SocieteLibertes, "Le texte porte sur la protection des enfants, matière relevant des droits des personnes et de la justice."),
                (SanteSocial, "La protection de l'enfance relève de l'action sociale."),
            ],
        ),
        (
            "projet de loi visant à offrir des réponses immédiates aux phénomènes troublant l'ordre public, la sécurité et la tranquilité de nos concitoyens (texte de la commission mixte paritaire)",
            vec![(SocieteLibertes, "Le libellé porte sur l'ordre public et la sécurité.")],
        ),
        (
            "proposition de loi visant à moderniser la gestion du patrimoine immobilier de l'état (texte de la commission mixte paritaire)",
            vec![(PouvoirAchatFiscalite, "Le texte porte sur la gestion du patrimoine immobilier de l'État, matière relevant du budget de l'État.")],
        ),
        (
            "proposition de loi visant à protéger les mineurs des risques auxquels les expose l'utilisation des réseaux sociaux (texte de la commission mixte paritaire)",
            vec![
                (Numerique, "Le texte porte sur l'utilisation des réseaux sociaux."),
                (SocieteLibertes, "Il porte sur la protection des mineurs."),
            ],
        ),
        (
            "projet de loi d'urgence pour la protection et la souveraineté agricoles (texte de la commission mixte paritaire)",
            vec![(EnvironnementEnergie, "Le texte porte sur l'agriculture.")],
        ),
        (
            "projet de loi d'urgence pour la protection et la souveraineté agricoles) (texte de la commission mixte paritaire)",
            vec![(EnvironnementEnergie, "Le texte porte sur l'agriculture.")],
        ),
        (
            "proposition de loi pour une montagne vivante et souveraine (texte de la commission mixte paritaire)",
            vec![(EnvironnementEnergie, "Le texte porte sur les territoires de montagne, objet relevant de l'aménagement et de l'agriculture.")],
        ),
        (
            "proposition de loi visant à doter la france d'une stratégie nationale de lutte contre les maladies cardio-neuro-vasculaires (texte de la commission mixte paritaire)",
            vec![(SanteSocial, "Le texte porte sur une stratégie nationale relative à des maladies.")],
        ),
        (
            "proposition de loi relative au droit à l'aide à mourir",
            vec![
                (SocieteLibertes, "Le texte porte sur la fin de vie."),
                (SanteSocial, "Il porte sur l'accompagnement médical en fin de vie."),
            ],
        ),
        (
            "projet de loi visant à offrir des réponses immédiates aux phénomènes troublant l'ordre public, la sécurité et la tranquillité de nos concitoyens",
            vec![(SocieteLibertes, "Le libellé porte sur l'ordre public et la sécurité.")],
        ),
        (
            "projet de loi sur la justice criminelle et le respect des victimes",
            vec![(SocieteLibertes, "Le texte porte sur la justice criminelle et la place des victimes.")],
        ),
        (
            "projet de loi sur la justice criminelle et le respect des victimes (texte de la commission mixte paritaire)",
            vec![(SocieteLibertes, "Le texte porte sur la justice criminelle et la place des victimes.")],
        ),
        (
            "projet de loi organique relatif au renforcement des juridictions criminelles",
            vec![
                (SocieteLibertes, "Le texte porte sur les juridictions criminelles."),
                (InstitutionsProcedure, "Texte organique portant sur l'organisation de juridictions."),
            ],
        ),
        (
            "projet de loi organique relatif au renforcement des juridictions criminelles (texte de la commission mixte paritaire)",
            vec![
                (SocieteLibertes, "Le texte porte sur les juridictions criminelles."),
                (InstitutionsProcedure, "Texte organique portant sur l'organisation de juridictions."),
            ],
        ),
        (
            "proposition de loi visant à reconnaître une présomption de légitime défense pour les forces de l'ordre, dans l'exercice de leurs fonctions",
            vec![(SocieteLibertes, "Le texte porte sur les règles de légitime défense applicables aux forces de l'ordre.")],
        ),
        (
            "proposition de loi visant à reconnaître une présomption de légitime défense pour les forces de l'ordre, dans l'exercice de leurs fonctions)",
            vec![(SocieteLibertes, "Le texte porte sur les règles de légitime défense applicables aux forces de l'ordre.")],
        ),
        (
            "motion de censure déposée en application de l'article 49, alinéa 2, de la constitution par mmes cyrielle chatelain, nadège abomangoli et 56 députés",
            vec![(InstitutionsProcedure, "Motion de censure, procédure prévue par la Constitution.")],
        ),
        (
            "proposition de loi visant à assurer le droit de chaque enfant à être assisté d'un avocat dans le cadre d'une mesure d'assistance éducative et de protection de l'enfance",
            vec![
                (SocieteLibertes, "Le texte porte sur l'assistance d'un avocat, matière relevant de la justice."),
                (SanteSocial, "Il porte sur les mesures d'assistance éducative et de protection de l'enfance, qui relèvent de l'action sociale."),
            ],
        ),
        (
            "projet de loi actualisant la programmation militaire pour les années 2024 à 2030 et portant diverses dispositions intéressant la défense (texte de la commission mixte paritaire)",
            vec![(PouvoirAchatFiscalite, "Loi de programmation fixant une trajectoire budgétaire de l'État sur plusieurs années.")],
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use hemicycle_data::domain::theme::DebatedText;

    /// Les cles de la table sont des cles normalisees (RM-02): une cle qui ne
    /// survit pas a la normalisation ne retrouverait jamais son texte.
    #[test]
    fn every_key_is_already_normalised() {
        for key in classifications().keys().chain(NO_FAMILY.iter()) {
            assert_eq!(TextKey::from_raw(key).as_str(), *key, "cle non normalisee: {key}");
        }
    }

    /// RM-03 et RM-05 sont portees par le domaine, mais une table fausse ne doit
    /// pas atteindre la base pour s'en apercevoir.
    #[test]
    fn every_entry_holds_one_to_three_justified_families() {
        for (key, families) in classifications() {
            assert!(!families.is_empty(), "{key}");
            assert!(families.len() <= 3, "{key}");
            for (_, justification) in &families {
                assert!(!justification.trim().is_empty(), "{key}");
            }
        }
    }

    /// Un texte ne peut pas etre a la fois rattache et sans famille.
    #[test]
    fn no_family_and_classified_do_not_overlap() {
        let table = classifications();
        for key in NO_FAMILY {
            assert!(!table.contains_key(key), "{key}");
        }
    }

    /// La table ne rattache que des textes reels: le libelle doit se re-extraire
    /// de lui-meme par la regle publiee.
    #[test]
    fn every_key_is_a_text_the_extraction_rule_recognises() {
        for key in classifications().keys() {
            assert!(DebatedText::from_subject(key).is_some(), "{key}");
        }
    }
}
