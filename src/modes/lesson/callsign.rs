use crate::modes::lesson::rand_char;

use super::LessonAnswerBox;

const ALPHA: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
const ALNUM: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
const NUM: &str = "0123456789";
const JA_PRF: &str = "AEFGHIJKLMNOPQRS";

pub struct JaCallsignGen;
impl Iterator for JaCallsignGen {
    type Item = LessonAnswerBox;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO: improve algorithm
        let s = match rand::random::<u8>() {
            0..=13 => {
                "7".to_string()
                    + rand_char("JKLMN")
                    + rand_char(NUM)
                    + rand_char(ALPHA)
                    + rand_char(ALPHA)
                    + rand_char(ALPHA)
            }
            14 => {
                "8".to_string()
                    + rand_char("JN")
                    + rand_char(NUM)
                    + rand_char(ALNUM)
                    + rand_char(ALNUM)
                    + rand_char(ALNUM)
            }
            15..=18 => "JA".to_owned() + rand_char(NUM) + rand_char(ALPHA) + rand_char(ALPHA),
            19 => "JR6".to_owned() + rand_char(ALPHA) + rand_char(ALPHA),
            20..=29 => "JD1".to_owned() + rand_char(ALPHA) + rand_char(ALPHA) + rand_char(ALPHA),
            30..=255 => {
                "J".to_string()
                    + rand_char(JA_PRF)
                    + rand_char(NUM)
                    + rand_char(ALPHA)
                    + rand_char(ALPHA)
                    + rand_char(ALPHA)
            }
        };

        let s = if rand::random::<u8>() < 50 {
            s + "/" + &rand_char(NUM)
        } else {
            s
        };

        Some(Box::new(s))
    }
}
