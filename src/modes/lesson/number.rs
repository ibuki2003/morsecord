use super::LessonAnswer;

#[derive(Debug, Clone)]
pub struct LessonAnswerContestNumber(String);

impl LessonAnswerContestNumber {
    pub fn new_5nn(s: &str) -> Self {
        Self("5NN ".to_owned() + s)
    }
}

impl LessonAnswer for LessonAnswerContestNumber {
    fn check(&self, mut s: &str) -> bool {
        if s.starts_with("5NN") {
            s = &s[3..];
            s = s.trim();
        }
        self.0[4..] == *s
    }

    fn into_str(&self) -> &str {
        &self.0
    }

    fn clone_boxed(&self) -> Box<dyn LessonAnswer> {
        Box::new(self.clone())
    }
}
