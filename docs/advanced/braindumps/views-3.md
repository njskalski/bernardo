While working on a Completion widget, using ComplexWidget "trait", which is just "common boilerplate" for composite
widgets, I encountered a following issue:

If "focused" subwidget appears as a result of external state changing (here: completion future resolving),
we have no "on update" to cause widget to *drop* outdated focus_state pointing to background (selfwidget)
or update focus to freshly_appeared "fuzzy".

This brings me to conclusion, that it would be beneficial, at least in this case, to introduce a new invariant:

That a widget can be modified only via "update" method. But this would lead to all kind of troubles, namely:

Imagine I implemented "multiplayer" editing, via some online service (possible one day). There's a separate thread
polling for updates, maybe listening to a websocket. Updates come, I need to update the local display of the text.

Right now, I just wait for RwLock on buffer state object, update state of the buffer, which is separated from widget,
and tell widget to re-render. The problem is that I have no mechanism to tell widget "hey, you've been updated",
it can "check for being updated" on tick.

If I had to "route all updates via update function", that would require introducing a mechanism to refer to a widget
in some way from outside. But ownership rules mean, the *only* way to do that would be to pass message through
it's owners, owners' owners etc, generally in opposite direction to focus path, but also to subtrees outside it.

It would also introduce assumptions about existence of sub-widgets, that can become obsolete just as quickly as
retained focus_group.

Bottom line it's just moving problem from one place to the other. I am going to fix it other way.