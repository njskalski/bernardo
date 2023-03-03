# Who needs what

This file is a simplified "map" of items Gladius uses to do it's things, and their relations, so I can decide where
to put them in terms of ownership.

```mermaid
flowchart 

BufferRegister 
DocumentId
BufferState

DocumentIdIdentifies{DocumentId\n identifies BufferState,\n even if it's not saved}

DocumentId --- DocumentIdIdentifies
BufferState --- DocumentIdIdentifies

BufferRegisterMaps{BufferRegister maps\nDocumentIds\nwith\n BufferStates}
DocumentId --- BufferRegisterMaps
BufferState --- BufferRegisterMaps
BufferRegister --- BufferRegisterMaps

W7e --- ProducesNavComp
Config --- ProducesNavComp

NavComp

ProducesNavComp{Produces\nNavComp} --> NavComp
ProducesSPaths{Produces\nSPaths} --> SPath

SPath

```