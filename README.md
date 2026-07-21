# AoC Suite

A a command-line tool and tui for Advent of Code (AOC) that helps you manage solutions, download puzzles, submit answers, with (future) support for multiple programming languages.

## Features

- Download and caches puzzle descriptions and input data automatically
- See your progress from the calendar
- Templating system for premade exercise layouts
- Library file support
- Open files in your editor of choice
- Submit solutions directly from the command line
- interacts with git for managing solutions

### Language support

Generally language implementations are made with the fewest tools possible for simplicity. However, few tools are required for each language to manage enviroments

- Python3 is managed via pip
- Rust is managed through cargo

## Installation

### From Source

Will be improved in the future. For now the cli can be installed via the cli

```bash
git clone https://github.com/your-username/aocsuite.git
cd aocsuite
cargo install --path aocsuite-cli
```

## Quick Start CLI

1. **Configure your session token**:
   Either by environment variables or set via the config command

   ```bash
   export AOC_SESSION="<MY-TOKEN>"
   aocsuite-cli config set session
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

- `aocsuite-cli open ` - Will open the puzzle and a file for your soloutions. Also opens the input along with a file for potential examples
- `aocsuite-cli run` - Run your solution on the AoC input. specify --test for your own examples
- `aocsuite-cli submit --part PART [ANSWER]` - Submit an answer, prompting when `ANSWER` is omitted

### Dependencies

All languages support simple adding, listing and removing of dependencies from external libraries. see `aocsuite-cli env`

### Libraries and templates

Local library code can be added via `aocsuite-cli lib`.

After adding library you may want those to always be imported in your template.
Use `aocsuite-cli template` to edit your template

### Caches

All data downloaded from Advent of code is cached locally to avoid multiple look ups and spare his servers. In case you want to remove these caches `aocsuite-cli clean cache` allows this.

Some languages also caches large files during building of a program. These can be cleaned through `aocsuite-cli clean lang`

### AoC interaction commands

- `aocsuite-cli view` - Opens the puzzle of the day in the browser
- `aocsuite-cli calendar` - Render your AoC calendar colored in the terminal
- `aocsuite-cli leaderboard` - Opens the global leaderboard. Or a private if id is given.

### Configuration Commands

- `aocsuite-cli config set KEY` - Set configuration value from a prompt
- `aocsuite-cli config get KEY` - Get configuration value

Configurations can also be managed through enviroment variables:

- `AOC_SESSION`
- `AOC_LANGUAGE`
- `AOC_YEAR`
- `AOC_TEMPLATE_DIR`
- `AOC_EDITOR`

### Git tracking

`aocsuite-cli git` - wraps around git to enable version control of the solution directory. A basic .gitignore is supplied to avoid tracking aocsuite specific files.

Files are stored at `$XDG_DATA_HOME/aocsuite` or `$HOME/.local/data/aocsuite` and can also be managed manually from there.

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Especially if you want to add support for your favorite editor or language.

## Acknowledgments

- [Advent of Code](https://adventofcode.com) by Eric Wastl
