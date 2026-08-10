//! Normalisation des textes HTML publies dans les jeux de donnees de l'Assemblee.

/// Decode les references HTML et retire le balisage present dans un texte
/// source, en conservant les retours a la ligne utiles a la lecture.
///
/// Cette normalisation est faite avant la persistance afin que toutes les
/// lectures de la base reçoivent le meme texte (README.md §6, RM-03).
pub fn normalize_source_text(raw: &str) -> String {
    let without_markup = strip_markup(raw);
    let decoded = decode_entities(&without_markup);

    let mut normalized = decoded.replace("\r\n", "\n").replace('\r', "\n");
    while normalized.contains("\n\n\n") {
        normalized = normalized.replace("\n\n\n", "\n\n");
    }
    normalized.trim().to_string()
}

fn strip_markup(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut cursor = 0;

    while let Some(relative_start) = raw[cursor..].find('<') {
        let start = cursor + relative_start;
        output.push_str(&raw[cursor..start]);

        let Some(relative_end) = raw[start..].find('>') else {
            output.push_str(&raw[start..]);
            cursor = raw.len();
            break;
        };
        let end = start + relative_end;
        let tag = raw[start..=end].trim().to_ascii_lowercase();

        let br_suffix = tag.as_bytes().get(3).copied();
        if tag.starts_with("<br") && matches!(br_suffix, Some(b'>') | Some(b'/') | Some(b' ')) {
            output.push('\n');
        } else if tag.starts_with("</p") && (tag == "</p>" || tag.starts_with("</p ")) {
            output.push_str("\n\n");
        }

        cursor = end + 1;
    }

    if cursor < raw.len() {
        output.push_str(&raw[cursor..]);
    }

    output
}

fn decode_entities(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut cursor = 0;

    while let Some(relative_start) = raw[cursor..].find('&') {
        let start = cursor + relative_start;
        output.push_str(&raw[cursor..start]);

        let Some(relative_end) = raw[start..].find(';') else {
            output.push_str(&raw[start..]);
            cursor = raw.len();
            break;
        };
        let end = start + relative_end;
        let entity = &raw[start + 1..end];

        if let Some(decoded) = decode_entity(entity) {
            output.push_str(&decoded);
        } else {
            output.push_str(&raw[start..=end]);
        }
        cursor = end + 1;
    }

    if cursor < raw.len() {
        output.push_str(&raw[cursor..]);
    }

    output
}

fn decode_entity(entity: &str) -> Option<String> {
    let code_point = if let Some(hexadecimal) = entity
        .strip_prefix("#x")
        .or_else(|| entity.strip_prefix("#X"))
    {
        u32::from_str_radix(hexadecimal, 16).ok()
    } else if let Some(decimal) = entity.strip_prefix('#') {
        decimal.parse::<u32>().ok()
    } else {
        return match entity.to_ascii_lowercase().as_str() {
            "amp" => Some('&'.to_string()),
            "apos" => Some('\''.to_string()),
            "gt" => Some('>'.to_string()),
            "lt" => Some('<'.to_string()),
            "nbsp" => Some(' '.to_string()),
            "quot" => Some('"'.to_string()),
            _ => None,
        };
    }?;

    char::from_u32(code_point).map(|character| character.to_string())
}

#[cfg(test)]
mod tests {
    use super::normalize_source_text;

    #[test]
    fn decodes_decimal_and_hexadecimal_entities() {
        let raw = "Quatre objectifs :<br>\n&#x2013; impliquer la f&#x00E9;d&#x00E9;ration &#8211; l&#x2019;exemple";

        assert_eq!(
            normalize_source_text(raw),
            "Quatre objectifs :\n\n– impliquer la fédération – l’exemple"
        );
    }

    #[test]
    fn keeps_unknown_entities_as_source_text() {
        assert_eq!(normalize_source_text("A &unknown; B"), "A &unknown; B");
    }
}
