# Workspace

Unfortunately, not everything on the planet is written in Rust. We often have to interface with other languages,
sometimes even in the same repository. Furthermore, companies often have their own internal build systems, package
managers or other project specific tools.

Gladius' "workspace" definition is aims to act as an umberella over perhaps a larger family of "projects", as they are
defined in their own languages. The idea is that restarting Gladius with this file present "wires up" multiple "goodies"
with on-board systems.

## Status

Each item is marked as DONE, IN PROGRES or ABSENT. Not all design is final or even present.

## Requirements

A good IDE offers following functions:

- Calling a compiler \[ABSENT\]
- Seeing compiler errors, warnings etc in code (no, printed output is NOT enough) \[IN PROGRESS\]
- Code navigation \[IN PROGRESS\]
- Code completion \[IN PROGRESS\]
- Debugger \[ABSENT\]

Other cool features that are not planned yet sometimes include:

- Multiplayer editing \[ABSENT, but designed with this possibility in mind\]
- VCS integration (usually really bad)

First 4 are targets for Beta release. Debugger support is required for RC1. Multiplayer editing may make it to RC1 too.

## Details

Once run the first time in a directory, Gladius opens or creates .gladius_worspace.ron file, as
implemented [here](/home/andrzej/r/rust/bernardo/src/w7e/workspace.rs).

There is no formalized definition yet.

As all gladius configuration files, we're using [Rusty Object Notation](https://docs.rs/ron/latest/ron/) standard
because it validates types for us for free.

### Project Scope

A workspace has multiple "project scopes". Scope is (right now) just a set of parameters:

- Language
- Path (relative to *.gladius_workspace.ron*)
- \[Optional\] Handler Id

The idea is that you can have multiple providers of Navigation and/or Completion operating in the same workspace. They
are called **NavCompProvider**s.

**Handler** is what "handles a project". For instance in Rust, that's going to be a piece of code that parses
**Cargo.toml**.

Currently it's **Handler**'s responsibility to choose a right **NavComProvider**.

There are no assumptions on whether the paths have to be non-overlapping at this time,
though it could be that such requirement emerges for the sake of simplicity.

#### Status and details

Currently the only **NavCompProvider** implemented is LSPNavCompProvider, that uses "language server protocol".

It's covered with basic integration tests for CLangd and Rust. These tests live in [src/big_tests](src/big_tests) and
are run in specialized [test_envs](/test_envs) fake filesystems.

### Workspace generation

Gladius is finished, when it "just works". One of more annoying chores of IDEs is "I downloaded this project from
github, authors use this or that build system, how do I integrate it with my IDE to get navigation". We collectively
wasted hundred of hours on this.

Gladius implements something called "workspace inspection". The idea is that when run the first time in a directory,
where no **.gladius_workspace.ron** is found,
a series of "inspectors" will go through the filesystem and check for "most typical project definitions". If they can
figure them out, they will generate sensible workspace definition.

User can later update it according to their preferences. Furthermore, a method of prioritizing **Inspectors** and
implementing your own (most likely via subprocesses like LSP) is planned.

#### Status and details

Currently only Rust and C/C++ inspectors implemented. First looks for **Cargo.toml**, second for
**compile_commands.json**. 

