use rand::{self, Rng};

use super::{number::LessonAnswerContestNumber, LessonAnswerBox};
pub struct AllJANumberGen {
    allja_nr: Vec<String>,
}

impl AllJANumberGen {
    pub fn new() -> Self {
        let allja_nr = [
            (101..=114).map(|x| x.to_string()).collect::<Vec<_>>(),
            (2..=48)
                .map(|x| {
                    if x < 10 {
                        format!("0{}", x)
                    } else {
                        x.to_string()
                    }
                })
                .collect::<Vec<_>>(),
        ]
        .concat();
        Self { allja_nr }
    }
}

impl Default for AllJANumberGen {
    fn default() -> Self {
        Self::new()
    }
}

impl Iterator for AllJANumberGen {
    type Item = LessonAnswerBox;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::thread_rng();
        let s = self.allja_nr[rng.gen_range(0..self.allja_nr.len())].clone();

        let s = match rand::random::<u8>() {
            0..=99 => s + "H",
            100..=199 => s + "M",
            200..=224 => s + "L",
            225..=255 => s + "P",
        };
        Some(Box::new(LessonAnswerContestNumber::new_5nn(&s)))
    }
}

#[test]
fn test_allja_number() {
    let mut gen = AllJANumberGen::new();
    for _ in 0..100 {
        println!("{}", gen.next().unwrap().into_str());
    }
}
