use std::time::Duration;

use crate::game::quizz::definition::Question;
use crate::game::quizz::{State, Transition};
use crate::output::{OutputPipe, Payload};

#[derive(Debug)]
pub struct QuestionState {
    question: Question,
    time_elapsed: Duration,
    time_limit: Duration,
}

impl QuestionState {
    pub fn new(question: Question, duration: Duration) -> Self {
        QuestionState {
            question,
            time_elapsed: Duration::default(),
            time_limit: duration,
        }
    }

    pub fn guess(&mut self, guess: &str, output_pipe: &mut OutputPipe) -> bool {
        let guessed_correctly = self.question.is_guess_correct(guess);
        if guessed_correctly {
            output_pipe.push(Payload::Text("Correct!".into()));
        } else {
            output_pipe.push(Payload::Text(guess.into()));
            output_pipe.push(Payload::Text("WRONG!".into()));
        }
        guessed_correctly
    }
}

impl State for QuestionState {
    fn tick(&mut self, _output_pipe: &mut OutputPipe, _dt: Duration) -> Option<Transition> {
        None
    }

    fn begin(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::Text("Time for a question!".into()));
        output_pipe.push(Payload::Audio(self.question.url.clone()));
    }

    fn end(&mut self, output_pipe: &mut OutputPipe) {
        output_pipe.push(Payload::StopAudio);
    }
}
