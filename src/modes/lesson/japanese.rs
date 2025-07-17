use super::LessonAnswerBox;
use rand::Rng;

// カタカナ（清音、濁音、半濁音を含む）
const KATAKANA: &str = "アイウエオカキクケコサシスセソタチツテトナニヌネノハヒフヘホマミムメモヤユヨラリルレロワヲンガギグゲゴザジズゼゾダヂヅデドバビブベボパピプペポ";

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

        Some(Box::new(result))
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
