### Problem:

I have multiple places, where I reuse same boilerplate code, that:

#### 1) stores result of layout locally in the Widget. Not really super useful,

truth be told intermediate LayoutedWidget would be better.

It could be something like

```
struct LayoutedWidget {
    rect : Rect,
    widget : &dyn Widget,
  ...  
  is_focused?
}

```

#### 2) storing focus within widget also hurts: in editor view I have:

```
// this is a workaround to set focus into something not visible yet in next frame.
   deferred_focus: Option<WID>,
```

that's actually not really because of storing focus in widget. That is because "focus group"
contains WIDs of only "visible in last frame" Views, so in .update I may be missing one
I want to point focus to in next frame.

How could this be fixed? Well, I could throw away focus group, it does introduce more problems than
it solves. It was originally devised to automate focus transfer within a widget and cache layouts.
Caching layouts is a premature optimisation, I can throw it away immediately.
But idea that it would be cool to have semi automated focus graph builder is not bad.

I could actually leverage that idea with another one: invariant that subwidget C cannot be drawn outside of rect of
parent P, enables something alike "color picking buffer" method. It could serve as both "way to know where focus goes"
but also "way to know escalation path of widgets".
The three items seem to be interconnected.

Let's play with that idea: so imagine I have a multi-layer buffer, that defines following functions:

- each pixel knows what widget it belongs to
- each pixel knows "what was the pixel I covered", so knows parent widget
- each pixel can query it's neighbours (at least in top layer), to facilitate focus transfer.

now this invites idea of "let's not keep focus in widget", but then I'd have to write down super well semantics of
"moving focus" within the widget (say highlighting a button on "enter" hit in editbox).

Ok, we want the "focus within", current semantics is superior to "fake mouse", because it does not destroy intermediate
information of focus path within the widget on transferring out the focus. Although on the other hand, I expect that to
"degenerate" into pretty much the same thing - after all in order to "not consume" focus transfer input, I have to "hit
a wall" within given widget, so when focus is back to it, it will be pointing at that wall anyway. I could probably even
prove that.

Well without widget tree that's traversable "from outside", I could use much simpler semantics.

Hmm there is one thing I might need to consider: if I remove focus from widgets, I introduce a different issue:
imagine I have a list of editboxes one under another. I want arrows (up, down) to alternate between editboxes within
that list, and ctrl-up, ctrl-down "get out" from that "list widget". So I have several widgets "grouped".
I want ctrl-arrows to jump between groups, and arrows to be normal input within escalation path. Now it's easy -
internal widgets just ignore ctrl-arrows, they get escalated to right place. But if I remove focus path from widgets,
I loose that.

Wait, this is another cool idea - actually, I don't need a "light widget tree". I don't need widget tree at all, I need
"focus tree". This is the only tree I need to proceed with.

Not even a tree, just a focus path. And it does not have to be a "path of Rc<>s", it can be a chain of functions
"given widget P, get me widget C". And it can be built in a chain.

So a crazy idea is now like this: let's merge:

'''
let mut focus_chain : FocusChain = main_view.layout_and_render(output and bla bla);

each widget on it's layout_and_render take ONE of the focus chains it's subwidgets return, attaches itself, and returns
it back.

(wait for input)

focus_chain.apply(input); // this goes down to the last widget gets feeds it input, if input consumed, pushes the
message etc.

'''

I gave these ideas a spin and here is what came out of it:

1) the primary problem is focus

### solutions:

2) merge render and layout. This seems easiest. They are literally called one after another. The only reason they ever
   got separated was the fact that I did not want to have render with mut.

   Or maybe that I wanted to know "available space" BEFORE rendering? If sibling widgets
   A and B are drawn one below the other, B needs info from A "how much have you used".

But no, this is honestly too steep price to pay.