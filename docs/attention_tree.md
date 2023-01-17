### Status

This file is Work in Progress

### Discussion

Most significant part of programmer's work nowadays, is understanding how a bigger system functions **before** we add
our changes. This process of getting to know the larger code base generally consists of two major activities:

- code navigation
- debugging

In both of these processes we *dive* into a sub-programs, usually implemented in separate functions, sometimes in some
other files. More often than not, whatever we're looking for is **not** where we look for it, so most of these *dives*
are quite immediately followed with an *emergence*.

Let's think about two particular scenarios

#### Scenario 1

User reads code of function1 in file1. There is a call to a function2 inside the same file. User *dives* to read it, and
after a short while decides to *emerge*. We want a "reverse of dive" to happen. Calling on "show me references of
function2" will overflow User with irrelevant information. Scrolling to look for function1 is an annoyance. Finding it
with tools like "outline" is less of an annoyance, but it's still a poor substitute to *undo-dive* move.

#### Scenario 2

User reads code of function1 in file1. There is a call to a function2 in file2. User *dives* to read it, editor opens a
new tab corresponding to file2, that joins 10 other opened tabs. User decides to go back to function1, but he does not
remember in which file it was. What follows is a sad story of opening and scrolling all 11 files looking for function1.

#### Result

In both cases, a *undo dive* or *emerge* operation is what user desires. Now *emerge* is unequivocal only in context of
browsing history. Now let's hope introduction of it will not ruin the experience as it did for the web :D.

### Attention tree or queue

So we want to be able to navigate "reading history", both between the opened editor tabs, and within one. A *dive* is a
potentially destructive action, that can result in destruction of attention. An *emerge* action can be viewed twofold:
either as a retraction, or statement "I got what I need from that place, let's see what's next". In first case, removing
secondary location from attention queue and overriding it with result of next *dive* would be justified. But how do we
tell it apart from the second case, where what we have seen might be important to our later investigation?

A typical editor (say IntelliJ) has a mechanism of "bookmarks". A lazy solution would be to just expect the user to mark
"views" he might want to revisit, something like a union of "bookmark" and a "perspective" in Eclipse. When debugger is
implemented, I will want the view to change automatically, not expect the user to open and arrange the views manually.

Another question is: is there a benefit for the tree-like structure mimicking the navigation history? I say yes, because
most likely this will also correspond to call graph, or even more specific - order of execution, even if call hierarchy
does not reflect it in full (think async, messages etc).

So the question remains - how do I decide, which items I decide to keep in the history, and which to discard?
Furthermore, how to decide which items constitute a reasonable first-degree nodes.

Say for instance, that I dive down to a place where a message M is sent. I decide I want to figure out "who will read
it". Let's mark that place X. I will most likely "look for references of M", which will open a "find references" FR
screen. From this screen I will most likely choose one location I want to see, I might be lucky and find the right spot
immediately, I might be wrong. Let's say I did first to place in code Y1, and then right to place in code Y2.

The tree would look like this:

```
root
    X
        FR
            Y1
            Y2
```

I might choose to remove FR and Y1 from the tree.

I could go in another lazy direction and declare Gladius "a modal editor", saying that "oh, navigation is a mode, and
Undo in navigation makes perfect sense". Sure, but at what expense? Introduction of atypical interface to just avoid
making tough calls is lazy design.

Anyway, I go with this thing, and see where that takes me.

#### Lols and trivia

I once heard that "if you have to use debugger, the code is already in bad shape". I would answer today that "a debugger
is usually more valuable to the project than a programmer making such statements".  