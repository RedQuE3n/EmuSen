namespace EmuSen.Pegasus

open System

/// Wire and file format versions. See EmuSen_Pegasus.md §6.1.
module Version =
    [<Literal>]
    let Protocol = 1uy

    [<Literal>]
    let FileSchema = 1uy

type PeerId =
    | PeerId of string

    static member New() = PeerId(Guid.NewGuid().ToString("N"))
    member this.Value = let (PeerId v) = this in v

type NoteId =
    | NoteId of string

    static member New() = NoteId(Guid.NewGuid().ToString("N"))
    member this.Value = let (NoteId v) = this in v

type PeerInfo =
    { Id: PeerId
      Name: string
      /// "#rrggbb", used to tint this peer's caret.
      Color: string }

/// Where a peer's caret and selection anchor sit, as offsets into the note.
type Presence =
    { Peer: PeerInfo
      Caret: int
      Anchor: int }

/// One message on the wire. Sync payloads are raw Yjs bytes, so a bridge to
/// y-websocket stays a shim rather than a rewrite -- see EmuSen_Pegasus.md §10.3.
type Frame =
    | Hello of PeerInfo
    | SyncStep1 of stateVector: byte[]
    | SyncStep2 of diff: byte[]
    | Update of update: byte[]
    | Awareness of Presence
    | Bye

exception ProtocolError of string
