# Save File Dialog: MVP Overview

The Save File Dialog is a crucial component, allowing users to choose a location to save their files.
This document details the Minimum Viable Product (MVP) for the Save File Dialog.

## Introduction

The primary goal of the Save File Dialog MVP is to provide users with a simple, intuitive way to save their files. Intuitive interfaces are a core design principle of Bernardo, and the Save File Dialog is no exception. Intuitive interface to specify the location and name of the file they wish to save. At this stage we aim to cover the basic functionality, ensuring compatibility with the rest of the application.

## Features

The Save File Dialog MVP will include the following features:
1. Directory Tree View
    - Seamless Navigation through directories.
    - Display the current directory path at the top of the tree.
    - Highlight the active directory.
    - Set the tree's root as the current directory.
2. File Naming Interface
    - A user-friendly text box for specifying the desired file name.
    - Default to the name of the currently open file, if applicable.
3. File Type/Extension Selection
    - A dropdown menu should allow users to select the file type/extension.
    - The chosen file type should append the appropriate extension to the file name.
4. Actions Controls
    - Clicking "Save" should save the file to the specified location with the chosen name and extension.
    - Clicking "Cancel" should close the dialog without saving the file.
5. Overwrite Alerts
    - Prompt users with a warning if they attempt to save a file with a name that already exists in the chosen location.

## Design Principles


## Design Considerations
- The dialog should be modal, ensuring users adress it before proceeding with other tasks.
- It should be resizable and should remember its last size and position for user convenience.
- Keyboard shortcuts for common actions (e.g., CTRL + S for save, ESC for cancel) should be implemented.

## Future Enhancements
While the MVP focuses on core functionality, future versions of the Save File Dialog could include:
- "Favorites" or "Recent Locations" section for quicker access to commonly used directories.
- "New Folder" button to allow users to create new folders.
- File previews for common file types.

## Feedback

If you have suggestions or encounter issues with the Save Dialog MVP, please raise an issue on GitLab or contact the project maintainers.


As a next step, I proceed in implementing SaveFileDialog. It's supposed to resemble a traditional design
as in XFCE prior GTK3 update.

Layout:

+-----------+--------------------------------+
| TreeView  |      ListView of files         |
| of        |                                |
|directories|                                |
|           |                                |
|           |                                |
|           |                                |
|           +--------------------------------+
|           | EditBox with file type         |
|           +-----------+--------+-----------+
|           | CancelBtn |        |  SaveBtn  |
+-----------+-----------+--------+-----------+