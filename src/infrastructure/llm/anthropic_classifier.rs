//! Proposition de familles thematiques par un modele de langage.
//!
//! Le modele ne recoit que le libelle du texte (RM-04) et ne rend que des
//! familles et du texte de justification (RM-10): la reponse est contrainte par
//! un schema qui ne porte aucun nombre.

use std::time::Duration;

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::application::ports::theme_classifier::{ClassifierError, ThemeClassifier};
use crate::domain::theme::{FamilyCode, ProposedFamily, MAX_FAMILIES};

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_MODEL: &str = "claude-opus-5";

/// Version de l'instruction, conservee avec chaque proposition pour la rendre
/// inspectable. A incrementer des que le texte ci-dessous change.
const PROMPT_VERSION: &str = "thematisation-v1";

/// Tentatives par texte. Au-dela, le texte reste non rattache et sera repris a
/// la passe suivante.
const MAX_ATTEMPTS: u32 = 3;

pub struct AnthropicThemeClassifier {
    http: reqwest::Client,
    api_key: String,
    model: String,
    system_prompt: String,
}

impl AnthropicThemeClassifier {
    /// BYOK: rend `None` sans cle, le site tourne sans proposition.
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY").ok().filter(|k| !k.is_empty())?;
        let model = std::env::var("ANTHROPIC_MODEL")
            .ok()
            .filter(|m| !m.is_empty())
            .unwrap_or_else(|| DEFAULT_MODEL.to_string());
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .ok()?;
        Some(Self {
            http,
            api_key,
            model,
            system_prompt: system_prompt(),
        })
    }

    fn request_body(&self, text_label: &str) -> Value {
        json!({
            "model": self.model,
            "max_tokens": 2000,
            // `low`: le rattachement d'un libelle est une tache courte. Le
            // schema fait le reste du travail de cadrage.
            "output_config": {
                "effort": "low",
                "format": { "type": "json_schema", "schema": response_schema() }
            },
            "system": self.system_prompt,
            "messages": [{
                "role": "user",
                // Le libelle du texte, rien d'autre (RM-04).
                "content": text_label
            }]
        })
    }
}

/// Referentiel ferme decrit au modele. Construit depuis le domaine: la page
/// methode et l'instruction ne peuvent pas diverger (RM-08).
fn system_prompt() -> String {
    let familles = FamilyCode::ALL
        .iter()
        .map(|f| format!("- `{}` — {} : {}", f.as_str(), f.label(), f.scope()))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "Tu rattaches un texte soumis au vote de l'Assemblée nationale à une ou plusieurs \
         familles thématiques.\n\n\
         Familles disponibles :\n{familles}\n\n\
         Règles :\n\
         - Tu ne reçois que le libellé du texte. Tu ne disposes d'aucun résultat de vote, \
         d'aucun décompte, d'aucun nom de groupe parlementaire, et tu n'en demandes pas.\n\
         - Tu retiens de une à {MAX_FAMILIES} familles, de la plus centrale à la moins centrale. \
         N'en retiens plusieurs que si le texte porte réellement sur plusieurs de ces domaines.\n\
         - Tu rattaches sur l'objet du texte, jamais sur son orientation, ses effets supposés \
         ou sa valeur.\n\
         - La justification dit ce que le libellé du texte porte, en une à deux phrases. \
         Aucun jugement, aucune évaluation, aucun chiffre, aucune estimation, aucun \
         qualificatif d'ampleur.\n\
         - Si aucune famille ne convient, rends une liste vide plutôt qu'un rattachement \
         approximatif.\n\
         - N'invente aucune famille : seuls les codes listés ci-dessus sont acceptés."
    )
}

fn response_schema() -> Value {
    let codes: Vec<&str> = FamilyCode::ALL.iter().map(|f| f.as_str()).collect();
    json!({
        "type": "object",
        "properties": {
            "familles": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "famille": { "type": "string", "enum": codes },
                        "justification": { "type": "string" }
                    },
                    "required": ["famille", "justification"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["familles"],
        "additionalProperties": false
    })
}

#[derive(Deserialize)]
struct Answer {
    familles: Vec<AnsweredFamily>,
}

#[derive(Deserialize)]
struct AnsweredFamily {
    famille: String,
    justification: String,
}

#[async_trait]
impl ThemeClassifier for AnthropicThemeClassifier {
    async fn propose(&self, text_label: &str) -> Result<Vec<ProposedFamily>, ClassifierError> {
        let body = self.request_body(text_label);
        let mut last_error = ClassifierError::Call("aucune tentative".to_string());

        for attempt in 1..=MAX_ATTEMPTS {
            let response = self
                .http
                .post(API_URL)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", API_VERSION)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await;

            let response = match response {
                Ok(response) => response,
                Err(error) => {
                    last_error = ClassifierError::Call(error.to_string());
                    backoff(attempt).await;
                    continue;
                }
            };

            let status = response.status();
            if status.as_u16() == 429 || status.is_server_error() {
                last_error = ClassifierError::Call(format!("HTTP {status}"));
                backoff(attempt).await;
                continue;
            }
            if !status.is_success() {
                let detail = response.text().await.unwrap_or_default();
                return Err(ClassifierError::Call(format!(
                    "HTTP {status}: {}",
                    detail.chars().take(300).collect::<String>()
                )));
            }

            let payload: Value = response
                .json()
                .await
                .map_err(|e| ClassifierError::Answer(e.to_string()))?;
            return parse_answer(&payload);
        }

        Err(last_error)
    }

    fn model(&self) -> &str {
        &self.model
    }

    fn prompt_version(&self) -> &str {
        PROMPT_VERSION
    }
}

async fn backoff(attempt: u32) {
    tokio::time::sleep(Duration::from_millis(500 * 2u64.pow(attempt - 1))).await;
}

/// Lit la reponse. Une famille hors referentiel ou une justification vide est
/// ecartee et journalisee (RM-05, RM-08); les autres sont conservees.
fn parse_answer(payload: &Value) -> Result<Vec<ProposedFamily>, ClassifierError> {
    match payload.get("stop_reason").and_then(Value::as_str) {
        Some("refusal") => {
            return Err(ClassifierError::Answer("réponse refusée".to_string()));
        }
        Some("max_tokens") => {
            return Err(ClassifierError::Answer("réponse tronquée".to_string()));
        }
        _ => {}
    }

    let text = payload
        .get("content")
        .and_then(Value::as_array)
        .and_then(|blocks| {
            blocks
                .iter()
                .find(|b| b.get("type").and_then(Value::as_str) == Some("text"))
        })
        .and_then(|block| block.get("text"))
        .and_then(Value::as_str)
        .ok_or_else(|| ClassifierError::Answer("aucun bloc de texte".to_string()))?;

    let answer: Answer =
        serde_json::from_str(text).map_err(|e| ClassifierError::Answer(e.to_string()))?;

    let mut proposed = Vec::with_capacity(answer.familles.len());
    for entry in answer.familles {
        let family = match FamilyCode::parse(&entry.famille) {
            Ok(family) => family,
            Err(_) => {
                tracing::warn!(famille = entry.famille, "famille hors référentiel écartée");
                continue;
            }
        };
        match ProposedFamily::new(family, entry.justification) {
            Ok(entry) => proposed.push(entry),
            Err(error) => tracing::warn!(%error, "proposition écartée"),
        }
    }
    Ok(proposed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(text: &str) -> Value {
        json!({
            "stop_reason": "end_turn",
            "content": [{ "type": "text", "text": text }]
        })
    }

    #[test]
    fn the_prompt_lists_every_family_of_the_referential() {
        let prompt = system_prompt();
        for family in FamilyCode::ALL {
            assert!(prompt.contains(family.as_str()), "{}", family.as_str());
            assert!(prompt.contains(family.label()));
        }
    }

    #[test]
    fn the_schema_only_accepts_referential_codes() {
        let schema = response_schema();
        let codes = schema["properties"]["familles"]["items"]["properties"]["famille"]["enum"]
            .as_array()
            .unwrap();
        assert_eq!(codes.len(), FamilyCode::ALL.len());
        assert!(codes.iter().any(|c| c == "logement"));
    }

    #[test]
    fn a_well_formed_answer_is_read() {
        let proposed = parse_answer(&payload(
            r#"{"familles":[{"famille":"logement","justification":"Le texte porte sur l'urbanisme."}]}"#,
        ))
        .unwrap();
        assert_eq!(proposed.len(), 1);
        assert_eq!(proposed[0].family(), FamilyCode::Logement);
    }

    #[test]
    fn a_family_outside_the_referential_is_dropped_without_failing() {
        let proposed = parse_answer(&payload(
            r#"{"familles":[{"famille":"securite","justification":"hors référentiel"},
                            {"famille":"sante-social","justification":"soins palliatifs"}]}"#,
        ))
        .unwrap();
        assert_eq!(proposed.len(), 1);
        assert_eq!(proposed[0].family(), FamilyCode::SanteSocial);
    }

    #[test]
    fn a_family_without_justification_is_dropped() {
        let proposed = parse_answer(&payload(
            r#"{"familles":[{"famille":"numerique","justification":"  "}]}"#,
        ))
        .unwrap();
        assert!(proposed.is_empty());
    }

    #[test]
    fn an_empty_answer_is_not_an_error() {
        let proposed = parse_answer(&payload(r#"{"familles":[]}"#)).unwrap();
        assert!(proposed.is_empty());
    }

    #[test]
    fn a_refusal_is_an_error() {
        let refused = json!({ "stop_reason": "refusal", "content": [] });
        assert!(parse_answer(&refused).is_err());
    }

    #[test]
    fn a_truncated_answer_is_an_error() {
        let truncated = json!({
            "stop_reason": "max_tokens",
            "content": [{ "type": "text", "text": "{\"familles\":[" }]
        });
        assert!(parse_answer(&truncated).is_err());
    }
}
