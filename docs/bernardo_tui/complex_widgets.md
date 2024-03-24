# Abstract

There are multiple complex widgets. The most prominent example is EditorView that consists of:

- Editor widget
- Find/Replace editboxes
- Completion bar
- Context bar (go to definition, find usages etc.)

Other examples are:

Save as View which consists of:

- TreeView
- ListView
- Buttons
- Editbox

There are two major issues ComplexWidget trait is trying to solve:

Focus and Input. They are handled very differently.

At any given time, we need two informations:

- in what order widgets are to be offered input? (Focus Path)
- what is the widget to be selected when I do ALT+Arrow

## Focus

A ComplexWidget stores "Display State", that remembers sizes and positions of Widgets, and generates "Focus Graph".

Focus Graph is a structure that tells "left from widget A is B" and "down from widget A is C".

When user decides to use ALT+arrow, we update focus path within ComplexWidget to point at newly highlighted Subwidget
using Focus Graph.

### Input

Complex widget does not implement default Input policy. The idea is that Focus Path will cause Bernardo to offer Input
first to the Subwidget selected via Focus Path.

This however leaves one scenario unsolved:

Imagine NestedMenuWidget enriched with EditBox that shows the query:

       ┌───────────────────┐
       │ go to definition  │
       │ find usages       │
       │ some other option │
      ...                 ... 
       │[editbox for query]│
       └───────────────────┘

Here is the issue: we want to route input from ComplexWidget to query Editbox, but then - if it's not consumed - we'd
like to offer it to nested menu above.

Within current Bernardo paradigm, this can be implemented in one, and one way only:
by making Editbox a child of NestedMenu. If however ComplexWidget is involved, they are siblings.

---
**Wait, the problem is even more serious.** We want to **FILTER** the messages. Arrows are supposed to go to nested
menu, and letters to EditBox.
---

This can be solved in two ways:

1) Parent widget defines "Input Routing" (a superset of information that's now FocusPath)

   It seems to be true, that Input Routing is required only in situations, where both widgets that would get
   the input are "highlighted simultaneously", that is they are merged into one. Like Editor view with hover, or
   NestedMenu with query.

   If they are *highlighted simultaneously*, it means there is no valid focus transfer between them.

   **Fuck, this is it!** ComplexWidget != CombinedWidget.
   ComplexWidget has focus graph and defined input. CombinedWidget has "all of us or none" for both.

   But the issue remains: what if one of the two widgets being parts of a combination is complex itself? I'd still
   like to be able to offer the input to the "deepest possible ancestor" and then back up.

   I cannot "change the order of consideration contract" mid-journey. It seems like I DO WANT to be able to route input
   from widget, just like path, **perhaps with filtering**.

   Wait, I think I can get rid of filtering, if focus path is DFS of subwidgets. Just change

```rust
fn get_focused_mut(&mut self) -> &mut dyn Widget;
```

to

```rust
fn get_focused(&self) -> Iterator<SubwidgetPointer<Self>>; // this can even be a vector.
```

and we're done.

## Focus transfer (focus graph)

At any given moment, user sees a rendered depiction, and has at most 4 allowed (arrow) moves.

All we need to do is to properly generate and react to this information.

Reaction is "update of focus path".

I think it is safe to assume that moving focus between widgets should not change the state of widgets. Within - OK, but
not between.

The questions are:

- Do we want to move building focus graph to recursive_treat_views (outside of widgets)?

The intermediate data to generate focus graph *is in the widgets* but it's also there *before the redraw*.
It seems like both focus sub-graph generation and redraw should be done together By "sub graph" I mean just the
information "when you go alt+arrow, this information goes to X and does Y".

Let's think for a moment if I can externalize this (move it out of the widget).

```
11111..................       1───2──────────────────┐   
11111..................       |   |  4───+────────┐  │
11111....55..6666......       |   |  │   │ [     ]│  │
11111....55............  from |   |  │   │        │  │
11111....55..7777......       |   |  └───+ [     ]┘  │ 
11111..................       |   |                  │
11111..................       └───+──────────────────┘
```

Say for a moment I processed ALT+Arrow outside Widget 7, say I know I want to move left to 5.

Now we have two options:

1) if focus state is stored in widgets, we need to go to save-as-dialog 4 and let it know "dude, update your focus from
   7 to 5".
2) if focus data was outside the widgets, we would just move a marker in an externalized focus graph.

How would it look like, this graph, and how would it be built?

If it was built top-down, it would be generated from chained subwidget-pointers stored in "color table".

If it was built bottom-up, it would come into being by wrapping results of subwidgets.

Top-down would require some kind of facility to drill-down-back, it would be the data for "recursive treat views". Focus
would become externalized.
Also, we could end up in a situation where I do "go left", but between the frames widget to the left ceased to exist. So
this chain would have to be "fallible". That's not a problem I guess.

---
Right now the issue is that I have externalized Input, and internalized Focus.
---

The cool invariant is that we have a strict EITHER between ALT+Arrow and any other key combo. Meaning that user EITHER
changes focus OR does any other input.

Meaning we could externalize focus too on redraw.



---

# Old stuff, do not read

One important piece of information we have in 12_focus.txt is:

"set focus needs to succeed even BEFORE the widget is drawn for the first time".
Meaning that waiting for focus graph to be generated is too late.

### Perfect DSL

Let's think how in perfect-dsl would this look like:

```
SaveAsDialog {
    VerticalSplit {
        TreeView,
        HorizontalSplit {
            FileList
            FileName
            VerticalSplit {
                Button1
                Button2
                Button3
            }
        }
    }
}


```

The question "what is the widget to be selected when I do ALT+Arrow" has two subquestions:

1) how this information is produced
2) where intermediate data is stored

Let's start from 2. Do I need intermediate data?

```
+---+---+
| 1 | 2 |
+---+---+
| 3 | 4 |
+---+---+
```

This classic example where I would like to be able to do a circle 1-2-4-3-1 says "no, if you store intermediate data in
tree it will break".

Actually, at any given moment, all I need is at most 4 edges of this graph. They can be derived from last redraw.

Let's assume we have an **INVARIANT that ALT+Arrow is never changing state other than focus**.
However other is not true - an action on Widget can change focus. We'd probably want the background tasks to NOT break
focus, but what do I do if I was looking at a file that ceases to exist? Crash?

Anyway, I can say "users move don't mean actions", but "actions can update focus". Nevertheless, user interacts **always
** with LAST DRAWN STATE. Meaning that if "four arrow information" is generated at draw - that's fine.


---

The question is "what would be the data type of this picking buffer". The default option "widget id" sucks, because
reconstructing a &mut handle from Id is tough.

Alternative would be some kind of getter-chain. I already have "subwidget!" macro, I could chain those.

[//]: # (I guess the question is the type. Say I have Chain -> dyn Widget.)

## Proposed solutions:

1) Extend paradigm, making "Focus Path" not exclusively from bottom of the tree upwards, but perhaps more like "Depth
   First Search tree ordering".

   After such change, ComplexWidget would put on "stack to offer input" first Editbox and then Nested menu.

## Let's discuss the alternative

In a braindump 12_focus.txt, I wrote that perhaps the focus shouldn't be stored in Widgets.
Widgets can define focus path/stack and widget relationships, but maybe Focus Graph should be global.

Focus Graph can be derived from last redraw, like a "color picking" algorithm in video games.

Some widgets would be active some not (if blocked by a modal).

So we could get a buffer like this:

```
11111..................       ┌───+──────────────────┐   
11111..................       |   |  ┌───+────────┐  │
11111....55..6666......       |   |  │   │ [     ]│  │
11111....55............  from |   |  │   │        │  │
11111....55..7777......       |   |  └───+ [     ]┘  │ 
11111..................       |   |                  │
11111..................       └───+──────────────────┘
```

And focus stack

(say I am pointing at 6)

```
2 (right pane) (does not accept input)
3 (save as dialog) (does not accept input)
6 (pointing at upper edit box)
```

Now when Dialog 3 ceases to exist, focus jumps back to right pane (2).

This can actually be quite nicely implemented by just changing signature from "get_focused -> subwidget" to "
get_focused -> iterator_of_subwidgets"

And focus graph would be:

```
1 -(right)-> 5
5 -(right)-> 6
6 -(down) -> 7
6 -(left)-> 5
7 -(left)-> 5
```

So going from 7 left and back right would lead not to 7 but to 6. Which I guess right now is the same.

We also have focus transfers. So for instance hitting Enter in 5 leads to "focus transfer" from 5 to 6, that's something
Widget 3 takes care of.

questions:

- Would messages still from bottom-up exclusively, or would we allow them to cross to siblings too?

  Right now neither inputs nor messages can be routed down, this functionality is in recursive_treat_views

I guess the question is:
