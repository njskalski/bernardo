# Focus, Widgets, Input and Messages

This chapter describes relationship and rules between Widgets, Focus, Input and Messages. It's a heart and soul of
Bernardo TUI, and it's most mature component.

## Definitions

**Widget** is anything that implements Widget trait. You define one if you want something to be drawn.
As opposed to many other libraries, Layouts are not **Widgets**.  
**Widget tree** is a tree in a sense of data structures, but not represented as any particular type. We say that A is
parent
of B when A owns and contains B.  
**Focus** is "where the input goes first". It's hinted to the user with a color highlight.  
**Focus path** is a path in **Widget Tree**, always starting with the top one (root of the tree), down to the smallest
one that is highlighted. Entire **Focus path** should be properly hinted to user.  
**Input** are universal events that are delivered to highlighted **Widget** via *on_input* method. Only **Widgets** on
the **Focus Path** will be offered any input.  
**Message** is any type implementing *AnyMsg* trait.

## Rules

1. **Widgets** form a tree. This tree represents *at the same time* two different relationships:
    - ownership (parent owns child in borrowchecker sense)
    - geometric containment: all children are guaranteed to draw only within the area of parent.

   All widgets are rectangular.
2. Each **Widget** can point at most one of its children as a successor, this way forming **Focus Path**. No successor
   means "**Focus Path** ends
   on me". This is represented in code by methods:

    ```rust
    fn get_focused(&self) -> Option<&dyn Widget>;
    
    fn get_focused_mut(&mut self) -> Option<&mut dyn Widget>;
    ```

3. Incoming **Input** is first offered to the last **Widget** *A* on the **Focus Path** via method:

    ```rust
    fn on_input(&self, input_event: InputEvent) -> Option<Box<dyn AnyMsg>>;
    ```

   where exactly one of two things happens:

    - **Input** is consumed, and a **Message** specific to **Widget** *A* is created.
    - **Input** is ignored, and then offered to a predecessor of **Widget** *A* on the **Focus Path**, which is a parent
      of *A* in the **Widget Tree**. This process may continue up to the root of **Widget tree**, at which
      point **Input** event can be discarded as irrelevant.

4. A **Message** created by **Widget** *A* will be offered to **Widget** *A* in method

    ```rust
    fn update(&mut self, msg: Box<dyn AnyMsg>) -> Option<Box<dyn AnyMsg>>;
    ```

   This method is the only place, where **Widget** can modify its state. This however does not imply
   that visual representation of the **Widget** can't change between the frames without any **Message**s.
   A **Widget** observing filesystem will change its appearance between frames should filesystem change occur.

   A **Message** produced by *update* method will be offered to the parent of **Widget** *A*.

## Why

Primary driver behind development of Bernardo TUI was my dissatisfaction with how focus is handled by competitive
libraries.

I wanted to have multiple scenarios covered in a coherent way, without a combinatorial explosion of complexity.
Here are some of them:

1. A hover dialog (CompletionWidget) with code suggestions should capture arrows up and down, but should not capture
   letters.   
   How it's done: EditorWidget owns CompletionWidget and points to it in **Focus path**. Therefore CompletionWidget is
   the first one to consider the input. It consumes arrows up and down, but ignores letters. Then letters are offered to
   EditorWidget, which consumes them.

2. A "save as dialog" should block certain part of UI (what we are saving), but not all UI (not filetree, other editors
   etc.).  
   How it's done: simply SaveAsDialog is owned by EditorView (that decides to ignore all input as long as SaveAsDialog
   is present). However other parts of UI (ancestors and siblings of EditorView) can choose to consider input when it's
   offered. If user moves **Focus** to directory tree, it does not care that some EditorView is blocked.

3. "Save as dialog" has a result, that should be returned to whoever have created the dialog.  
   How it's done: *update* method of SaveAsDialog returns a **Message** to parent **Widget** containing the outcome of
   dialog. It can be "user cancelled" at which point parent will remove the dialog and not draw it again, or "user chose
   file X", at which parent will act accordingly (and then most likely close the dialog too).

## Other considerations

### 1. Layouts are not **Widgets**

There are multiple reasons for that:

- Ownership. Layouts would have to own **widgets** they lay out. So parent accessing it's descendant widgets to do any
  *&mut* operations would have to access them via some *&mut* method. This would either result in ugly calls like
    ```rust 
    self.split[2].split[0].set_value("something");
    ```
  Creating helper calls like *get_mut_widget_a* would not work, because they inevitably would have to take *&mut* of
  self, blocking access to all other fields.  
  Having direct *&mut* of two different child widgets at the same time would be
  impossible if they share the same layout.
- State representing focus. Consider for a moment following layout:
    ```
  ┏━━━━━┳━━━━━━┓
  ┃  A  ┃  D   ┃
  ┠╌╌╌╌╌╂╌╌╌╌╌╌┨
  ┃  B  ┃  C   ┃
  ┗━━━━━┻━━━━━━┛
    ```
  let's assume we have three split layouts: top one that splits along vertical line (L1), and two child layouts that
  split along horizontal ones (L2 on the left and L3 on the right). We want to make a full circle with our focus from A
  counterclockwise, so A,
  B, C, D, A.  
  When moving our focus from A to B, the L2 layout would remember "my focus path points to B". On transfer from B to C,
  L1 would update its focus pointer from left to right.
  Move from C to D can occur without problem, but on coming back from D to A, L1 would point again to L2, which
  remembers that it points to B. We have a "jump" of focus, unintuitive from users point of view.

### 2. Subscriptions

Interesting part of Elm/PureScript design that I did not properly consider was the term of "subscriptions", which are
messages generated by external services and fed into the Widget. This would simplify reasoning, I might want to adopt
it. 