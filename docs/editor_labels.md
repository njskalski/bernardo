# Editor Labels design doc

Editor Labels (EL) are pieces of information displayed alongside code, 
that are not part of the code. They can be auto-annotations, compilation errors
or warnings or whatever other information I decide to add.

## Goals
It is important to strike a balance between "useful labels" and "sensory
overload". An instant toggle between annotations is a bare minimum.

## Types

I think of following parameters to a label:

Trigger:
- hover: appearing only while hovering over a trigger symbol/line
- constant: visible whenever labels are enabled

Style:
- over/above symbol (working with hover), covering code
- inline - appearing immediately after the symbol, pushing further columns to 
the right. Cursor should treat entire label as one symbol (for selection) but
not include them in the "copy/paste" results.
- top-bottom inline - appearing as row between code
- margin - appearing either in column left from code
- commentary - appearing immediately after end-of-line symbol 

for BETA I require at least:
- inline
- top-bottom inline

## Providers

Provider is a source of Labels. There will be at least two of those
in most basic setup:
- LSP will provide type inference
- Compiler/Linter output will provide errors and warnings

That means:
- there will be multiple providers
- merging their outputs is 101
- I need a method to either:
  - merge losslesly
  - settle conflicts

At this point I will go with "merge losslessly". I set a hierarchy of
priorities to providers.