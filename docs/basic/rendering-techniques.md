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
- **visible_rect** is an argument showing up in multiple function, used for rendering optimisation. It's always
  non-empty.
- **SizePolicy** is a **Widget**'s setting that determines how it will use available space. For each axis (X, Y):

    - self determined (DeterminedBy::Widget)
    - imposed by layout (DeterminedBy::Layout)

## Basics

The size of Widget is negotiated between it's and Parent Widget usually with help of Layouts, but not necessarily.

The long story short is that Widget has two options:

1) it can either say "whatever, I'll accept any size to render" OR
2) it has to decide "my full size" without knowing any limits.

If requested "full size" is beyond what is possible to draw in current frame, Widget will NOT be drawn, and a
replacement will be drawn instead. Most of the time, Widgets will either declare "whatever" or be covered with "
scrolls", enabling any size they desire.

### Full cycle

Cycle of drawing of widget is multiple calls:

1) First, we call

```rust
fn prelaout(&mut self)
```

This is a call which widgets use to pull data from sources other than message system. I am still debating if I don't
want to move to "subscription" model like in Elm / PureScript.

2) Most of the times

```rust
fn size_policy(&self) -> SizePolicy
```

is consulted. If it says "use whatever layout decided" then step nr 3 doesn't happen. Otherwise (on at least one axis)
it does.

3) Most of the times (but not always)

```rust
fn full_size(&self) -> XY
```

is called. This call is supposed to return A FULL SIZE of the Widget. Not a minimal size, not a maximal size, but "
imagine there is no limits, how much would you use" size. There is no way to inform Widget ahead of time "what is the
limit" ahead of that call (there is a very good reason for that in [here](braindumps/history_of_new_layout.md), but long
story short,
lifting that requirement is a can of worms it took me hours to clean up). If this is called, most likely widget will be
acomodated with a Scroll, so its wishes are met no matter what they are.

4) Then (always), layout is called

```rust
fn layout(&mut self, screenspace : Screenspace);
```

When it's called, one of the two things are guaranteed: either Widget declared "whatever size" via ```size_policy``` OR
```output_size``` is at least as big as ```full_size()```.
Furthermore, visible_rect is not degenerated, that is - it's at least 1 cell big. We do NOT layout/render widgets, that
will not be visible.

This is the call, where Wideget is supposed to do all the calculations before rendering. It's NOT supposed to change
anything else, but I can't enforce that.

We are not interested in any results of Layout. Widget that was called to Layout will be drawn. It cannot change the
contract it communicated in steps 2 and 3 at this point.

```visible_rect``` can be used for such thing as "cenering the label", but really - shouldn't.

5) An entry point to a widget rendering is obviously

```rust
fn render(&self, theme: &Theme, focused: bool, output: &mut dyn Output);
```

Output carries the parameters of ```size()``` and ```visible_rect``` that are guaranteed to be identical to ones
in ```layout``` call.

As you probably noticed, there is no "position" parameter. That's because all widgets are drawn in their own coordinate
spaces, that spans from (0, 0) to ```output.size()```.
This softly enforces the rule "widget cannot draw beyond it's allocated part of the screen".
The upper limits are enforced dynamically, but it is an error to draw beyond ```output.size()```, and in
debug mode Bernardo will assert it.

## Scrolling

Widget's expose a method

```rust
fn kite(&self) -> XY;
```

How to think about it: imagine you are kite surfing. Kite is the opposite of anchor - it's in the air and you follow it.

Widget can tell "of all the things rendered, this one pixel is the most important". A scroll is supposed to make "least
effort move to accomodate for that".

A ```WithScroll<W: Widget>``` renders ```W``` in "as much space as needed" output, and then shows a finite window of it.
A window
moves as little as possible from it's previous position to a new one in such the direction, that ```W.kite()``` is
always included in that window.

What is kite? Usually some cursor. If you want to scroll around the file, you have to move the cursor.

## TODO

- implement contract validation