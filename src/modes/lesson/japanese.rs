use super::{LessonAnswer, LessonAnswerBox};
use kanaria::string::UCSStr;
use rand::Rng;
use unicode_normalization::UnicodeNormalization;

// カタカナ（清音、濁音、半濁音を含む）
const KATAKANA: &str = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポ";

pub struct NormalizedJapaneseAnswer {
    original: String,
    pub(crate) normalized: String,
}

impl NormalizedJapaneseAnswer {
    pub fn new(text: String) -> Self {
        let normalized = normalize_japanese(&text);
        Self {
            original: text,
            normalized,
        }
    }
}

impl LessonAnswer for NormalizedJapaneseAnswer {
    fn check(&self, s: &str) -> bool {
        let normalized_input = normalize_japanese(s);
        self.normalized == normalized_input
    }

    fn into_str(&self) -> &str {
        &self.original
    }

    fn clone_boxed(&self) -> Box<dyn LessonAnswer> {
        Box::new(Self {
            original: self.original.clone(),
            normalized: self.normalized.clone(),
        })
    }
}

fn normalize_japanese(s: &str) -> String {
    let s = UCSStr::from_str(s).upper_case().katakana().to_string();
    s.nfkd().collect::<String>()
}

pub struct JapaneseFiveCharGen;

impl JapaneseFiveCharGen {
    fn random_char() -> char {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = KATAKANA.chars().collect();
        chars[rng.gen_range(0..chars.len())]
    }
}

impl Iterator for JapaneseFiveCharGen {
    type Item = LessonAnswerBox;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result = String::new();

        // 5文字を生成
        for _ in 0..5 {
            result.push(Self::random_char());
        }

        Some(Box::new(NormalizedJapaneseAnswer::new(result)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_japanese_five_char_gen() {
        let mut gen = JapaneseFiveCharGen;

        for _ in 0..10 {
            let result = gen.next().unwrap();
            let s = result.into_str();
            println!("Generated: {}", s);

            // 5文字であることを確認
            assert_eq!(s.chars().count(), 5);
        }
    }
}
