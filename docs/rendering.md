# Rendering

This chapter describes some design choices about rendering that are worth knowing.

## Important types and definitions

- **Output** is a two-dimensional array of Cells. It can either be an in-memory thing or an actual framebuffer of your
  terminal
- **Cell** is the smallest unit of display. There two types of cells:
    - "Begin" cell, that defines a style (background and foreground colors, effects) and a character to be drawn (aka
      a "grapheme").
    - "Continuation" cell, that basically says "use style from my left and don't add any characters here, previous cell
      requires more than one column to be drawn".  
      There are some characters that are wider than one column even using monospace font, and in the
      beginning of this project I was ambitious enough to support them.
- **SizeConstraint** - in order to support scrolling, a widget needs to hear "use as much of that dimension as you want"
  . Therefore, a
  size of **Output** is given with **SizeConstraint**, which allows any of the axis to be unconstrained.

### More about SizeConstraint

Of course there is no infinite arrays in memory, draws beyond what can be seen result in no-ops. Nevertheless, I try
to optimise them out providing ```.visible_hint()``` method, that retuns a Rect corresponding to "actual surface
underlying this Output", but translated to **Widget**'s coordinate space.
It's also "abused" sometimes for cases where no good solution in infinite space exists. An example of such situation
is "NoEditorWidget" which draws a centered "no editor loaded" message. In order to center the message, a proper size
limit needs to exist, so ```.visible_hint()``` is used.

## Basics

An entry point to a widget rendering is obviously

```rust
fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output);
```

As you probably noticed, there is no "position" parameter. That's because all widgets are drawn in their own coordinate
spaces, that spans from (0, 0) to ```output.size_constraint()```.
This softly enforces the rule "widget cannot draw beyond it's allocated part of the screen".
The upper limits are enforced dynamically, but it is an error to draw beyond ```output.size_constraint()```, and in
debug mode
Bernardo will assert it.

But how this allocation is decided? Well, before render is called, few other things happen.

## Layouting: min_size and update_and_layout

### TL;DR

**Widgets** tell how big they are.  
In a two-step negotiaion process:

- first they say "below that I can't be drawn"
- in second step "ok, then within this budget, that's how much I want"

Decision of "how to use budget" is made by **Widget**  
Size of the budget (**SizeConstraint**) is decided by parent **Widget**.

That's it. Things can be simple. In your face W3C!

### Proper description

```rust 
fn min_size(&self) -> XY
```

is simple: it's a non-mut call that's supposed to tell parent widget "what's the smallest amount of
screen, where drawing this **Widget** makes sense". If there's less space available, widget will not be drawn.

```rust 
fn update_and_layout(&mut self, sc: SizeConstraint) -> XY;
```

is more complicated. First, it is **guaranteed** to be called with **SizeConstraint** that is greater or equal
than ```self.min_size()```.
Whenever you implement a custom layout, you must not break this rule.

This method allows **Widget** to prepare for drawing (hence the ```&mut```) and returns information "how much space
the **Widget** will use in this draw, under given constraint". Again, returning size greater than constraints allows is
an error.

Widgets themselves choose the way they use the space - some will decide to use "as much as possible", while other will
constrain themselves. Some will offer user a choice of policy via ```FillPolicy``` typed parameter.

Also, as you might have noticed, result of ```update_and_layout``` is XY, meaning that Widgets cannot be infinite. The
reason for it is simple: I couldn't find a valid reason to enable such **Widgets**, and supporting that case would break
my idea for scrolling (and probably some other things). 