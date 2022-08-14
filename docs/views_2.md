### Problem:

I want to get rid of generic_display_state. Preferably also get_widget* too.

## what is it used for

It is used to implement reusable navigation within widget, deriving graph from representation.

## what went wrong

The rendering code now reads a list of WidgetIDRects (WIRS) from DisplayState (created on layout) and renders them
using "get_widget(wir.id)". It's a recurring pattern, it looks bad.

Furthermore, it is not possible to set focus to widget that is not visible in previous frame, because it's absent
in navigation graph.

## what is really needed from it

Well poiting to widget "visible in next frame" is a must have. Of course lifting requirement that "widget is visible"
means, that widgets have to have some "emergency mechanism", but we'll keep it internal for now.

I'd like to reduce interface of Widgets, get rid of
get_widget_*(wid) and subwidgets(), all the Bernardo Paradigm requires is a focus path. Focus path can be achieved with
a single self-reference, that can be implemented as boxed getter or sth like this.

If I really want to build graph from representation, the only part I need is "neighbours of my focused subwidget", not
all subwidgets.

And last, but not least, I don't need to draw it, I can generate it from layouts.
