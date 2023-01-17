# Layout

In previous chapters I wrote why Layouts are not Widget. So what are they?
They are pieces of common code that help to arrange **SubWidgets** within a parent **Widget**.

In order to avoid blocking entire **Parent Widget** while using ```&mut```, layouts are in simple terms functions

```
F : ParentWidget -> [(pointer to subwidget, where to draw it)]
```

where ```pointer to subwidget``` is also a function ```F : ParentWidget -> Subwidget```, or even more specifically a
pair of functions, because we might want to use ```mut``` or not.

This way layouts can be cloned (which is a cheap way to bribe the borrowchecker), stored etc, unbound to ```self``` of
parent Widget. This enables some flexibility while working on Bernardo, where I have not decided  "how much the
**Layout**s will be responsible for" or "when are they being called". A complex Widget usually defines a Layout-building
function that returns an unbound Layout, and we can use it multiple times in render cycle.

There is some common-code for complex **Widgets**, but it's not as mature as other parts of the library, and still
brewing. I am not particularly proud of that part of design, I still hope for some kind enlightment for simplification.

I may introduce a simple DSL to build layouts at later time. 