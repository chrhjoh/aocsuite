# TODO

## Features

- [x] Http: Implement Http commands
- [x] Run: Make Base template for rust run
- [x] Editor: Implement a simple neovim editor
- [x] Benchmarking: Add simple benchmarking times
- [x] Template: Add options for user defined templates
- [x] File writes: Add confirmation
- [x] File open: (edit and run) check if files exist before starting.
- [x] Move the actual running into temporary files inside .aocsuite (symlink lib file, both for running and editing).
- [ ] Make a system that allows adding, editing, and removing (not template), lib and template files.
- [x] Dependencies for run environments (add and remove)
- [x] Add git support (create basic .gitignore to remove caches etc)
- [x] Move everything to $XDG_DATA_HOME dir. (needs to move languages in some way)
- [x] config as prompt
- [x] Add cache support into the runner if answer is correct.
- [x] edit should take lib and template args
- [ ] Github release
- [ ] Upload crates

## Bug Fixes

- [ ]

## Refactoring

- [x] aocsuite-packages: Move into -client, -editor, -lang, -config packages
- [ ] improve error messages

## Tests

- [ ] Implement some tests

## Documentation

- [ ] Write some documentation.
- [x] Create a README.md
