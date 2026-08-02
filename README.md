# AoC Suite

A a command-line tool and tui for Advent of Code (AOC) that helps you manage solutions, download puzzles, submit answers, with (future) support for multiple programming languages.

## Features

- Download and caches puzzle descriptions and input data automatically
- See your progress from the calendar
- Templating system for premade exercise layouts
- Library file support
- Open files in your editor of choice
- Submit solutions from the CLI or TUI
- interacts with git for managing solutions

### Language support

Generally language implementations are made with the fewest tools possible for simplicity. However, few tools are required for each language to manage enviroments

- Python3 is managed via pip
- Rust is managed through cargo

## Installation

### From Source

For now the CLI and TUI can be installed from their workspace crates:

```bash
git clone https://github.com/your-username/aocsuite.git
cd aocsuite
cargo install --path aocsuite-cli
cargo install --path aocsuite-tui
```

## Quick Start CLI

1. **Configure your session token**:
   Set it through the non-echoing configuration prompt:

   ```bash
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

## Quick Start TUI

Start the terminal interface with:

```bash
aocsuite-tui
```

The TUI provides three tabs:

- **Calendar**: browse released years and puzzles, download or refresh puzzle
  descriptions, open a puzzle in the browser or editor, and run its solver with
  AoC input or the shared example. Submit answers with `s`. Press `1` or `2` to
  run that part, `i` to toggle AoC/shared-example input, and `u` to refresh the
  selected year's calendar.
- **Language**: select Rust or Python for the current session and manage
  packages, libraries, and templates.
- **Config**: manage the default year, editor, run-history retention, and AoC
  session credential.

Press `Tab` or `Shift-Tab` to change tabs, `?` for the active tab's keymap, and
`q` to quit. Use `Up`, `Down`, `PageUp`, or `PageDown` to scroll long help and
`Esc` to close it. The layout adapts on narrow terminals on a best-effort basis.

Git, cleanup, uninstall, and leaderboards remain CLI workflows. TUI solver
execution uses the current in-session language and runs part one or part two
directly; custom input paths remain CLI-only.

## Session Token Setup

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

The editor falls back to `EDITOR` when no editor is configured.

### Git tracking

`aocsuite-cli git` - wraps around git to enable version control of the solution directory. A basic .gitignore is supplied to avoid tracking aocsuite specific files.

Files are stored at `$AOCSUITE_DATA_DIR`, `$XDG_DATA_HOME/aocsuite`, or `$HOME/.local/share/aocsuite`, in that order. Set `AOCSUITE_DATA_DIR` to override the complete runtime root.

## License

This project is licensed under the [MIT License](LICENSE).

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Especially if you want to add support for your favorite editor or language.

## Acknowledgments

- [Advent of Code](https://adventofcode.com) by Eric Wastl
