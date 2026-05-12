use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralAssistConfig {
    pub tokenizer_path: Option<String>,
    pub onnx_model_path: Option<String>,
    pub task: NeuralAssistTask,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NeuralAssistTask {
    ActorMentionDetection,
    EventDedupEmbedding,
    FrictionPhraseScoring,
    StanceTensionScoring,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NeuralAssistSignal {
    pub task: NeuralAssistTask,
    pub label: String,
    pub char_start: usize,
    pub char_end: usize,
    pub score: f32,
}

pub trait NeuralAssist {
    fn analyze(&self, text: &str) -> Vec<NeuralAssistSignal>;
}
