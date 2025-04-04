# Config

Absent any pre-existing config, Gladius will create **~/.config/gladius/config.ron**
which is just a Default instance of  [config.rs](/src/config/config.rs), which you can use as reference.

If you ever get an error "failed to load due to config issue", just rename that file and you'll get a fresh one (can
happen if we introduce new/rename old options).

# Default keymap

## Editor

| description                      | key combination       | comment                                                                                                                     |
|----------------------------------|-----------------------|-----------------------------------------------------------------------------------------------------------------------------|
| cursor movement                  | arrows                |                                                                                                                             |
| jump to next word/previous word  | ctrl + arrows         |                                                                                                                             |
| highlight                        | shift + arrows        | contrary to what Vim apologists claim, it's actually possible to "stop highlighting" when shift is released                 |
| highlight entire word            | ctrl + shift + arrows |                                                                                                                             |
| select all                       | ctrl + a              |                                                                                                                             |
| copy                             | ctrl + c              |                                                                                                                             |
| paste                            | ctrl + v              |                                                                                                                             |
| save                             | ctrl + s              |                                                                                                                             |
| save as                          | ctrl + d              | for some reason shell does not disinguish between shift+ctrl+s and shift+s, but it does with                 arrows. Weird. |
| find                             | ctrl + f              |                                                                                                                             |
| replace                          | ctrl + r              |                                                                                                                             |
| reformat                         | ctrl + g              |                                                                                                                             |
| **enter "cursor dropping mode"** | ctrl + w              | When in Cursor Dropping Mode, press enter to add/remove cursors. Press ESC to go back to edit with multiple cursors :)      |
| undo                             | ctrl + z              |                                                                                                                             |
| redo                             | **ctrl + x**          |                                                                                                                              I couldn't get shift-ctrl-z working, so I had to go with something close.                                                                                                         |
| ask for completions              | ctrl + space          | like in Eclipse                                                                                                             |

## Tree view

| description                             | key combination | comment             |
|-----------------------------------------|-----------------|---------------------|
| hide/unhide hidden (dot-prefixed) files | ctr+h           | not implemented yet |

## General keys

| description                      | key combination | comment                                      |
|----------------------------------|-----------------|----------------------------------------------|
| close Gladius                    | ctrl + q        |                                              |
| open context menu/everything bar | ctrl + e        |                                              |
| move focus between panes         | alt + arrows    |                                              |
| fuzzy files     | ctrl+j| fuzzy finding of files                       |
| find in files | ctrl + g| full text search over all files              |
| exit (window, dialog, menu etc.) | esc             | generally think of "esc" as "go away" button |
| make screenshot | ctrl + u| screenshots can be later checked with `cargo run --bin reader` |