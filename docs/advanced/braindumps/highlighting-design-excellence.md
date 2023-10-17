# Highlighting Design Excellence: Distinguishing `on_input()` from `update()`
In the design of our application, special attention has been given to the distinction between two core methods: `on_input()` and `update()`. The rationale for this distinction stems from the need to handle diverse sources of updates and inputs. Here, we delve into the reasons that shaped this design decision.

## Scenarios Prompting Widget Updates:
A widget's state can change based on multiple triggers. The primary scenarios are:

1. **User Input**:
    - Source: Direct interaction from the user.
    - Method: `on_input()`
    - Example: A user types into a text box or clicks a button.
2. **Out-of-User Input**:
    - Source: External changes that aren't directly user-driven.
    - Method: `update()`
    - Example: A change in the filesystem triggering an update in the SaveFileDialog.
3. **Child Widget Reactions**:
    - Source: Responses or reactions based on a child widget's actions.
    - Method: `update()`
    - Example: A dropdown menu in a widget might emit a message when an option is selected, prompting the parent widget to adjust its display.

```mermaid
graph TD
A[Widget State] --> B[User Input]
A --> C[Out-of-User Input]
A --> D[Child Widget Reactions]
B --> E[`on_input()` Method]
C --> F[`update()` Method]
D --> F
```

## Conclusion:
The separation between `on_input()` and `update()` is not just a matter of semantics. It represents a design choice aimed at efficiently handling diverse sources of updates. By categorizing the sources and directing them to specific methods, we ensure that the widget responds appropriately and efficiently to every change, enhancing the user experience and system robustness.