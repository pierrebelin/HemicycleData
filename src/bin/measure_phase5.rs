//! Binaire jetable: mesure sur la base reelle les faits qui contraignent les
//! pages theme x groupe x periode (Phase 5), hors thematisation.
//!
//! Les nombres produits ici alimentent la section « Hypotheses » de
//! todo/SPEC-PAGES-THEME-GROUPE.md. Aucun n'est estime.
//!
//! Usage: cargo run --bin measure_phase5

use sqlx::postgres::PgPoolOptions;
use sqlx::{PgPool, Row};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")?;
    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&url)
        .await?;

    groups(&pool).await?;
    period(&pool).await?;
    volumetry(&pool).await?;
    tally_shape(&pool).await?;
    coverage(&pool).await?;

    Ok(())
}

/// Une requete = une section. `.persistent(false)`: le pooler Neon garde les
/// instructions preparees au-dela du processus (voir CLAUDE.md).
async fn rows(pool: &PgPool, sql: &str) -> Result<Vec<sqlx::postgres::PgRow>, sqlx::Error> {
    sqlx::query(sql).persistent(false).fetch_all(pool).await
}

async fn groups(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== M1. Groupes presents dans les repartitions de scrutin ===");
    let sql = "
        SELECT t.group_uid,
               COALESCE(g.abbrev, '(hors referentiel)') AS abbrev,
               COALESCE(g.label, '(hors referentiel)') AS label,
               COUNT(*) AS tally_lines,
               MIN(s.scrutin_date) AS first_date,
               MAX(s.scrutin_date) AS last_date,
               MIN(t.member_count) AS min_members,
               MAX(t.member_count) AS max_members
        FROM scrutin_group_tallies t
        JOIN scrutins s ON s.uid = t.scrutin_uid
        LEFT JOIN parliamentary_groups g ON g.uid = t.group_uid
        GROUP BY t.group_uid, g.abbrev, g.label
        ORDER BY tally_lines DESC";
    for r in rows(pool, sql).await? {
        println!(
            "{:<14} {:<10} lignes={:>5} du {} au {} effectif {:?}..{:?}  | {}",
            r.get::<String, _>("group_uid"),
            r.get::<String, _>("abbrev"),
            r.get::<i64, _>("tally_lines"),
            r.get::<chrono::NaiveDate, _>("first_date"),
            r.get::<chrono::NaiveDate, _>("last_date"),
            r.get::<Option<i16>, _>("min_members"),
            r.get::<Option<i16>, _>("max_members"),
            r.get::<String, _>("label"),
        );
    }

    println!("\n=== M2. Lignes de groupe par scrutin ===");
    let sql = "
        SELECT lines, COUNT(*) AS scrutins
        FROM (SELECT scrutin_uid, COUNT(*) AS lines
              FROM scrutin_group_tallies GROUP BY scrutin_uid) x
        GROUP BY lines ORDER BY lines";
    for r in rows(pool, sql).await? {
        println!(
            "  {} lignes -> {} scrutins",
            r.get::<i64, _>("lines"),
            r.get::<i64, _>("scrutins")
        );
    }
    Ok(())
}

async fn period(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== M3. Periode: bornes, sessions ===");
    let sql = "
        SELECT MIN(scrutin_date) AS first_date, MAX(scrutin_date) AS last_date,
               COUNT(DISTINCT session_ref) AS sessions,
               COUNT(*) FILTER (WHERE session_ref IS NULL) AS without_session,
               COUNT(DISTINCT date_trunc('month', scrutin_date)) AS months
        FROM scrutins";
    let r = &rows(pool, sql).await?[0];
    println!(
        "  du {} au {} | sessions distinctes={} | sans session={} | mois couverts={}",
        r.get::<chrono::NaiveDate, _>("first_date"),
        r.get::<chrono::NaiveDate, _>("last_date"),
        r.get::<i64, _>("sessions"),
        r.get::<i64, _>("without_session"),
        r.get::<i64, _>("months"),
    );

    let sql = "
        SELECT session_ref, COUNT(*) AS scrutins,
               MIN(scrutin_date) AS first_date, MAX(scrutin_date) AS last_date
        FROM scrutins GROUP BY session_ref ORDER BY first_date";
    for r in rows(pool, sql).await? {
        println!(
            "  {:<28} {:>5} scrutins  {} -> {}",
            r.get::<Option<String>, _>("session_ref")
                .unwrap_or_else(|| "(aucune)".into()),
            r.get::<i64, _>("scrutins"),
            r.get::<chrono::NaiveDate, _>("first_date"),
            r.get::<chrono::NaiveDate, _>("last_date"),
        );
    }
    Ok(())
}

async fn volumetry(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== M4. Scrutins par texte debattu (proxy d'une page de theme) ===");
    let sql = "
        SELECT COUNT(*) AS texts,
               MAX(n) AS max_scrutins,
               ROUND(AVG(n), 1)::float8 AS avg_scrutins,
               PERCENTILE_DISC(0.5) WITHIN GROUP (ORDER BY n) AS median,
               SUM(n) FILTER (WHERE n >= 100)::int8 AS scrutins_in_big_texts,
               COUNT(*) FILTER (WHERE n >= 100) AS texts_over_100
        FROM (SELECT text_key, COUNT(*) AS n
              FROM scrutin_debated_texts GROUP BY text_key) x";
    let r = &rows(pool, sql).await?[0];
    println!(
        "  textes={} | max={} | moyenne={} | mediane={} | textes>=100 scrutins: {} (portant {} scrutins)",
        r.get::<i64, _>("texts"),
        r.get::<i64, _>("max_scrutins"),
        r.get::<f64, _>("avg_scrutins"),
        r.get::<i64, _>("median"),
        r.get::<i64, _>("texts_over_100"),
        r.get::<Option<i64>, _>("scrutins_in_big_texts").unwrap_or(0),
    );

    println!("\n  Les 10 textes les plus votes:");
    let sql = "
        SELECT d.label, COUNT(*) AS n,
               MIN(s.scrutin_date) AS first_date, MAX(s.scrutin_date) AS last_date
        FROM scrutin_debated_texts sd
        JOIN debated_texts d ON d.text_key = sd.text_key
        JOIN scrutins s ON s.uid = sd.scrutin_uid
        GROUP BY d.label ORDER BY n DESC LIMIT 10";
    for r in rows(pool, sql).await? {
        let label: String = r.get("label");
        println!(
            "  {:>4} scrutins  {} -> {}  {}",
            r.get::<i64, _>("n"),
            r.get::<chrono::NaiveDate, _>("first_date"),
            r.get::<chrono::NaiveDate, _>("last_date"),
            label.chars().take(80).collect::<String>(),
        );
    }

    println!("\n=== M5. Types de scrutin: ce que porte reellement un texte ===");
    let sql = "
        SELECT ballot_type_label, outcome_label, COUNT(*) AS n
        FROM scrutins GROUP BY ballot_type_label, outcome_label ORDER BY n DESC";
    for r in rows(pool, sql).await? {
        println!(
            "  {:<28} {:<10} {:>5}",
            r.get::<String, _>("ballot_type_label"),
            r.get::<String, _>("outcome_label"),
            r.get::<i64, _>("n"),
        );
    }

    println!("\n  Objets par nature (l'ensemble du texte vs amendement/article):");
    let sql = "
        SELECT CASE
                 WHEN subject ILIKE '%amendement%' THEN 'amendement'
                 WHEN subject ILIKE 'l''ensemble%' THEN 'ensemble du texte'
                 WHEN subject ILIKE '%article%' THEN 'article'
                 WHEN subject ILIKE '%motion%' THEN 'motion'
                 ELSE 'autre'
               END AS nature,
               COUNT(*) AS n
        FROM scrutins GROUP BY nature ORDER BY n DESC";
    for r in rows(pool, sql).await? {
        println!(
            "  {:<20} {:>5}",
            r.get::<String, _>("nature"),
            r.get::<i64, _>("n")
        );
    }
    Ok(())
}

async fn tally_shape(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== M6. Origine des repartitions et positions majoritaires ===");
    let sql = "SELECT origin, COUNT(*) AS n FROM scrutin_group_tallies GROUP BY origin";
    for r in rows(pool, sql).await? {
        println!(
            "  {:<16} {:>7}",
            r.get::<String, _>("origin"),
            r.get::<i64, _>("n")
        );
    }
    let sql = "
        SELECT COALESCE(majority_position, '(aucune)') AS pos, COUNT(*) AS n
        FROM scrutin_group_tallies GROUP BY pos ORDER BY n DESC";
    for r in rows(pool, sql).await? {
        println!(
            "  majoritaire {:<20} {:>7}",
            r.get::<String, _>("pos"),
            r.get::<i64, _>("n")
        );
    }

    println!("\n  Croisement origine x position majoritaire (H7):");
    let sql = "
        SELECT origin, (majority_position IS NULL) AS no_majority, COUNT(*) AS n
        FROM scrutin_group_tallies GROUP BY origin, no_majority ORDER BY n DESC";
    for r in rows(pool, sql).await? {
        println!(
            "  {:<16} sans position majoritaire={:<6} {:>7}",
            r.get::<String, _>("origin"),
            r.get::<bool, _>("no_majority"),
            r.get::<i64, _>("n"),
        );
    }

    println!("\n  Lignes de groupe sans aucun votant (effectif nul ou groupe absent):");
    let sql = "
        SELECT COUNT(*) AS empty_lines
        FROM scrutin_group_tallies
        WHERE votes_for = 0 AND votes_against = 0 AND abstentions = 0
          AND not_voting = 0 AND voluntary_not_voting = 0";
    let r = &rows(pool, sql).await?[0];
    println!("  {} lignes a zero", r.get::<i64, _>("empty_lines"));

    println!("\n=== M7. Effectif d'un groupe: varie-t-il dans le temps ? ===");
    let sql = "
        SELECT t.group_uid, COALESCE(g.abbrev, t.group_uid) AS abbrev,
               COUNT(DISTINCT t.member_count) AS distinct_sizes,
               MIN(t.member_count) AS min_size, MAX(t.member_count) AS max_size
        FROM scrutin_group_tallies t
        LEFT JOIN parliamentary_groups g ON g.uid = t.group_uid
        GROUP BY t.group_uid, g.abbrev
        ORDER BY distinct_sizes DESC LIMIT 15";
    for r in rows(pool, sql).await? {
        println!(
            "  {:<10} {} effectifs distincts, de {:?} a {:?}",
            r.get::<String, _>("abbrev"),
            r.get::<i64, _>("distinct_sizes"),
            r.get::<Option<i16>, _>("min_size"),
            r.get::<Option<i16>, _>("max_size"),
        );
    }
    Ok(())
}

async fn coverage(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== M8. Thematisation: etat au moment de la mesure ===");
    let sql = "
        SELECT (SELECT COUNT(*) FROM debated_texts) AS texts,
               (SELECT COUNT(*) FROM theme_assignments WHERE closed_on IS NULL) AS current_assignments,
               (SELECT COUNT(*) FROM theme_proposals) AS proposals,
               (SELECT COUNT(*) FROM scrutin_debated_texts) AS scrutins_with_text,
               (SELECT COUNT(*) FROM scrutins) AS scrutins";
    let r = &rows(pool, sql).await?[0];
    println!(
        "  textes={} | rattachements courants={} | propositions={} | scrutins avec texte={}/{}",
        r.get::<i64, _>("texts"),
        r.get::<i64, _>("current_assignments"),
        r.get::<i64, _>("proposals"),
        r.get::<i64, _>("scrutins_with_text"),
        r.get::<i64, _>("scrutins"),
    );

    println!("\n=== M9. Positions nominales rattachables a un groupe ===");
    let sql = "
        SELECT COUNT(*) AS positions,
               COUNT(*) FILTER (WHERE group_uid IS NULL) AS without_group,
               COUNT(DISTINCT actor_uid) AS actors,
               COUNT(DISTINCT group_uid) AS groups
        FROM scrutin_votes";
    let r = &rows(pool, sql).await?[0];
    println!(
        "  positions={} | sans groupe={} | acteurs={} | groupes={}",
        r.get::<i64, _>("positions"),
        r.get::<i64, _>("without_group"),
        r.get::<i64, _>("actors"),
        r.get::<i64, _>("groups"),
    );

    println!("\n  Acteurs ayant vote sous plus d'un groupe sur la legislature:");
    let sql = "
        SELECT COUNT(*) AS movers FROM (
            SELECT actor_uid FROM scrutin_votes WHERE group_uid IS NOT NULL
            GROUP BY actor_uid HAVING COUNT(DISTINCT group_uid) > 1) x";
    let r = &rows(pool, sql).await?[0];
    println!("  {} acteurs", r.get::<i64, _>("movers"));
    Ok(())
}
