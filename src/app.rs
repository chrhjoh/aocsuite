use chrono::Datelike;

use crate::{
    AocCommand,
    cli::AocArgs,
    language::{LanguageRunner, base_language_dir, get_language_runner},
    utils::today,
};

pub fn run_aocsuite(args: AocArgs) {
    match args.command {
        AocCommand::New { template, language } => {
            let mut template = template;
            let runner = get_language_runner(&language);
            let day = args.day.unwrap_or_else(|| today().day());
            let year = args.year.unwrap_or_else(|| today().year());
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

            let result = runner.scaffold(day, year, template);
        }
        _ => panic!("Not Implemented Yet"),
    }
}
