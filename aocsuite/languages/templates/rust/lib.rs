pub mod utils {
    use clap::{Parser, ValueEnum};
    use std::fs;
    use std::io::{self};
    use std::process;

    /// Struct for command line arguments using `clap` derive macro
    #[derive(Parser, Debug)]
    #[command(name = "Advent of Code Exercise", about = "Run a specified exercise")]
    struct AocNamespace {
        /// Path to file input for the exercise
        #[arg(long, required = true)]
        data_path: String,

        /// Exercise to run (1 or 2)
        #[arg(long, required = true)]
        exercise: u8, // Accept exercise as a number (1 or 2)
    }

    /// Enum for handling exercise choices (1 or 2)
    enum Exercise {
        Exercise1,
        Exercise2,
    }

    impl Exercise {
        fn from_u8(value: u8) -> Result<Self, String> {
            match value {
                1 => Ok(Exercise::Exercise1),
                2 => Ok(Exercise::Exercise2),
                _ => Err(format!(
                    "Invalid exercise number: {}. Choose 1 or 2.",
                    value
                )),
            }
        }
    }
    /// Function to load data from the specified path
    fn load_data(data_path: &str) -> io::Result<String> {
        fs::read_to_string(data_path).map(|data| data.trim().to_string())
    }

    /// Run function that matches the parsed exercise and executes the corresponding logic
    pub fn run<F1, F2>(exercise1: F1, exercise2: F2)
    where
        F1: Fn(&str) -> i32,
        F2: Fn(&str) -> i32,
    {
        let args = AocNamespace::parse(); // Use clap to parse the arguments
        let input_data = load_data(&args.data_path).unwrap_or_else(|err| {
            eprintln!("Error reading file: {}", err);
            process::exit(1);
        });
        // Convert the exercise number to an Exercise enum variant
        let exercise = Exercise::from_u8(args.exercise).unwrap_or_else(|err| {
            eprintln!("{}", err);
            process::exit(1);
        });

        let answer = match exercise {
            Exercise::Exercise1 => exercise1(&input_data),
            Exercise::Exercise2 => exercise2(&input_data),
        };

        println!("Answer to exercise {:?}: {}", args.exercise, answer);
    }
}