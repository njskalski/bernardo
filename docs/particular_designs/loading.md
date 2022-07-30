# Gladius Loading Procedure #

Gladius is started with zero to many parameters. They are of following kinds:

- global parameters \[zero or more \]
- file inputs \[zero or more \]
- directory inputs \[ zero or one \]

Parsing *Args* structure should arrange them in proper buckets.

## Workspace root

First, **workspace root** is determined:

* If more than one directory is specified, a program immediately terminates with an error
* If a single directory is specified, it becomes **workspace root**
* If no directory is specified, OS's current directory (cwd) becomes **workspace root**

## Opening files

All files within **workspace root** directory or it's ancestors, will be attempted to be opened.
Failures will be logged to **standard log**.

All files specified outside **workspace root** will **not** be opened, and corresponding errors will be sent to
**standard log**.

## Opening Workspace

Gladius defines it's Workspace format in *w7e* module. It's designed to be lightweight and delegate
language specific options immediately to dedicated **language hanlders**.

Workspace is defined within a single *.gladius_workspace.ron* file in **workspace root**.

At current state, nesting workspaces is **not** supported, all subsequent (ancestors) workspace
definitions will be treated as files.

### When workspace file is found

Gladius will attempt to restore all workspace settings. This process can succeed completely, partially, or completely
fail.

**In no case however Gladius will attempt overwriting workspace definition** without consulting
user first.

* In case of complete failure to load .gladius_workspace.ron, program **stops** immediately with appropriate error
  message.

  A standard course of action in such case is to rename workspace file, and try to recreate it from scratch.

* In case workspace loaded successfully, but one or more **language handlers** failed to initialize given provided
  options, a partial load happens.

  Errors produced by **language handlers** will be produced to **standard log**.

### When workspace is NOT found

Standard procedure in case of workspace not being found is to attempt to create one.
Assistant to this process are **ianguage nspectors**

#### Language inspector

Language inspectors are defined in *w7e* module. Their job is to **quickly** determine whether a corresponding **
language handler**
should be invoked for particular directory or not.

To avoid explosion of filesystem requests, only directories at certain depth will be
inspected. A match from one inspector immediately stops recursion into subdirectories.
One directory might however be matched by more than one **inspector**.

### Language handler

Each supported programming language have their own way of handling their projects.

**Job of language handler** is to provide common interface to these using specialized,
language specific tool, like "cargo" or "cabal".

These tools usually define a project file or directory, sometimes "workspaces" (in such case I use quotation to
distinguish them
from Gladius **workspace**s).

No matter whether loaded by **workspace**'s **project scope** or created ad-hoc at request of **language inspector**, it
is job of **language handler** to provide language specific **advanced options** to particular directories of
**workspace**.

These typically include:

- run configurations
- code navigation
- code suggestions

**Language handler** is invoked with top most directory identified by **inspector** OR the directory defined in
**project scopes** in Gladius **workspace** file.