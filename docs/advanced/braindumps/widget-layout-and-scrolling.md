Up until recently, Layout call on Widget had a following semantics:

widget.layout(max_size) -> xy {
    // you have up to max_size of space,
    // return how much you used up.

    // The widget should use as much space as it can in a reasonable way.
    // Reasonable means - not rendering gibberish.
    // So for instance a button should not stretch beyond reason, but
    // a multi-column list should use all width to space the columns, and just as many rows as it has data.
}

Now with scrolling it becomes tricky. On one hand I could say, max_size = (u16::max, u16::max), but then any spacing
algorithm would fill the huuuge space and render probably blanks.

So I need a new schema: max_size has to become something like Some(x), Some(y), where None means "use as much as you
want". And then I delegate it to the widget to use "comfortable" amount of space and do not over-space above it.


