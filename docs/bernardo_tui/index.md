# Bernardo/Gladius docs

Bernardo TUI is a text user interface library.

Gladius, is the project of code editor for which Bernardo was written.

They live in a single repository at this time, but will be split in the future.

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