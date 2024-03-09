use anyhow::Context;
use rand::seq::SliceRandom;

use super::LessonAnswerBox;

pub struct FileSourceGen {
    data: Vec<String>,
}
const LESSON_TXT_DIR: &str = "./lesson_txt/";
impl FileSourceGen {
    pub fn new(filename: &str) -> anyhow::Result<Self> {
        anyhow::ensure!(!filename.contains('/'), "invalid filename");

        let p = std::path::Path::new(LESSON_TXT_DIR).join(filename);

        if !p.is_file() {
            let e = std::fs::read_dir(LESSON_TXT_DIR)
                .context("error: lesson_txt directory not found.")?
                .map(|x| x.unwrap().file_name().to_str().unwrap().to_owned())
                .filter(|x| !x.starts_with('.'))
                .collect::<Vec<_>>()
                .join(", ");

            anyhow::bail!("invalid filename. availables: {}", e);
        }

        let text = std::fs::read_to_string(p)?;
        let data = text.lines().map(|x| x.to_owned()).collect::<Vec<_>>();
        Ok(Self { data })
    }
}

impl Iterator for FileSourceGen {
    type Item = LessonAnswerBox;

    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::thread_rng();
        let v = self.data.choose(&mut rng)?;
        Some(Box::new(v.clone()))
    }
}
