namespace EmuSen.Pegasus

open System
open System.Buffers.Binary
open System.IO

/// The append-only note file. Layout and the recovery argument are in
/// EmuSen_Pegasus.md §6.
module Store =

    let magic = "PGSS"B

    [<Literal>]
    let HeaderBytes = 32

    [<Literal>]
    let RecordHeaderBytes = 8

    let writeHeader (stream: Stream) (noteId: NoteId) (createdAt: DateTimeOffset) =
        let header = Array.zeroCreate<byte> HeaderBytes
        Array.blit magic 0 header 0 4
        header[4] <- Version.FileSchema
        Array.blit ((Guid.Parse noteId.Value).ToByteArray()) 0 header 8 16
        BinaryPrimitives.WriteInt64LittleEndian(Span(header, 24, 8), createdAt.ToUnixTimeMilliseconds())
        stream.Write(header, 0, HeaderBytes)

    let readHeader (stream: Stream) =
        let header = Array.zeroCreate<byte> HeaderBytes

        if stream.Read(header, 0, HeaderBytes) <> HeaderBytes then
            raise (ProtocolError "note file is shorter than its header")

        if header[0..3] <> magic then
            raise (ProtocolError "not a Pegasus note file: bad magic")

        if header[4] <> Version.FileSchema then
            raise (ProtocolError $"note file schema v{header[4]}, this build understands v{Version.FileSchema}")

        NoteId(Guid(header[8..23]).ToString "N")

    /// Reads records until one is torn, which is the normal end state after a
    /// crash. Returns the intact updates and the offset where good data stops.
    let readRecords (stream: Stream) =
        let updates = ResizeArray<byte[]>()
        let mutable atEnd = false
        let mutable goodUpTo = stream.Position

        while not atEnd do
            let head = Array.zeroCreate<byte> RecordHeaderBytes

            if stream.Read(head, 0, RecordHeaderBytes) <> RecordHeaderBytes then
                atEnd <- true
            else
                let length = BinaryPrimitives.ReadUInt32LittleEndian(ReadOnlySpan(head, 0, 4))
                let expected = BinaryPrimitives.ReadUInt32LittleEndian(ReadOnlySpan(head, 4, 4))

                if int64 length > stream.Length - stream.Position then
                    atEnd <- true
                else
                    let payload = Array.zeroCreate<byte> (int length)

                    if stream.Read(payload, 0, int length) <> int length then
                        atEnd <- true
                    elif Crc32.ofBytes payload <> expected then
                        atEnd <- true
                    else
                        updates.Add payload
                        goodUpTo <- stream.Position

        updates.ToArray(), goodUpTo

    let recordBytes (payload: byte[]) =
        let record = Array.zeroCreate<byte> (RecordHeaderBytes + payload.Length)
        BinaryPrimitives.WriteUInt32LittleEndian(Span(record, 0, 4), uint32 payload.Length)
        BinaryPrimitives.WriteUInt32LittleEndian(Span(record, 4, 4), Crc32.ofBytes payload)
        Array.blit payload 0 record RecordHeaderBytes payload.Length
        record

    let private openAppend (path: string) =
        let s = new FileStream(path, FileMode.Open, FileAccess.ReadWrite, FileShare.Read)
        s.Seek(0L, SeekOrigin.End) |> ignore
        s

    /// One note on disk, held open for appends.
    type NoteFile(path: string, ?id: NoteId) =
        let noteId, recovered, torn =
            if File.Exists path then
                use s = new FileStream(path, FileMode.Open, FileAccess.ReadWrite, FileShare.Read)
                let noteId = readHeader s
                let updates, goodUpTo = readRecords s
                let torn = goodUpTo < s.Length
                // Drop the torn tail so the next append is not stranded behind it.
                if torn then s.SetLength goodUpTo
                noteId, updates, torn
            else
                let noteId = defaultArg id (NoteId.New())
                let dir = Path.GetDirectoryName(Path.GetFullPath path)
                if not (String.IsNullOrEmpty dir) then Directory.CreateDirectory dir |> ignore
                use s = new FileStream(path, FileMode.CreateNew, FileAccess.Write, FileShare.Read)
                writeHeader s noteId DateTimeOffset.UtcNow
                noteId, [||], false

        let mutable stream = openAppend path
        let mutable records = recovered.Length

        member _.Path = path
        member _.NoteId = noteId

        /// Updates recovered at open, in order, ready to replay into a replica.
        member _.Recovered = recovered

        /// True when a torn trailing record was found and dropped -- evidence of
        /// an earlier crash, not an error.
        member _.TornRecordDropped = torn

        member _.RecordCount = records

        member _.Append(update: byte[]) =
            if update.Length > 0 then
                let record = recordBytes update
                stream.Write(record, 0, record.Length)
                // Reaches the OS here, so a process crash loses nothing; only a
                // power cut can lose the tail. See EmuSen_Pegasus.md §6.4.
                stream.Flush()
                records <- records + 1

        /// Forces the tail to physical media. Called on close and on idle, not
        /// per keystroke.
        member _.Sync() = stream.Flush true

        /// Collapses the log to a single snapshot record through a temp file and
        /// an atomic rename. See EmuSen_Pegasus.md §6.3.
        member _.Compact(snapshot: byte[]) =
            let temp = path + ".compacting"

            do
                use out = new FileStream(temp, FileMode.Create, FileAccess.Write, FileShare.None)
                writeHeader out noteId DateTimeOffset.UtcNow
                let record = recordBytes snapshot
                out.Write(record, 0, record.Length)
                out.Flush true

            stream.Flush true
            stream.Dispose()
            File.Move(temp, path, true)
            stream <- openAppend path
            records <- 1

        /// The readable projection beside the note. Regenerated, never read back
        /// as truth -- EmuSen_Pegasus.md §6.5.
        member _.WriteProjection(text: string) =
            let target = Path.ChangeExtension(path, ".md")
            let temp = target + ".tmp"
            File.WriteAllText(temp, text)
            File.Move(temp, target, true)

        interface IDisposable with
            member _.Dispose() =
                stream.Flush true
                stream.Dispose()
