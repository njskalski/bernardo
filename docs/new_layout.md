# New Layouting

In this document I try to remove reliance on "visible hint" to the degree it's possible.

## SizePolicy

**SizePolicy** is a **Widget**'s setting that determines how it will use available space. It can be:

- self determined (DeterminedBy::Widget)
- imposed by layout (DeterminedBy::Layout)

The final size of Widget is determined with a following algorithm:

For each axis, we have following this matrix of options:

|             | DeterminedBy::Widget             | DeterminedBy::Layout                                            |
|-------------|----------------------------------|-----------------------------------------------------------------|
| Some(max_x) | Widget uses between size.x space | Widget uses EXACTLY max_x space, no matter how much data it has |
| None        | Widget uses size.x space         | Warning is emitted, same behavior as with DeterminedBy::Widget  |

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

Let's discuss how to implement Option2

1) change size to full_size()
2) move fill policy from widget to Layout's setting (LeafLayout and SplitLayout are enough)
3) change Layout signature from sc to pair (your decided size, your visible part != none)