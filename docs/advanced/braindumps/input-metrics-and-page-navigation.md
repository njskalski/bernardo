# Handling Input: Size and Page-Up Considerations
Managing input events such as page-up and page-down within widgets, like the List widget, necessitates an understanding of the visible portion of the list. This becomes challenging because, as of now, widgets don't inherently possess information about their size.

## Potential Solutions
1. **Direct Size Integration**: A straightforward solution would be to incorporate a `Cell<XY>` within `widget.size(max_size: XY)`. Alternatively, we could consider `size(&mut self)`, but this would involve retaining the last frame state in widgets, which may not be ideal.
2. **Adapting React's Approach**: Taking inspiration from the React library and its adaptation in `tui-react`, we can distinguish between the widget's state (`state`) and parameters passed from the parent (`props`). While I haven't explicitly implemented `props`, the output size seems to align with this category.

## Proposed Enhancement
I'm inclined to augment the `InputEvent` to include details about the output size. There's also consideration to include the focus path, but it's debatable if a widget should have that awareness. The question arises: Should a non-focused widget ever receive input? As of now, my assumption is that it shouldn't.

## Contextual Input
It's essential to recognize that any input operates within the context of the most recent frame presented to the user. This context is pivotal in ensuring that user interactions are both meaningful and accurate.
