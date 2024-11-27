# Vision

From the begining of Bernardo project, context menu was designed to be a "game changer".
It was also sometimes called "everything bar" and has it's own Input event.

There is an idea I called "context escalation", which means "I request context menu at any place, and get
all relevant options from the context of current cursor AND whatever surrounding contexts are available".

So for instance hovering over a function symbol in Editor, I would have available options from:

- NavComp for that symbol
- perhaps options from surrounding code symbol (say entire function body surrounding that symbol)
- all options from Editor
- all options from entire Gladius

Later I decided, that "escalate" (which was envisioned as a hotkey) is perhaps too complicated. It would
be quite easy to "overshoot" such used-all-time button. So instead I want fuzzy search in all contexts
combined.

# Bottom line

Here's what I need:

- a list of actions collected from root node down to end of focus path
- a "kite" of the focused widget, that informs "where the context menu should pop"
- a mutable reference to the focused widget so I can process the MSG from Context Menu

# Detail implementation

I will implement context bar on level of Main Widget.

It will act_on "everything_bar" input, grab the entire chain of "get_focused_mut", aggregate context actions,
when it reaches bottom get the kite.

Then it will use Hover layout to position "context menu" over the kite and offer all collected options.

Then, it will act_on action

## Some old thoughts (brain dump, don't pay attention to it):

Each of widgets on the focus path needs to be able to provide options for context menu.
Context menu itself needs to be positioned above or below "special cursor", or "kite", wherever it is
now.

Possible solutions:

1) recursive aggregation (somewhere at "act_on" call path)
2) focus path definition, that would enable "collection at the later date from within the widget".
3) some global bs.

#3 is a no go.

#2 - I wanted to write "that's impossible because of borrowchecker" but then I recalled that I have "subwidget pointer"
type that kinda solves the problem - I can just make a chain of "subwidget pointers" into a "focus path", keeping
all powers of Rust and pleasing the borrowchecker.

#1 recursive aggregation would require additional research.