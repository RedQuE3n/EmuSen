namespace EmuSen.Pegasus

open System
open System.IO
open System.Text

/// Frame <-> bytes. Format and tag assignments are in EmuSen_Pegasus.md §10.3.
module Codec =

    [<Literal>]
    let private TagHello = 0uy

    [<Literal>]
    let private TagSyncStep1 = 1uy

    [<Literal>]
    let private TagSyncStep2 = 2uy

    [<Literal>]
    let private TagUpdate = 3uy

    [<Literal>]
    let private TagAwareness = 4uy

    [<Literal>]
    let private TagBye = 5uy

    /// Frames are bounded so a malformed or hostile length cannot make us
    /// allocate arbitrarily; see EmuSen_Pegasus.md §10.5.
    [<Literal>]
    let MaxFrameBytes = 64 * 1024 * 1024

    // Binary rather than JSON: System.Text.Json cannot serialise F# unions and
    // PeerId is one. See EmuSen_Pegasus.md §4.6.
    let private writeWith (tag: byte) (write: BinaryWriter -> unit) =
        use ms = new MemoryStream()
        use w = new BinaryWriter(ms, UTF8Encoding false, true)
        w.Write tag
        write w
        w.Flush()
        ms.ToArray()

    let private writePeer (w: BinaryWriter) (p: PeerInfo) =
        w.Write p.Id.Value
        w.Write p.Name
        w.Write p.Color

    let private readPeer (r: BinaryReader) : PeerInfo =
        let id = r.ReadString()
        let name = r.ReadString()
        let color = r.ReadString()

        { Id = PeerId id
          Name = name
          Color = color }

    /// Serialise a frame to its plaintext body. Encryption and length-prefixing
    /// happen above this, in Pegasus.Net.
    let encode (frame: Frame) : byte[] =
        match frame with
        | Hello peer -> writeWith TagHello (fun w -> writePeer w peer)
        | SyncStep1 sv -> writeWith TagSyncStep1 (fun w -> w.Write sv)
        | SyncStep2 diff -> writeWith TagSyncStep2 (fun w -> w.Write diff)
        | Update u -> writeWith TagUpdate (fun w -> w.Write u)
        | Awareness p ->
            writeWith TagAwareness (fun w ->
                writePeer w p.Peer
                w.Write p.Caret
                w.Write p.Anchor)
        | Bye -> [| TagBye |]

    let decode (body: byte[]) : Frame =
        if body.Length = 0 then
            raise (ProtocolError "empty frame")

        let tag = body[0]
        let payload = body[1..]

        try
            match tag with
            | TagSyncStep1 -> SyncStep1 payload
            | TagSyncStep2 -> SyncStep2 payload
            | TagUpdate -> Update payload
            | TagBye -> Bye
            | TagHello ->
                use r = new BinaryReader(new MemoryStream(payload), UTF8Encoding false)
                Hello(readPeer r)
            | TagAwareness ->
                use r = new BinaryReader(new MemoryStream(payload), UTF8Encoding false)
                let peer = readPeer r

                Awareness
                    { Peer = peer
                      Caret = r.ReadInt32()
                      Anchor = r.ReadInt32() }
            | unknown -> raise (ProtocolError $"unknown frame tag {unknown}")
        with
        | :? EndOfStreamException -> raise (ProtocolError $"truncated payload for frame tag {tag}")
        | :? ArgumentException -> raise (ProtocolError $"malformed payload for frame tag {tag}")

/// CRC-32 (IEEE), used by the store to detect a torn trailing record.
module Crc32 =

    let private table =
        lazy
            (Array.init 256 (fun i ->
                let mutable c = uint32 i

                for _ in 0..7 do
                    c <- if c &&& 1u <> 0u then 0xEDB88320u ^^^ (c >>> 1) else c >>> 1

                c))

    let compute (data: byte[]) (offset: int) (count: int) =
        let t = table.Value
        let mutable crc = 0xFFFFFFFFu

        for i in offset .. offset + count - 1 do
            crc <- t[int ((crc ^^^ uint32 data[i]) &&& 0xFFu)] ^^^ (crc >>> 8)

        crc ^^^ 0xFFFFFFFFu

    let ofBytes (data: byte[]) = compute data 0 data.Length
