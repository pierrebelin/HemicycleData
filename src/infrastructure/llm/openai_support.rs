use serde_json::Value;

/// Extrait le texte JSON d'une réponse Responses API.
///
/// La structure publique peut contenir plusieurs éléments de sortie. Le
/// générateur ne lit que le bloc `output_text`; une réponse refusée,
/// incomplète ou dépourvue de texte est rejetée par le port appelant.
pub(crate) fn output_text(payload: &Value) -> Result<&str, String> {
    if payload
        .get("status")
        .and_then(Value::as_str)
        .is_some_and(|status| status == "incomplete" || status == "failed")
    {
        return Err("reponse incomplete ou en echec".into());
    }

    if let Some(text) = payload.get("output_text").and_then(Value::as_str) {
        return Ok(text);
    }

    payload
        .get("output")
        .and_then(Value::as_array)
        .and_then(|items| {
            items.iter().find_map(|item| {
                item.get("content")
                    .and_then(Value::as_array)
                    .and_then(|content| {
                        content.iter().find_map(|block| {
                            if matches!(
                                block.get("type").and_then(Value::as_str),
                                Some("output_text" | "text")
                            ) {
                                block.get("text").and_then(Value::as_str)
                            } else {
                                None
                            }
                        })
                    })
            })
        })
        .ok_or_else(|| "aucun bloc output_text".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn reads_the_convenience_output_text_field() {
        let payload = json!({"output_text":"{\"ok\":true}"});
        assert_eq!(output_text(&payload).unwrap(), "{\"ok\":true}");
    }

    #[test]
    fn reads_nested_output_text_blocks() {
        let payload = json!({
            "output": [{
                "type": "message",
                "content": [{"type": "output_text", "text": "{}"}]
            }]
        });
        assert_eq!(output_text(&payload).unwrap(), "{}");
    }

    #[test]
    fn rejects_incomplete_responses() {
        let payload = json!({"status":"incomplete"});
        assert!(output_text(&payload).is_err());
    }
}
