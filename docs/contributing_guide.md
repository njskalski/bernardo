# Contributing Guide

A process of contributing to Bernardo / Gladius, typically follows a following plan:

1) Identify a ticket / feature you want to implement and claim it on Gitlab Issues list, so others know you are workig
   on it.
2) Fork the repository
3) Write your code and tests
4) Make sure your automated tests work in Gitlab CI
5) Make a Merge Request
6) Get feedback on your MR, apply fixes if necessary
7) Receive positive review from project maintainer
8) MR merged!

In this document, I will walk you through the part 3 of an imaginary "example task" to showcase the building blocks of
Bernardo and tools that are available to work on such task.

## Example workflow

Say we want to implement "viewing images" in Gladius, using block characters to display low-resolution render of a (
static) gif.

### Task

Definition of Done of the task is:

1) when a gif file is opened (either via tree view or fuzzy search), instead of editor, a ImageView is opened
2) ImageView has scroll
3) ImageView reacts to keys ```+``` and ```-```, doubling/halving the size
4) ImageView offers ContextMenu options corresponding to ```+``` and ```-``` inputs

Let's assume we found a nice library on [crates.io](http://crates.io) that does something like:

```
struct Gif {

    fn new(bytes : &Vec<u8>) -> Result<Self, ()>;
    
    // parameters x and y are floats in range [0..1] 
    fn get_color_at(&self, x : float, y: float) -> Color;```
}
```

Here is how I would implement this task:

1) Create two Widgets:
    1) First, called ```ImageWidget```, just grabs a ```struct Gif``` and displays it
    2) Second, called ```ImageView```, wraps ```ImageWidget``` with a scroll, reacts to i/o and replaces it with
       larger/smaller versions when necessary
2) Write tests to ```ImageView```
3) Add ImageView to Display enum in Main View [here](../src/widgets/main_view/display.rs).
4) Add code to MainView handling opening gif files
5) Add integration tests that entire Editor is behaving correctly

#### Step 1 - the two Widgets

First question - why two? That's because I don't want to reimplement scrolling. There is a wrapper Widget called
```WithScroll<W>```, that implements scrolling.

##### Widget 1 : ImageWidget

```WithScroll<W>``` implements scrolling in a quite unusual way, that is "by following the Kite". Every Widget has a function
```fn kite(&self) -> XY``` (defaulting to ```(0, 0)```) which means "the special position I really want to be displayed
in case of scrolling". In Editor - that's a cursor. In TreeView - that's the first character of higlighted position. On
layout, ```WithScroll<W>``` will take the least possible adjustment to accommodate for this requirement - that is move
the "visible rect" as little as possible to contain the Kite of internal Widget W.

What would be the good "kite" for ```ImageWidget```? We have two options. The easiest approach would be to highlight an
individual "pixel" (like a cursor) and follow that. But that's not a great UX.
If we want the scroll to immediately react to arrow right (as opposed to waiting for highlighted pixel to reach right
edge of image), we can use information that comes to ```ImageWidget``` in layout phase:

```rust
fn layout(&mut self, screenspace: Screenspace) {
    //screenspace.visible_rect contains the information about "how much of the widget is even visible"
    self.last_visible_rect = Some(screenspace.visible_rect);
}
```

The solution would be to save in ```ImageWidget``` information:

```rust
struct ImageWidget {
    //...
    size: XY,
    kite: XY,
    last_move_x: bool, // true if we last scrolled horizontally towards right, false otherwise
    last_move_y: bool, // true if we last scrolled vertically downwards, false otherwise
    last_visible_rect: Option<Rect>,
    //...`
}
```

When we process input left/right/up/down in ImageWidget, we use this information to:

1) in case the move is continued in direction of previous move, we just add/substract 1 from x/y coordinate in kite.
2) in case the move is made in direction different from the last time (on a given axis), we "jump" the kite position by ```last_visible_rect.x``` or ```.y```, to make it "move the scroll".

To add input handling to ImageWidget, we need several things:
1) a ```Msg``` type, that implements ```AnyMsg``` trait.
2) in ```fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>>;``` react to InputEvent's corresponding to arrows by emitting ```Msg```s defined in previous point.
3) in ```fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;``` consume these ```Msg```s and adjust the ```kite```.
4) add the overloaded ```fn kite(&self) -> XY```

##### Widget 2: ImageView

ImageView will have several roles:
1) create ImageWidget of the right size and put it into Scroll
2) handle input of '+' and '-' to change size.
3) pass the focus down to ImageWidget, so it can handle arrow keys input.

It will look something like this:

```rust
pub struct ImageView {
    internal : WithScroll<ImageWidget>,
    
}

impl Widget for ImageView {
    //...

    // These function make sure that Input is passed to ImageWidget
    // All events *not consumed* by ImageWidget, will be offered 
    // to ImageView immediately after.
    fn get_focused(&self) -> Option<&dyn Widget> {
        Some(self.internal.as_ref())
    }
    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget> {
        Some(self.internal.as_mut())
    }
    
    //...
}
```

Input handling follows exactly the same scenario as above:
1) we handle only the input we're interested in, here '+' and '-'
2) we convert it to InputView specific Msg
3) we handle these Msg in ```fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;```


##### Tests

At this point we have some code that we would like to test. Let's start with Widget test.

