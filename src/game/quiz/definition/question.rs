use lazy_static::lazy_static;
use regex::Regex;
use serde::de;
use serde::{Deserialize, Deserializer};
use std::hash::{Hash, Hasher};
use unidecode::unidecode;

lazy_static! {
    static ref FORBIDDEN_GUESS_CHARACTERS_REGEX: Regex = Regex::new("[^a-z0-9]").unwrap();
}

fn sanitize(answer: &str) -> String {
    let answer = unidecode(answer);
    FORBIDDEN_GUESS_CHARACTERS_REGEX
        .replace_all(&answer.to_lowercase(), "")
        .into()
}

fn bool_from_string<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    match String::deserialize(deserializer)?
        .trim()
        .to_lowercase()
        .as_ref()
    {
        "true" => Ok(true),
        "false" | "" => Ok(false),
        other => Err(de::Error::invalid_value(
            de::Unexpected::Str(other),
            &"true, false or blank",
        )),
    }
}

#[derive(Deserialize, Hash, PartialEq, Eq)]
pub struct RawQuestion {
    pub url: String,
    pub answer: String,
    pub acceptable_answers: Option<String>,
    pub category: String,
    pub score_value: u32,
    #[serde(deserialize_with = "bool_from_string")]
    pub daily_double: bool,
}

#[derive(Clone, Debug)]
pub struct Question {
    pub url: String,
    pub answer: String,
    pub acceptable_answers: Regex,
    pub category: String,
    pub score_value: u32,
    pub daily_double: bool,
}

impl Question {
    pub fn is_guess_correct(&self, guess: &str) -> bool {
        let sanitized_guess = sanitize(guess);
        self.acceptable_answers.is_match(&sanitized_guess)
    }
}

impl PartialEq for Question {
    fn eq(&self, other: &Self) -> bool {
        self.url == other.url
            && self.category == other.category
            && self.score_value == other.score_value
    }
}
impl Eq for Question {}

impl Hash for Question {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.url.hash(state);
        self.category.hash(state);
        self.score_value.hash(state);
    }
}

impl From<RawQuestion> for Question {
    fn from(raw_question: RawQuestion) -> Self {
        // Gather all answers
        let mut acceptable_answers = Vec::new();
        acceptable_answers.push(raw_question.answer.to_owned());
        if let Some(answers) = raw_question.acceptable_answers {
            for answer in answers.split("|") {
                acceptable_answers.push(answer.to_owned());
            }
        }

        // Sanitize
        let acceptable_answers: Vec<String> = acceptable_answers
            .iter()
            .filter_map(|answer| {
                let sanitized = sanitize(answer);
                if sanitized.is_empty() {
                    None
                } else {
                    Some(format!("({})", sanitized))
                }
            })
            .collect();

        // Turn into a regex
        let regex_to_parse = format!("^{}$", acceptable_answers.join("|"));
        let acceptable_answers = Regex::new(&regex_to_parse).unwrap();

        Question {
            url: raw_question.url,
            answer: raw_question.answer,
            acceptable_answers: acceptable_answers,
            category: raw_question.category,
            score_value: raw_question.score_value,
            daily_double: raw_question.daily_double,
        }
    }
}
