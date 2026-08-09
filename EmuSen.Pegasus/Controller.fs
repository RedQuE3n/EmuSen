module EmuSen.Pegasus.Controller

open System
open System.IO
open System.Threading
open EmuSen.Pegasus
open EmuSen.Pegasus

type ConnectionState =
    | Offline
    | Hosting of code: string * port: int
    | Waiting of code: string * port: int
    | Connected of peer: string
    | Failed of reason: string

/// Where notes live. Resolved through SpecialFolder rather than a literal
/// ".local/share", which was Linux-only and wrong on the two other RIDs we
/// publish. An existing workspace at the old path keeps being used rather than
/// being stranded, the same order ConfigStore already follows for settings.
/// See EmuSen_Pegasus.md §11.
let defaultWorkspaceRoot =
    let home = Environment.GetFolderPath Environment.SpecialFolder.UserProfile

    let data =
        match Environment.GetFolderPath Environment.SpecialFolder.LocalApplicationData with
        | "" -> Path.Combine(home, ".local", "share")
        | path -> path

    let current = Path.Combine(data, "Pegasus", "workspace")
    let legacy = Path.Combine(home, ".local", "share", "pegasus", "workspace")

    if not (Directory.Exists current) && Directory.Exists legacy then
        legacy
    else
        current

/// Owns the workspace, the open note and the session, so the view stays a
/// function of state. Compaction threshold and projection policy live here.
type Notepad(root: string, displayName: string) =
    let workspace = new Workspace(root)

    let self =
        { Id = PeerId.New()
          Name = displayName
          Color = "#7c5cff" }

    let changed = Event<unit>()
    let stateChanged = Event<ConnectionState>()
    let remotePresence = Event<Presence>()

    let mutable openNote: (NoteId * DocumentActor * Store.NoteFile * IDisposable) option = None
    let mutable session: Session option = None
    let mutable host: Host option = None
    let mutable connection = Offline
    let mutable cts = new CancellationTokenSource()

    /// Rewrites the log as one snapshot once it has grown past this many
    /// records. See EmuSen_Pegasus.md §6.3.
    let compactThreshold = 512

    let setState s =
        connection <- s
        stateChanged.Trigger s

    let closeNote () =
        match openNote with
        | Some(_, doc, file, sub) ->
            sub.Dispose()
            file.WriteProjection doc.Text
            if file.RecordCount > compactThreshold then file.Compact doc.Snapshot
            file.Sync()
            (file :> IDisposable).Dispose()
            (doc :> IDisposable).Dispose()
        | None -> ()

        openNote <- None

    member _.Self = self
    member _.Workspace = workspace
    member _.Changed = changed.Publish
    member _.ConnectionChanged = stateChanged.Publish
    member _.RemotePresence = remotePresence.Publish
    member _.Connection = connection
    member _.Notes = workspace.Notes

    member _.CurrentNoteId = openNote |> Option.map (fun (id, _, _, _) -> id)
    member _.Document = openNote |> Option.map (fun (_, doc, _, _) -> doc)

    member _.Text =
        match openNote with
        | Some(_, doc, _, _) -> doc.Text
        | None -> ""

    member this.Open(id: NoteId) =
        if this.CurrentNoteId <> Some id then
            closeNote ()
            let doc, file, sub = workspace.OpenNote id
            openNote <- Some(id, doc, file, sub)
            doc.Changed.Add(fun () -> changed.Trigger())
            changed.Trigger()

    member this.CreateNote(name: string) =
        let entry = workspace.Create name
        this.Open entry.Id
        entry

    member _.Rename(id, name) =
        workspace.Rename(id, name)
        changed.Trigger()

    member _.Edit(text: string) =
        match openNote with
        | Some(_, doc, _, _) -> doc.ReplaceAll text
        | None -> ()

    /// Writes the readable projection and forces the log to media.
    member _.Checkpoint() =
        match openNote with
        | Some(_, doc, file, _) ->
            file.WriteProjection doc.Text
            file.Sync()
        | None -> ()

    member private this.Attach(s: Session) =
        session <- Some s
        s.PresenceChanged.Add remotePresence.Trigger
        s.PeerJoined.Add(fun p -> setState (Connected p.Name))
        s.Faulted.Add(fun e -> setState (Failed e.Message))
        s.Closed.Add(fun () -> if connection <> Offline then setState Offline)
        s.RunAsync() |> ignore

    /// Starts listening and returns the code the other peer needs.
    member this.StartHosting(?port: int) =
        match this.Document with
        | None -> failwith "open a note before hosting"
        | Some doc ->

        this.Disconnect()
        cts <- new CancellationTokenSource()
        let code = Crypto.newJoinCode ()
        let h = new Host(defaultArg port 0, code, self, doc)
        h.Start()
        host <- Some h
        setState (Waiting(code, h.Port))

        task {
            try
                let! s = h.AcceptAsync cts.Token
                setState (Hosting(code, h.Port))
                this.Attach s
            with
            | :? OperationCanceledException -> ()
            | e -> setState (Failed e.Message)
        }
        |> ignore

        code, h.Port

    member this.Join(address: string, port: int, code: string) =
        match this.Document with
        | None -> failwith "open a note before joining"
        | Some doc ->

        this.Disconnect()
        cts <- new CancellationTokenSource()

        task {
            try
                let! s = Client.connectAsync address port code self doc cts.Token
                this.Attach s
                setState (Connected "connecting...")
            with e ->
                setState (Failed e.Message)
        }
        |> ignore

    member _.Disconnect() =
        cts.Cancel()
        session |> Option.iter (fun s -> (s :> IDisposable).Dispose())
        session <- None
        host |> Option.iter (fun h -> (h :> IDisposable).Dispose())
        host <- None
        setState Offline

    interface IDisposable with
        member this.Dispose() =
            this.Disconnect()
            closeNote ()
            (workspace :> IDisposable).Dispose()
            cts.Dispose()
