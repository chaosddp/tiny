use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Debug, Deserialize, Serialize)]
pub struct Choice {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Instructions {
    Text(String),
    Object(JsonValue),
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Question {
    Noul(Instructions),
    Choice {
        instructions: Instructions,
        criteria: Vec<Choice>,
    },
    Score {
        instructions: Instructions,
        criteria: Vec<String>,
    },
}

#[derive(Debug, Deserialize, Serialize)]
pub enum State {
    Text(String),
    Array(Vec<String>),
    Object(JsonValue),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DecisionMessage {
    pub state: State,
    pub questions: Vec<Question>,
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DecisionTokenUsage {
    pub input: usize,
    pub output: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Probability {
    pub name: String,
    pub value: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChoiceAnswer {
    pub key: String,
    pub choice: String,
    pub confidence: f32,
    pub probabilities: Vec<Probability>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreAnswer {
    pub key: String,
    pub score: f32,
    pub confidence: f32,
    pub legend: HashMap<String, String>,
    pub probabilities: Vec<Probability>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Answer {
    Choice(ChoiceAnswer),
    Score(ScoreAnswer),
    Noul { key: String, noul: f32 },
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DecisionResponse {
    pub model: String,
    pub answers: Vec<Answer>,
}
