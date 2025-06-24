use crate::{
    AocResult,
    language::{Language, LanguageRunner, base_language_dir, get_language_runner},
    utils::{PuzzleDay, PuzzleYear},
};

pub fn scaffold(
    day: PuzzleDay,
    year: PuzzleYear,
    template: Option<String>,
    language: &Language,
) -> AocResult<()> {
    let mut template = template;
    let runner = get_language_runner(&language);
    template = template.or_else(|| {
        let base_dir = base_language_dir(&language);
        let default_template_dir = base_dir.join("templates");
        if default_template_dir.exists() {
            Some(
                default_template_dir
                    .to_str()
                    .expect("Path should be UTF-8")
                    .to_owned(),
            )
        } else {
            None
        }
    });

    runner.scaffold(day, year, template)?;
    Ok(())
}
