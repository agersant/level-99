use anyhow::*;
use serde::Deserialize;
use std::fs::File;
use std::path::Path;

#[derive(Clone, Debug, Deserialize)]
pub struct Question {
    pub url: String,
    pub answer: String,
}

#[derive(Debug, Deserialize)]
pub struct QuizzDefinition {
    questions: Vec<Question>,
}

pub struct Quizz {
    remaining_questions: Vec<Question>,
    current_question: Option<Question>,
}

impl Quizz {
    pub fn new(definition: QuizzDefinition) -> Quizz {
        Quizz {
            remaining_questions: definition.questions.clone(),
            current_question: None,
        }
    }

    pub fn begin_new_question(&mut self) -> Option<&Question> {
        if self.remaining_questions.is_empty() {
            return None;
        }
        self.current_question = Some(self.remaining_questions.swap_remove(0));
        self.current_question.as_ref()
    }
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
}
