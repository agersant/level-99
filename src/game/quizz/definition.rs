use anyhow::*;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct Question {
    pub url: String,
    pub answer: String,
    pub score_value: u32,
}

impl Question {
    pub fn is_guess_correct(&self, guess: &str) -> bool {
        guess == self.answer
    }
}

#[derive(Debug, Deserialize)]
pub struct QuizzDefinition {
    questions: Vec<Question>,
}

impl QuizzDefinition {
    pub fn open(source: &Path) -> Result<QuizzDefinition> {
        let mut questions = Vec::new();

        let file = File::open(source)?;
        let mut csv_reader = csv::Reader::from_reader(file);
        for question in csv_reader.deserialize() {
            let question: Question = question?;
            questions.push(question);
        }

        Ok(QuizzDefinition { questions })
    }

    pub fn get_questions(&self) -> &Vec<Question> {
        &self.questions
    }
}
