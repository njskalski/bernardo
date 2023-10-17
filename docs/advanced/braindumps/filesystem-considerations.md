# Filesystem Considerations

The filesystem is the backbone of any application that deals with file management, and we need to tread carefully as we design and implement it for our software.

## Introduction
While the future may bring the possibility of a mock filesystem or even a remote filesystem, our current focus remains centered on traditional local filesystems.

## Core Responsibilities
Our filesystem will handle:

1. **OS Communication**: Relaying which files are present.
2. **File State Monitoring**: Keeping tabs on the status and changes to files.
3. **Language Server Protocol Interaction**: Communicating file changes.

## Design Architecture
The filesystem will be implemented as a **singleton**—initialized only once at the main level to ensure consistency and avoid redundancy.

While widgets like the FileTree and SaveFileAsDialog will interface with the filesystem, they'll do so by querying "views" into the filesystem. It's crucial to distinguish between the internal state of these widgets and the filesystem itself. For instance, the expanded state of tree nodes in the FileTree doesn't directly correlate to the filesystem's state.

## File Monitoring
File monitoring will be offloaded to a dedicated thread, leveraging the "notify" features of the kernel. In future iterations, this might also tap into remote services for more complex monitoring tasks.

## Updating Widgets
Widgets will receive filesystem updates through our standard update protocol, channeled via the "recursive_treat_views" function. Given that this tree's size will remain manageable, we'll avoid over-optimizing here, which could lead to unwanted complexities.

## Handling IO Operations
IO operations, due to their unpredictable nature, can sometimes be blocking. For instance, a request to list files in a directory might take significantly longer than expected.

To address this, we'll structure our IO operations to return a preliminary response within a short time frame (e.g., a 500ms timeout). This initial response will clarify if the data is complete or if there's more to come. If the latter, the IO operation will continue in the background, offloaded to a separate thread, with results delivered using our standard update mechanism.

