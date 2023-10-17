# Evaluating the Merger of Widget and Layout
The idea of merging the Widget and Layout constructs presents itself as a tempting simplification, especially as their distinctions become blurred over time.

## Historical Separation:
Historically, the separation between Widget and Layout seemed justified. One primary reason was the type arguments associated with Msg in Widgets. However, this distinction lost its weight when I transitioned to a boxed trait approach, removing the type arguments.

Another reason that comes to mind is the desire for `widget.update()` to have immediate access to child widgets through a mutable reference, steering clear of any `Rc<RefCell>` complications.

## The Ideal Solution:
Ideally, a solution would allow for a tree structure similar to HTML or JSON, defining a hierarchy of widgets/layouts, but with the flexibility to directly access the members.

Consider the `TwoButtonEdit` example:

```mermaid
graph TD
A[Window(30,20)] --> B[Frame(Style::Double)]
B --> C[EditBox(some parameters)]
B --> D[Separator]
B --> E[Line(Dir::Horizontal)]
E --> F[Button(OK)]
E --> G[Separator]
E --> H[Button(Cancel)]
```

For this structure, one possible approach to access the EditBox and buttons as members would be through function-passing: a transformation `&mut Widget -> &mut Widget` that would accept Self as an argument.

Alternatively, the ownership could be reversed: Widgets could be owned by Layouts, and methods on Self, such as `Self.ButtonOK()`, would retrieve specific widgets (like the "OK button") from the corresponding layout. While this approach simplifies ownership concerns, it could lead to less intuitive code structures.

