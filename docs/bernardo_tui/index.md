# Bernardo TUI

Bernardo TUI is a text user interface library.

## Motivation and philosophy

Bernardo is centered around self-imposed constraints. When code base gets large enough, the primary issue becomes "how to not break anything" and "figure out where the code lives".

Bernardo paradigm aims to give a developer "reasonable assurances" and reduce the development drag of large codebases.

The price for that separation of concerns is in "elasticity" - there interfaces that would be much easier to implement in other paradigms. The argument I am trying to make is "suboptimal order is better than a complete chaos, especially in large projects" 

## General information

- [Focus, Widgets, Input and Messages](focus_and_input.md) is the essential first read.
- [Rendering basics](rendering.md)
- [Layout and Scrolls](layouts.md) building blocks of complex widgets
- [Complex widgets](complex_widgets.md)

## Work in progress

- [Attention tree](attention_tree.md) [WIP]

## Available widgets

Simple Widgets:

- [EditBox](../../src/widgets/edit_box.rs)
- [Button](../../src/widgets/button.rs)
- [TextWidget](../../src/widgets/text_widget.rs) ("Label" in other libraries)
- [ListWidget](../../src/widgets/list_widget/list_widget.rs)
- [TreeViewWidget](../../src/widgets/tree_view/tree_view.rs)
- [NestedMenu](../../src/widgets/nested_menu/widget.rs) (not used in Gladius, replaced with ContextMenu everywhere)

Complex/Combined Widgets:

- [GenericDialog](../../src/widgets/generic_dialog/generic_dialog.rs)
- [SaveFileDialog](../../src/widgets/save_file_dialog/save_file_dialog.rs) ("SaveAsDialog" in other libraries)
- [EditorWidget](../../src/widgets/editor_widget/editor_widget.rs) (draws text and some hovers)
- [EditorView](../../src/widgets/editor_view/editor_view.rs) (above, but wrapped with scroll, line numbers and with
  find/replace added on top)
- [BigList](../../src/widgets/big_list/big_list_widget.rs) which draws "multiple instances of the same widget so you can
  select from them"
- [ContextMenu](../../src/widgets/context_menu/widget.rs) offers a tree-like (nested) menu + fuzzy search.

Specialized variants:

- [ContextBar](../../src/widgets/editor_widget/context_bar/widget.rs) is a specialized variant of ContextMenu used
  within EditorWidget. It may get renamed.
- [FileTreeWidget](../../src/widgets/file_tree_view/file_tree_view.rs) is a specialized variant of TreeViewWidget, where the TreeNode is set to FileTreeNode.

### Inspiration for name

[Bernardo Gui](https://en.wikipedia.org/wiki/Bernard_Gui), an inquisitor, historian and for a period of time, Bishop of
Tui :D. Portrayed as violent, unfair man in
"The Name of the Rose" novel by Umberto Eco. Some sources claim that this image is overly negative. I consider using
mouse for programming a heresy, so I figured I need a help from professional inquisition that no one expects.

To those who say "I shouldn't name a project after clearly a negative character" - I do not support nor condemn a historical figure.
