# New Layouting

In this document I try to remove reliance on "visible hint" to the degree it's possible.

## SizePolicy

**SizePolicy** is a **Widget**'s setting that determines how it will use available space. It can be:

- self determined (DeterminedBy::Widget)
- imposed by layout (DeterminedBy::Layout)

The final size of Widget is determined with a following algorithm:

For each axis, we have following this matrix of options:

|             | DeterminedBy::Widget     | DeterminedBy::Layout                                            |
|-------------|--------------------------|-----------------------------------------------------------------|
| Some(max_x) | Widget uses size.x space | Widget uses EXACTLY max_x space, no matter how much data it has |
| None        | Widget uses size.x space | Warning is emitted, same behavior as with DeterminedBy::Widget  |

But if this is determinant (and it seems so), does widget even need to return result? Not really, we can guess it's
size.

If we can guess it's size, then passing SizeConstraint to layout is kinda pointless, widget can be externally "
informed":
given the settings, "this is how much space you get".

And then we arrive to another question: should the setting of SizePolicy be assigned to widget or to layout?

Does Widget ever need this information? Will it adjust its behavior to given SizePolicy and SizeConstraint?

Right now - yes. We have Editor widget that uses these information to decide where to draw "Overlays" (hover).
We have NoEditor widget, that centers the label, but that can be layouted too.

## Why we have visible_hint()

Which widgets use SizeConstraint.visible_hint():

- Editor for hovers
- WithScroll for deciding "how broad margin to draw for the line numbers", this can be replaced with child.size() tbh
- BigList and List widgets for culling.

That's it. So hovers and culling.

### Option 1 "this is your decided size" WITHOUT visible_hint

- Editor would not know how to position hovers close to edges of screen.
- WithScroll would have to have fixed margin width (no problem)
- Lists would spend time rendering invisible items.

### Option 2 "this is your decided size" WITH visible_hint

- Nothing changes in Editor
- Nothing changes in Scroll
- Nothing changes in Lists

I went with this, and here are the issues I have found:

I moved SizePolicy to Layout, and now I don't know how to implement WithScroll that acts like a widget.
Why? Well I have to decide FullSize of WithScroll not knowing "how much space is available".

What would happen if (again) I'd add return value to widget "here is your max, give me your size", and move the ...
ahh, I'd get "unbounded" issue again - how can I limit the widget not knowing it's size. FUCK.

So there is a solution, Widget can say "as much as possible" in their full_size(). How would that work:
Widget returns "potentially unbounded" full size. Layout decides "this is your quota" and sends the info back to widget.

But that's the problem back again: if widget tells me "as much as possible" and I have scroll, I can't give it back
"ok, this is your quota", because there is no limit.

One of the parties have to decide AHEAD of time their size. The information flows from the TOP, from the Layout,
because only screen size is known.

Widget has to know it's full size. Now how do WithScroll/Editor knows "what's my desired size".
If I do full expensive calculation of true size of widget within WithScroll and Editor, would that
solve the issue?
No. WithScroll would not know what to communicate upwards. It could easily say "too much" and get replacement
drawn. I could handle this widget separately, but that's a disaster.

I could also remake scroll into a stateful layout. The layout has the information WithScroll needs.

Wait, again if I tell widget "give me your size given this limit" and it chooses to use everything,
I do have the information I need by the time I create an output. Ah, but I can't communicate the limit to widget
if I have a scroll, because scroll means I have no limit.
So letting widget choose it's size given "constraint" implies the hell I just removed.

OK, options:

1) WithScroll becomes a Layout
2) Widget returns a PAIR (my true full size, my preference for alignment)

Can we decide based on that?
We can scale up Editor and Scroll and List to whatever layout can do.
Scroll can ask widget below for true size, and ignore preference... YES. This can fkn work.
