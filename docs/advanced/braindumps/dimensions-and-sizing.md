# Understanding Widget Size and Depiction

The essence of a widget's size is critical in determining its FocusGroup from its Layout. As we delve deeper into the concept, it becomes evident that the Layout itself could undergo modifications based on the size.

## The Concept of "Depiction"

Let's introduce a term: "Depiction." In the context of a widget, the input it receives is always interpreted relative to its most recent display state, which we'll designate as its "Depiction". This Depiction is derived from various parameters, such as the output size, theme, focus path, among others. If these parameters remain unchanged, the previous Depiction can be conveniently reused.

It's imperative to understand that any input that alters the state will invariably invalidate the previously cached Depiction.

## The Widget Lifecycle

To comprehend the nuances of size and Depiction, let's chart out the typical lifecycle of a widget:

1. **Creation**: The widget comes into existence.
2. **Size Query**: The widget is prompted to provide its minimum size requirement.
3. **Layout Evaluation**: Once the widget's minimum size constraint is met, it is informed of the "actual size available." This moment is pivotal as it's the earliest stage where the Layout can (and should) be determined. At this juncture, the calculation of the Focus Group is optional but can be performed.
4. **Rendering Phase**: The widget undergoes rendering based on its size and layout.
5. **Input Phase**: The widget might receive inputs that can influence its state.
6. **Communication Phase**: There might be messages or feedback from child widgets or components.

Translating this into a code-based perspective:

- Step 2 translates to `widget.min_size`.
- Step 3 becomes `widget.layout`. If the widget relies on a Layout helper trait for recalculating its sub-widgets, it triggers `Layout.sizes()`. Given that Layout involves a mutable call, `sizes()` must access its "owner" in a mutable fashion.