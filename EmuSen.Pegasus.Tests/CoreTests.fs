module EmuSen.Pegasus.Tests.CoreTests

open System
open System.IO
open Xunit
open FsCheck
open FsCheck.Xunit
open EmuSen.Pegasus

// ---------------------------------------------------------------------------
// Codec
// ---------------------------------------------------------------------------

let private peer =
    { Id = PeerId "abc123"
      Name = "tristan"
      Color = "#8844ff" }

[<Fact>]
let ``every frame case survives a codec round trip`` () =
    let cases =
        [ Hello peer
          SyncStep1 [| 1uy; 2uy; 3uy |]
          SyncStep2 [| 9uy; 8uy |]
          Update [| 0uy; 255uy; 7uy |]
          Awareness { Peer = peer; Caret = 12; Anchor = 4 }
          Bye ]

    for case in cases do
        Assert.Equal(case, Codec.decode (Codec.encode case))

[<Fact>]
let ``an empty frame is rejected rather than misread`` () =
    Assert.Throws<ProtocolError>(fun () -> Codec.decode [||] |> ignore)

[<Fact>]
let ``an unknown tag is rejected rather than ignored`` () =
    Assert.Throws<ProtocolError>(fun () -> Codec.decode [| 99uy; 1uy |] |> ignore)

[<Property>]
let ``sync payloads round trip for any byte array`` (payload: byte[]) =
    let payload = if isNull payload then [||] else payload
    Codec.decode (Codec.encode (Update payload)) = Update payload

// ---------------------------------------------------------------------------
// Crc32
// ---------------------------------------------------------------------------

[<Fact>]
let ``crc32 matches the known IEEE check value`` () =
    Assert.Equal(0xCBF43926u, Crc32.ofBytes "123456789"B)

[<Fact>]
let ``crc32 detects a single flipped bit`` () =
    let data = Array.init 64 byte
    let mutated = Array.copy data
    mutated[30] <- mutated[30] ^^^ 1uy
    Assert.NotEqual(Crc32.ofBytes data, Crc32.ofBytes mutated)

// ---------------------------------------------------------------------------
// Crypto
// ---------------------------------------------------------------------------

[<Fact>]
let ``a sealed frame opens with the same join code`` () =
    let key = Crypto.deriveKey "7-lantern-quartz"
    let plain = "the quick brown fox"B
    Assert.Equal<byte[]>(plain, Crypto.openSealed key (Crypto.seal key plain))

[<Fact>]
let ``a sealed frame does not open with a different join code`` () =
    let sealed' = Crypto.seal (Crypto.deriveKey "7-lantern-quartz") "secret"B
    let wrong = Crypto.deriveKey "7-lantern-cobalt"
    Assert.Throws<ProtocolError>(fun () -> Crypto.openSealed wrong sealed' |> ignore)

[<Fact>]
let ``tampering with the ciphertext is detected`` () =
    let key = Crypto.deriveKey "3-ember-tulip"
    let sealed' = Crypto.seal key "hello world"B
    sealed'[Crypto.NonceBytes + 2] <- sealed'[Crypto.NonceBytes + 2] ^^^ 0xFFuy
    Assert.Throws<ProtocolError>(fun () -> Crypto.openSealed key sealed' |> ignore)

[<Fact>]
let ``the join code is case and whitespace insensitive`` () =
    Assert.Equal<byte[]>(Crypto.deriveKey "7-Lantern-Quartz", Crypto.deriveKey "  7-lantern-quartz ")

[<Fact>]
let ``sealing twice yields different bytes`` () =
    let key = Crypto.deriveKey "5-banjo-orbit"
    Assert.NotEqual<byte[]>(Crypto.seal key "same"B, Crypto.seal key "same"B)

[<Fact>]
let ``a challenge is answered only by the matching key`` () =
    let key = Crypto.deriveKey "2-cactus-velvet"
    let challenge = Crypto.newChallenge ()
    Assert.True(Crypto.verifyChallenge key challenge (Crypto.respondToChallenge key challenge))
    let impostor = Crypto.deriveKey "2-cactus-walnut"
    Assert.False(Crypto.verifyChallenge key challenge (Crypto.respondToChallenge impostor challenge))

// ---------------------------------------------------------------------------
// DocumentActor
// ---------------------------------------------------------------------------

/// Ships everything each side is missing, in both directions.
let private sync (a: DocumentActor) (b: DocumentActor) =
    let forB = a.DiffSince b.StateVector
    let forA = b.DiffSince a.StateVector
    b.ApplyRemote forB
    a.ApplyRemote forA

[<Fact>]
let ``concurrent edits converge and neither is lost`` () =
    use alice = new DocumentActor()
    use bob = new DocumentActor()
    alice.Insert(0, "shared base. ")
    sync alice bob

    alice.Insert(alice.Length, "ALICE ")
    bob.Insert(bob.Length, "BOB ")
    sync alice bob

    Assert.Equal(alice.Text, bob.Text)
    Assert.Contains("ALICE", alice.Text)
    Assert.Contains("BOB", alice.Text)
    Assert.Contains("shared base.", alice.Text)

[<Fact>]
let ``a remote update is not echoed back to its sender`` () =
    use alice = new DocumentActor()
    use bob = new DocumentActor()
    alice.Insert(0, "hello")

    let bobEmitted = ResizeArray<byte[]>()
    use _sub = bob.LocalUpdate.Subscribe bobEmitted.Add
    bob.ApplyRemote(alice.DiffSince bob.StateVector)

    Assert.Equal("hello", bob.Text)
    Assert.Empty(bobEmitted)

[<Fact>]
let ``a local edit is emitted for sending onward`` () =
    use doc = new DocumentActor()
    let emitted = ResizeArray<byte[]>()
    use _sub = doc.LocalUpdate.Subscribe emitted.Add
    doc.Insert(0, "typed")
    Assert.Single emitted |> ignore

[<Fact>]
let ``Changed fires for both local and remote edits`` () =
    use alice = new DocumentActor()
    use bob = new DocumentActor()
    let mutable count = 0
    use _sub = bob.Changed.Subscribe(fun () -> count <- count + 1)
    bob.Insert(0, "local")
    alice.Insert(0, "remote")
    bob.ApplyRemote(alice.DiffSince bob.StateVector)
    Assert.Equal(2, count)

[<Fact>]
let ``a replica seeded from a snapshot matches its source`` () =
    use source = new DocumentActor()
    source.Insert(0, "seeded content")
    use restored = new DocumentActor(source.Snapshot)
    Assert.Equal(source.Text, restored.Text)

[<Fact>]
let ``ReplaceAll turns a whole buffer into a minimal edit`` () =
    use doc = new DocumentActor()
    doc.Insert(0, "the quick brown fox")
    doc.ReplaceAll "the quick red fox"
    Assert.Equal("the quick red fox", doc.Text)
    doc.ReplaceAll ""
    Assert.Equal("", doc.Text)
    doc.ReplaceAll "fresh"
    Assert.Equal("fresh", doc.Text)

[<Fact>]
let ``ReplaceAll edits only the changed span, so a concurrent edit survives`` () =
    use alice = new DocumentActor()
    use bob = new DocumentActor()
    alice.Insert(0, "aaaa BBBB cccc")
    sync alice bob

    // Alice retypes the middle word while Bob appends, neither having synced.
    alice.ReplaceAll "aaaa ZZZZ cccc"
    bob.Insert(bob.Length, " dddd")
    sync alice bob

    Assert.Equal(alice.Text, bob.Text)
    Assert.Contains("ZZZZ", alice.Text)
    Assert.Contains("dddd", alice.Text)

[<Fact>]
let ``a tracked caret follows edits before it and ignores edits after it`` () =
    use doc = new DocumentActor()
    doc.Insert(0, "abcdef")
    let caret = doc.TrackCaret 3
    doc.Insert(0, "XXXXX")
    Assert.Equal(8, doc.ReadCaret caret)
    doc.Insert(doc.Length, "ZZZ")
    Assert.Equal(8, doc.ReadCaret caret)

[<Property(MaxTest = 60)>]
let ``two replicas converge under any interleaving of edits`` (edits: (bool * NonNegativeInt * NonEmptyString) list) =
    use alice = new DocumentActor()
    use bob = new DocumentActor()
    alice.Insert(0, "base")
    sync alice bob

    for toAlice, NonNegativeInt at, NonEmptyString text in List.truncate 12 edits do
        let target = if toAlice then alice else bob
        target.Insert(min at target.Length, text)

    sync alice bob
    alice.Text = bob.Text

// ---------------------------------------------------------------------------
// Store
// ---------------------------------------------------------------------------

let private tempDir () =
    let dir = Path.Combine(Path.GetTempPath(), "pegasus-tests", Guid.NewGuid().ToString "N")
    Directory.CreateDirectory dir |> ignore
    dir

[<Fact>]
let ``appended updates are recovered in order`` () =
    let path = Path.Combine(tempDir (), "note.pegasus")
    let noteId =
        use file = new Store.NoteFile(path)
        file.Append [| 1uy; 2uy |]
        file.Append [| 3uy |]
        file.NoteId

    use reopened = new Store.NoteFile(path)
    Assert.Equal(noteId, reopened.NoteId)
    Assert.False reopened.TornRecordDropped
    Assert.Equal<byte[][]>([| [| 1uy; 2uy |]; [| 3uy |] |], reopened.Recovered)

[<Fact>]
let ``a document survives a close and reopen through its log`` () =
    let path = Path.Combine(tempDir (), "note.pegasus")

    do
        use doc = new DocumentActor()
        use file = new Store.NoteFile(path)
        use _sub = doc.LocalUpdate.Subscribe file.Append
        doc.Insert(0, "durable ")
        doc.Insert(doc.Length, "content")

    use file = new Store.NoteFile(path)
    use restored = new DocumentActor()
    for update in file.Recovered do restored.ApplyRemote update
    Assert.Equal("durable content", restored.Text)

[<Fact>]
let ``a torn trailing record is dropped and everything before it survives`` () =
    let path = Path.Combine(tempDir (), "note.pegasus")

    do
        use file = new Store.NoteFile(path)
        file.Append [| 1uy; 2uy; 3uy |]
        file.Append [| 4uy; 5uy; 6uy; 7uy |]

    // Simulate a crash mid-append: lop three bytes off the last record.
    do
        use fs = new FileStream(path, FileMode.Open, FileAccess.Write)
        fs.SetLength(fs.Length - 3L)

    use reopened = new Store.NoteFile(path)
    Assert.True reopened.TornRecordDropped
    Assert.Equal<byte[][]>([| [| 1uy; 2uy; 3uy |] |], reopened.Recovered)

[<Fact>]
let ``a corrupted record body is caught by its crc`` () =
    let path = Path.Combine(tempDir (), "note.pegasus")

    do
        use file = new Store.NoteFile(path)
        file.Append [| 10uy; 20uy; 30uy; 40uy |]

    do
        use fs = new FileStream(path, FileMode.Open, FileAccess.Write)
        fs.Seek(int64 Store.HeaderBytes + int64 Store.RecordHeaderBytes + 1L, SeekOrigin.Begin) |> ignore
        fs.WriteByte 0xFFuy

    use reopened = new Store.NoteFile(path)
    Assert.True reopened.TornRecordDropped
    Assert.Empty reopened.Recovered

[<Fact>]
let ``appending after a torn tail keeps the file readable`` () =
    let path = Path.Combine(tempDir (), "note.pegasus")

    do
        use file = new Store.NoteFile(path)
        file.Append [| 1uy |]
        file.Append [| 2uy; 2uy |]

    do
        use fs = new FileStream(path, FileMode.Open, FileAccess.Write)
        fs.SetLength(fs.Length - 1L)

    do
        use recovered = new Store.NoteFile(path)
        recovered.Append [| 3uy; 3uy; 3uy |]

    use final = new Store.NoteFile(path)
    Assert.Equal<byte[][]>([| [| 1uy |]; [| 3uy; 3uy; 3uy |] |], final.Recovered)

[<Fact>]
let ``compaction collapses the log while preserving the document`` () =
    let path = Path.Combine(tempDir (), "note.pegasus")

    use doc = new DocumentActor()

    do
        use file = new Store.NoteFile(path)
        use _sub = doc.LocalUpdate.Subscribe file.Append
        for word in [ "one "; "two "; "three "; "four " ] do
            doc.Insert(doc.Length, word)
        Assert.Equal(4, file.RecordCount)
        file.Compact doc.Snapshot
        Assert.Equal(1, file.RecordCount)

    use reopened = new Store.NoteFile(path)
    Assert.Single reopened.Recovered |> ignore
    use restored = new DocumentActor()
    for update in reopened.Recovered do restored.ApplyRemote update
    Assert.Equal(doc.Text, restored.Text)

[<Fact>]
let ``a file that is not a note is rejected by its magic`` () =
    let path = Path.Combine(tempDir (), "impostor.pegasus")
    File.WriteAllBytes(path, Array.create 64 0uy)
    Assert.Throws<ProtocolError>(fun () -> new Store.NoteFile(path) |> ignore)

[<Fact>]
let ``the markdown projection is written beside the note`` () =
    let dir = tempDir ()
    let path = Path.Combine(dir, "ideas.pegasus")
    use file = new Store.NoteFile(path)
    file.WriteProjection "# Ideas\n\nfirst thought\n"
    Assert.Equal("# Ideas\n\nfirst thought\n", File.ReadAllText(Path.Combine(dir, "ideas.md")))

// ---------------------------------------------------------------------------
// Client ids. YDotNet's default Doc() draws from roughly 6 bits; two replicas
// sharing one lose each other's edits silently. EmuSen_Pegasus.md §4.5.
// ---------------------------------------------------------------------------

[<Fact>]
let ``replicas get distinct client ids`` () =
    let ids = [ for _ in 1..500 -> let d = new DocumentActor() in let i = d.ClientId in (d :> IDisposable).Dispose(); i ]
    Assert.Equal(500, ids |> List.distinct |> List.length)

[<Fact>]
let ``a client id is never zero and stays under the 2 to the 32 ceiling`` () =
    for _ in 1..200 do
        use d = new DocumentActor()
        Assert.InRange(d.ClientId, 1UL, ClientId.ExclusiveMax - 1UL)

[<Fact>]
let ``delta sync stays exact below the ceiling and breaks at it`` () =
    // Pins the YDotNet 0.6.0 boundary from EmuSen_Pegasus.md §4.7. If the
    // second half starts passing, the library was fixed and the cap can lift.
    let roundTrip (idA: uint64) (idB: uint64) =
        use a = new DocumentActor(clientId = idA)
        use b = new DocumentActor(clientId = idB)
        a.Insert(0, "BASE")
        b.ApplyRemote(a.DiffSince null)
        a.Insert(a.Length, "-A")
        b.Insert(b.Length, "-B")
        sync a b
        a.Text = b.Text

    Assert.True(roundTrip (ClientId.ExclusiveMax - 1UL) (ClientId.ExclusiveMax - 2UL))
    Assert.False(roundTrip ClientId.ExclusiveMax (ClientId.ExclusiveMax + 1UL))

[<Fact>]
let ``sharing a client id loses data, which is why we never share one`` () =
    // Pins the pathology the explicit id exists to prevent. If this ever starts
    // converging, YDotNet changed and the note in the design doc needs revisiting.
    use alice = new DocumentActor(clientId = 4242UL)
    use bob = new DocumentActor(clientId = 4242UL)
    alice.Insert(0, "AAAA")
    bob.Insert(0, "BBBB")
    sync alice bob
    Assert.NotEqual<string>(alice.Text, bob.Text)

// ---------------------------------------------------------------------------
// Caret adjustment
// ---------------------------------------------------------------------------

[<Fact>]
let ``an insert before the caret pushes it along`` () =
    Assert.Equal(8, Caret.adjust "abcdef" "XXXXXabcdef" 3)

[<Fact>]
let ``an insert after the caret leaves it alone`` () =
    Assert.Equal(3, Caret.adjust "abcdef" "abcdefZZZ" 3)

[<Fact>]
let ``an insert exactly at the caret leaves it alone`` () =
    Assert.Equal(3, Caret.adjust "abcdef" "abcZZZdef" 3)

[<Fact>]
let ``a delete before the caret pulls it back`` () =
    Assert.Equal(3, Caret.adjust "XXXXXabcdef" "abcdef" 8)

[<Fact>]
let ``a delete after the caret leaves it alone`` () =
    Assert.Equal(3, Caret.adjust "abcdefZZZ" "abcdef" 3)

[<Fact>]
let ``a caret inside a deleted span lands at the edit site`` () =
    Assert.Equal(4, Caret.adjust "abcdWXYZefgh" "abcdefgh" 8)

[<Fact>]
let ``a caret is clamped into the new buffer`` () =
    Assert.Equal(0, Caret.adjust "abcdef" "" 4)
    Assert.InRange(Caret.adjust "abcdef" "ab" 6, 0, 2)

[<Fact>]
let ``an unchanged buffer never moves the caret`` () =
    Assert.Equal(4, Caret.adjust "abcdef" "abcdef" 4)

[<Property(MaxTest = 300)>]
let ``an adjusted caret always lands inside the new buffer`` (before: NonNull<string>) (after: NonNull<string>) (NonNegativeInt caret) =
    let result = Caret.adjust before.Get after.Get caret
    result >= 0 && result <= after.Get.Length

// ---------------------------------------------------------------------------
// Agnosticism. Pegasus is a notepad on a windowing toolkit, not a part of the
// emulator, and LunaP is intended to be published on its own. Both claims are
// only true while this holds. See EmuSen_Pegasus.md §11.
// ---------------------------------------------------------------------------

[<Fact>]
let ``Pegasus references the toolkit and nothing else of EmuSen`` () =
    let allowed = set [ "EmuSen.LunaP" ]

    let referenced =
        typeof<DocumentActor>.Assembly.GetReferencedAssemblies()
        |> Array.map _.Name
        |> Array.filter (fun n -> n.StartsWith("EmuSen.", StringComparison.Ordinal))
        |> Array.filter (fun n -> not (allowed.Contains n))
        |> Array.distinct

    Assert.True(
        referenced.Length = 0,
        $"""Pegasus reaches past the toolkit into: {String.Join(", ", referenced)}"""
    )

[<Fact>]
let ``the workspace path is not hardcoded to one platform`` () =
    // ".local/share" is Linux-only and was wrong on the macOS and Windows RIDs
    // this project publishes for.
    let root = Controller.defaultWorkspaceRoot
    Assert.False(String.IsNullOrWhiteSpace root)
    Assert.True(Path.IsPathRooted root)
