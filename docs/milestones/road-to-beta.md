What I am working on right now:
- I am trying to get the "code results widget" to display anything

Here is what I want to be present in Gladius Beta:

1)
Code navigation:
    Go to definition
    Show usages
        On selected - open file

2)
Full text search

3)
USER FOCUS PATH

3)
Context menu

4)
Overall commands:
    - prune unchanged buffers
    - learning mode

5)
git filtering in:
    - file search
    - full text search

6)
[DONE] Editor "labels" (in-code overlay-"comments", clearly distingushible messages noting errors/warnings)

6.5)
Status bar (at minimum file, col, row)

7)
Code "collapsing"

8)
copy-cut-paste and context bar in tree!

Nice to haves (only if they turn out to be low hanging fruits):

9) errors/warnings in editor

9.5) a file descriptor I can stream to output from build commands to make the labels

10)
Tests:
save as:
    - error handling in save (failed write, disappearing dir etc)
    - changes the path to new in consecutive save as
Fuzzy file search
Resize event (start in crossterm_input.rs)

11)
validate invariants

12)
I'd love to see SubwidgetPointer refactored to SubwidgetPointerOp and handling of "non found subwidget"

Docs:
1) how views work
2) how main loop works
3) how filesystems are implemented




