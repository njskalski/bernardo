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

## Layouting: size() and layout()

### TL;DR

**Widgets** tell how big they are.  
In a two-step negotiaion process:

- first they say "this is how much space I need to be fully drawn"
- in second step "ok, then within this budget, that's how much I want"

Decision of "how to use budget" is made by **Widget**  
Size of the budget (**SizeConstraint**) is decided by parent **Widget**.

That's it. Things can be simple. In your face W3C!

### Proper description

```rust 
fn size(&self) -> XY;
```

Tells the full size of the widget, based on all available information at the time of call. Widgets **should not** return
a portion of their full size and implement scrolling themselves. Widgets **should** return full size, and refuse to
render in case there's not enough space, to force programmer to use proper scrolling wherever necessary.

Guarantees:

- called each frame before layout()

```rust 
fn layout(&mut self, sc: SizeConstraint) -> XY;
```

This method allows **Widget** to prepare for drawing (hence the ```&mut```) and returns information "how much space
the **Widget** will use in this draw, under given constraint".

Widgets cannot be infinite, hence XY as return type. Reason: scrolling.

Widgets themselves choose the way they use the space - some will decide to use "as much as possible", while other will
constrain themselves. Some will offer user a choice of policy via ```FillPolicy``` typed parameter.

Guarantees:

- SizeConstraint >= self.size()
- called each frame before render()

Requirements:

- return value <= SizeConstraint

## Scrolling

Widget's expose a method

```rust
fn kite(&self) -> XY;
```

How to think about it: imagine you are kite surfing. Kite is the opposite of anchor - it's in the air and you follow it.

Widget can tell "of all the things rendered, this one pixel is the most important". A scroll is supposed to make "least
effort move to accomodate for that".

A ```WithScroll<W: Widget>``` renders ```W``` in *infinite* output, and then shows a finite window of it. A window
moves as little as possible from it's previous position to a new one in such the direction, that ```W.kite()``` is
included in that window.

What is kite? Usually some cursor. If you want to scroll around the file, you have to move the cursor. But "that's
destructive". Don't worry, attention tree got your back.

Requirements:

- self.kite() <= self.layout(...)

## TODO

- implement contract validation