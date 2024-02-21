# SPath

SPath is a type that was designed to mimic a filesystem.

## Why not just PathBuf?

Imagine I want to keep a number file names in memory, for instance as data to quickly redraw TreeWidget showing a
filesystem (perhaps remote, perhaps mock one).

Storing PathBuf / URIs would force me to keep large amounts of identical prefixes to files sharing common subdirectory.

## Why not just query filesystem live?

Filesystem operations are I/O. I want to cache results of such operations between frames (I am not going to list
directory every redraw).

So I needed a caching mechanism anyway, SPath is just a caching mechanism that's "one per filesystem".

## Other advantages

1) One place to keep all metadata for files (like handles that APIs like "inotify" return)
2) It's easy to mock a filesystem for tests
3) It's possible to implement a network file system, even when no FUSE is available for it (hypothetical)

## What's missing

Here are some use cases that are not implemented by current design (require updates to design):

1) It's currently not possible to open a file that's not within FSF associated with [Workspace](/workspace.md). This is
   a shortcoming, it would be great to be able to browse (and navigate) source straight from web (crates.io) or cached
   outside working directory. 