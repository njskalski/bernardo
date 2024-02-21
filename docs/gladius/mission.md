# Mission Statement

Gladius is editor that "just works" and "fucks off".

"Just works" means: you don't need to troubleshoot to figure out why code navigation doesn't work. It should "just work"
with majority (say 80%) of projects found in the wild with **zero config**. That's a ambitious goal, but by no means
impossible. Just far out.

## "Fucks off" means:

### No stupid modals

Have you played Settlers 2? In 1996 we had a UI that keeps you informed without being annoying. Because UX in video
games is taken seriously.

### No crashes

In RC1 (far in the future), no unsafe code or unwraps will be allowed. Furthermore, each external system (like Language
Server Protocol provider) will be tested with "Bad Actor" (so something that slows down, crashes, returns malformed data
etc.) to make sure all such scenarios are properly handled.

### No desructive defaults

Typing fast should be fearless. If we absolutely have to ask you a question ("do you wish to override file X?"), the
option highlighted by default in TUI will be NO.

### All slow actions async and interruptible

Slow hard drive, network or LSP should **never** cause a slowdown in redraw. We will show you "working on it" icon and
you can hit say "actually, don't bother" (ESC) button **always**. If you have to write 'kill -9 gladius', that's a bug
and we need to fix it.

### No stupid tutorials

Nobody has time to take 6 weeks sabbatical and attend an online course of VIM, Emacs or any other "proper way of doing
things". We have plenty of standard UI expectations, developed through 80s and 90s. Like CTRL-C CTRL-V. Or ESC.

If you bought a car and it had a joystick instead of steering wheel and buttons instead of gearbox, you would return
that shit right away and warn all your friends "this manufacturer went mad".

### No mouse

Mouse is a tool for graphics designers, radar operators and architects. Not for coders. If your IDE requires mouse,
you're not doing it right.

### No setup

All keybindings should be as common and natural as possible.

A setup should be disposable and easy to reproduce.

No debugging of scripts.

### No flow-killers

Have you ever wondered what all these options like "View Page source", "Inspect" or "Refresh" are doing in your
"progressive web app streaming music" right-click (context) menu?
Like what percentage of users want to "Inspect Accessibility Properties"? Buttons like "Forward" or "Next" will break
20% of web pages.

These options are out of their place and cause more trouble than solve.

If you open context menu in IntelliJ Clion, it shows 15-20 options, depending on where you click. Among them "Clean
Python Compiled Files" (in C++/Rust IDE!), "Create Snippet", "Create Gist" or "Diagrams". I never used any of them. The
only
role they play is to
make me look for the option I want longer and kill the flow.

Gladius will:

1) show only options **relevant to curent context**
2) never allow some plugins to throw random stuff where your attention goes
3) offer fuzzy text search among the options
4) allow to fluently "escalate context", which means "no, I didn't mean this symbol, I meant entire paragraph" **clearly
   highlighting the context** of menu

### No opening web browser

If you have to go to StackOverflow or ChatGPT web page to ask for something like "how do I open file in Python", that's
also killing the flow. Most websites make money trading your attention, so they are designed to get as much of that raw
resource as possible.

Simple facts specific to context of your work should be available from within, without need for additional tools.