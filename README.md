# Gladius & Bernardo Repository
**Mirror Warning**: If you're viewing this on GitHub, be aware it's a manually synchronized mirror that I might occasionally forget to update. For the most current version, visit [GitLab Repository](https://gitlab.com/njskalski/bernardo).

## Introduction
This repository houses two primary projects:
- Bernardo: A Text-based User Interface (TUI) widget library.
- Gladius: A feature-rich code editor.

Both projects are currently maintained under a single repository for ease of development. It's essential to note that both projects are in their alpha phase, which means they might be unstable. Please refrain from using them for critical tasks.

**Licenses**: Under GPLv3. A potential re-release of Bernardo under LGPL is under consideration. If you're thinking about contributing, please ensure you're comfortable with this licensing scenario.

## Current Features of Gladius
- File reading and writing capabilities.
- Syntax highlighting via tree-sitter, with themes derived from the syntect crate.
- Undo/redo functionality (Use `ctrl-z` for undo and `ctrl-x` for redo. Note: `shift+ctrl` shortcuts have known issues with the command-line).
- Clipboard operations: copy (`ctrl-c`) and paste (`ctrl-v`).
- Fuzzy file or directory navigation with `ctrl-h`.
- A sophisticated "save-as" dialog (primarily for widget functionality testing).
- Multi-cursor support: Activate "drop cursor" mode with `ctrl-w`, navigate with arrows, and add/remove cursors using Enter. `Esc` exits this mode.

**Fun Fact**: The name inspiration comes from [Bernardo Gui](https://pl.wikipedia.org/wiki/Bernard_Gui), a historian, inquisitor, and for some time, the Bishop of Tui. Represented as a controversial figure in Umberto Eco's "The Name of the Rose," we found the juxtaposition fitting for our project that deems using a mouse for programming as heretical.

## Build Instructions
Having build issues? Here are some steps to assist:

**SSH Key Validation**: Ensure your SSH keys are set up correctly. Check the [GitLab SSH Key Documentation](https://docs.gitlab.com/ee/user/ssh.html).

**Git Submodule Initialization:**

```bash
git submodule init
git submodule update
```

## Running the Projects
1. Check your Cargo version: `cargo --version`
2. Build the project: `cargo build`

The projects have been confirmed to compile on rustc version 1.72.1 and cargo version 1.72.1.

To run, simply use `cargo run`.

## Testing
Execute tests using `cargo test`.