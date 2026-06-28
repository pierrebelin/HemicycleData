use super::dossier::Score;

pub fn compute_score(title: &str, last_activity_label: &str) -> Score {
    let progress = score_progress(last_activity_label);
    let magnitude = score_magnitude(title);
    let total = ((progress as u16 * 2 + magnitude as u16 * 2) * 100 / 40) as u8;
    Score {
        progress,
        magnitude,
        total,
    }
}

fn score_progress(label: &str) -> u8 {
    let lower = label.to_lowercase();
    if lower.contains("promulgation") {
        10
    } else if lower.contains("vote solennel") || lower.contains("scrutin public") {
        9
    } else if lower.contains("adoption") || lower.contains("discussion en séance") {
        7
    } else if lower.contains("1ère lecture") || lower.contains("1ere lecture") {
        6
    } else if lower.contains("commission") {
        4
    } else if lower.contains("renvoi") {
        3
    } else if lower.contains("dépôt") || lower.contains("depot") {
        2
    } else {
        1
    }
}

fn score_magnitude(title: &str) -> u8 {
    let lower = title.to_lowercase();
    if lower.contains("finances")
        || lower.contains("financement")
        || lower.contains("budget")
    {
        10
    } else if lower.contains("réforme") || lower.contains("reforme") {
        7
    } else {
        3
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn promulgation_scores_highest_progress() {
        let score = compute_score("Texte lambda", "Promulgation de la loi");
        assert_eq!(score.progress, 10);
    }

    #[test]
    fn vote_solennel_scores_9() {
        let score = compute_score("Texte", "Vote solennel");
        assert_eq!(score.progress, 9);
    }

    #[test]
    fn depot_scores_low() {
        let score = compute_score("Texte", "Dépôt");
        assert_eq!(score.progress, 2);
    }

    #[test]
    fn plf_scores_max_magnitude() {
        let score = compute_score("Projet de loi de finances pour 2026", "Dépôt");
        assert_eq!(score.magnitude, 10);
    }

    #[test]
    fn reforme_scores_medium_magnitude() {
        let score = compute_score("Réforme des retraites", "Dépôt");
        assert_eq!(score.magnitude, 7);
    }

    #[test]
    fn ordinary_text_scores_low_magnitude() {
        let score = compute_score("Ratification d'un accord", "Dépôt");
        assert_eq!(score.magnitude, 3);
    }

    #[test]
    fn total_is_normalized_on_100() {
        let score = compute_score("Projet de loi de finances", "Promulgation de la loi");
        // (10*2 + 10*2) * 100 / 40 = 100
        assert_eq!(score.total, 100);

        let score = compute_score("Accord technique", "Dépôt");
        // (2*2 + 3*2) * 100 / 40 = 25
        assert_eq!(score.total, 25);
    }
}
