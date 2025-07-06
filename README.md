# AOC Suite

A set of libraries used to generate a command-line toolkit for Advent of Code (AOC) that helps you manage solutions, download puzzles, submit answers, with (future) support for multiple programming languages.

## Features

- Download puzzle descriptions and input data automatically
- Templating for new days
- Open files in editor
- Submit solutions directly from the command line

## Installation

### From Source

```bash
git clone https://github.com/your-username/aocsuite.git
cd aocsuite
cargo install --path aocsuite-cli
```

## Quick Start

1. **Configure your session token (Preferably through env variables to avoid leaking tokens)**:

   ```bash
   export AOC_SESSION="<MY-TOKEN>"
   ```

2. **Generate a new set of files**:
   ```bash
   aocsuite-cli new
   ```
3. **Work on your solution** in the generated directory and run it:
   Using the editor: `aocsuite-cli edit` or manually

4. **Test your solution**:
   ```bash
   aocsuite-cli test
   ```
5. **Run your solution**:
   ```bash
   aocsuite-cli run
   ```
6. **Submit your answer**:
   ```bash
   aocsuite-cli submit <PART> <ANSWER>
   ```

### Session Token Setup

To get your input and submit answers to Advent of Code website, you'll need your session token:

1. Log in to [Advent of Code](https://adventofcode.com)
2. Open browser developer tools
3. Go to Application/Storage → Cookies
4. Find the `session` cookie value
5. Configure it: `aocsuite-cli config set session`
6. Paste your session token into the prompt.

## Commands

### Core Commands

Most commands require day and year and can be specified as --day and --year

- `aocsuite-cli new ` - Download and create exercise files
- `aocsuite-cli download` - Download puzzle and input
- `aocsuite-cli submit --part PART ANSWER` - Submit an answer
- `aocsuite-cli edit` - Open solution in editor
- `aocsuite-cli run` - Run your solution with input.txt as input
- `aocsuite-cli test <INPUT FILE>` - Run your solution with example.txt or specified input file

### Configuration Commands

- `aocsuite-cli config set KEY` - Set configuration value from a prompt
- `aocsuite-cli config get KEY` - Get configuration value

Configurations can also be managed through enviroment variables:

- `AOC_SESSION`
- `AOC_LANGUAGE`
- `AOC_YEAR`
- `AOC_TEMPLATE_DIR`
- `EDITOR`

## Project Structure

```
aoc/
├── .aocsuite/
│   └── config.json          # Workspace configuration
├── data/                    # Downloaded puzzle descriptions and inputs
│   └── year2024/
│       └── day1/
│           ├── puzzle.md    # Puzzle description
│           ├── example.txt  # Puzzle Example input
│           └── input.txt    # Puzzle input
├── templates/               # Language-specific templates
│   └── rust/
│       └── lib.rs          # Rust solution template
└─── rust/                   # Rust solutions
    └── year2024/
        └── day1/
            ├── Cargo.toml
            └─── src/
                ├── main.rs
                └── lib.rs


```

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Especially if you want to add support for your favorite editor or language.

- Adding support for new programming languages
- Adding new editor integrations

## Acknowledgments

- [Advent of Code](https://adventofcode.com) by Eric Wastl
