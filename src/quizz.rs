struct Question {
    url: String,
    correct_answers: Vec<String>,
}

struct QuizzDefinition {
    questions: Vec<Question>,
}

struct QuizzRuntime {
    definition: QuizzDefinition,
}
