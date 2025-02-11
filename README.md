# Bernardo / Gladius

This is a repository currently hosting two projects:

- [Bernardo](docs/bernardo_tui/index.md) is a TUI widget library.

- [Gladius](docs/gladius/index.md) is a code editor.

Click in one of them to jump to specific documentation.

## Status

Both projects are in beta state, they *will* crash, save often.

Beta means "all key functions are implemented, but they are not of their final quality".

The project turned out to be around 5x larger than originally estimated. Therefore a lot of code is of "power through"
quality: I implemented something "quick and dirty" that works + a test that it does not break tomorrow.

Only when the project is "complete enough" I will start targeted optimizations. A targeted optimisation is one informed
by a proper profiling (flamegraph), preferrably of an automated test run on loop ~1000 times. Any optimisation absent of
such experimental information is considered "spray and pray" and I really have no time to even discuss that.

## How to build it

```bash
apt-get install curl build-essential gcc make clangd clang nodejs python3-pip python3-venv
git submodule init
git submodule update
cargo build --release
```

To get Rust Language Server protocol:

```bash
rustup component add rust-analyzer
```

To get othe LSPs, check [pipelines](.forgejo/workflows/everything.yaml)

## FAQ
### Code completion doesn't work
Most likely it's an issue in .gladius_workspace.ron file. It should specify which LSPs should be used.
So first, navigate to .gladius_workspace.ron file in you project. It'll be generated after first run of gladius on your project. If autocompletion doesn't work, likely it's gonna look like this:  
```
(
    scopes: [],
)
```
It means, that inspectors didn't manage to capture which languages should be supported.  
So assume your project is written in Rust. Your .gladius_workspace.ron file should look like this:
```
(
    scopes: [
        (
            lang_id: RUST,
            path: "",
            handler_id_op: Some("rust"),
        ),
    ],
)
```
You can run gladius with -r flag to reconfigure after this change.  
That should fix the problem. You can see in [test_envs](test_envs/) more .gladius_workspace.ron files for other languages.  

### X stopped working after modification/pull.
Simply building project after modification may not be enough, especially if key bindings have changed. Remember to reconfigure gladius by running it with -r or --reconfigure flag. It will create a new config and rename previous one to "old" version. You can inspect them in ~/.config/gladius/

### How to run tests locally?
First install forgejo-runner. [helpful link](https://docs.codeberg.org/ci/actions/)  
Then run 
```
./forgejo-runner exec
```  
Alternatively you can call
```
cargo nextest run --no-fail-fast --retries 4
```  
but the first method is recommended.

### Missing dependecies
If you encounter a bug and you suspect you're missing a dependency (like missing LSP), for now inspect [this config](.forgejo/workflows/everything.yaml)

## Primary repository

The project used to be developed at [Gitlab](https://gitlab.com/njskalski/bernardo), but I am in the process of
migrating it to
[Codeberg](https://codeberg.org/njskalski/bernardo) due to "being able to run the runners locally".

There are out-of-date copies at Gitlab, sr.ht and other services that I evaluated.

## License

Licenses: [GPLv3](COPYRIGHT), with a target to re-release Widget Library of Bernardo
as [LGPLv3](https://www.gnu.org/licenses/lgpl-3.0.en.html) at
later date.
If you decide to contribute, be sure you are OK with that. That's the only "CLA" required.
However, the text editor widget will remain GPLv3 without 'L'.

Here is a website describing reasoning behind the projects: [Triarii](https://njskalski.gitlab.io/triarii)

## Contributing guide

You want to contribute? Amazing! I wish I've met you earlier. To help you start, I wrote
a [guide](docs/contributing_guide.md).
