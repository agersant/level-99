use anyhow::*;
use std::collections::HashSet;
use std::fs::File;
use std::path::Path;

pub mod question;

pub use question::Question;
use question::RawQuestion;

#[derive(Debug)]
pub struct QuizzDefinition {
    questions: HashSet<Question>,
}

impl QuizzDefinition {
    pub fn open(source: &Path) -> Result<QuizzDefinition> {
        let mut questions = HashSet::new();

        let file = File::open(source)?;
        let mut csv_reader = csv::Reader::from_reader(file);
        for question in csv_reader.deserialize() {
            let raw_question: RawQuestion = question?;
            questions.insert(raw_question.into());
        }

        Ok(QuizzDefinition { questions })
    }

    pub fn get_questions(&self) -> &HashSet<Question> {
        &self.questions
    }
}
