# Bernardo / Gladius

This is a repository currently hosting two projects:

- [Bernardo](docs/bernardo_tui/index.md) is a TUI widget library.

- [Gladius](docs/gladius/index.md) is a code editor.

Click in one of them to jump to specific documentation. 

## Status

Both projects are in beta state, they *will* crash, save often.

Beta means "all key functions are implemented, but they are not of their final quality".

The project turned out to be around 5x larger than originally estimated. Therefore a lot of code is of "power through" quality: I implemented something "quick and dirty" that works + a test that it does not break tomorrow.

Only when the project is "complete enough" I will start targeted optimizations. A targeted optimisation is one informed by a proper profiling (flamegraph), preferrably of an automated test run on loop ~1000 times. Any optimisation absent of such experimental information is considered "spray and pray" and I really have no time to even discuss that.

## License

Licenses: [GPLv3](COPYRIGHT), with a target to re-release Widget Library of Bernardo as [LGPLv3](https://www.gnu.org/licenses/lgpl-3.0.en.html) at
later date.
If you decide to contribute, be sure you are OK with that. That's the only "CLA" required.
However, the text editor widget will remain GPLv3 without 'L'.

Here is a website describing reasoning behind the projects: [Triarii](https://njskalski.gitlab.io/triarii)

## Contributing guide

You want to contribute? Amazing! I wish I've met you earlier. To help you start, I wrote a [guide](docs/contributing_guide.md).
