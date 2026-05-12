use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct SourceSpan {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<String>,
    pub char_start: usize,
    pub char_end: usize,
    pub text: String,
}

impl SourceSpan {
    pub fn new(doc_id: Option<&str>, char_start: usize, char_end: usize, source: &str) -> Self {
        Self {
            doc_id: doc_id.map(ToString::to_string),
            char_start,
            char_end,
            text: source
                .get(char_start..char_end)
                .unwrap_or_default()
                .to_string(),
        }
    }
}
