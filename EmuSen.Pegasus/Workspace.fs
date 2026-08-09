namespace EmuSen.Pegasus

open System
open System.IO

type NoteEntry =
    { Id: NoteId
      Name: string
      Deleted: bool }

/// A directory of notes plus an index that is itself a note, so creation and
/// rename merge through the same CRDT as text. See EmuSen_Pegasus.md §10.6.
type Workspace(root: string) =
    let root = Path.GetFullPath root
    do Directory.CreateDirectory root |> ignore

    let indexPath = Path.Combine(root, "_index.pegasus")
    let indexFile = new Store.NoteFile(indexPath)

    let index = new DocumentActor()

    do
        for update in indexFile.Recovered do
            index.ApplyRemote update

    let indexSub = index.LocalUpdate.Subscribe indexFile.Append

    // The index document holds one line per note as "id\tname\tdeleted", which
    // keeps it a plain Y.Text and avoids a second root type -- Pegasus_Sync.md §6.
    let parse (line: string) =
        match line.Split '\t' with
        | [| id; name; deleted |] ->
            Some
                { Id = NoteId id
                  Name = name
                  Deleted = deleted = "1" }
        | _ -> None

    let entries () =
        index.Text.Split('\n', StringSplitOptions.RemoveEmptyEntries)
        |> Array.choose parse
        // A rename appends a fresh line, so the last mention of an id wins.
        |> Array.groupBy _.Id
        |> Array.map (fun (_, group) -> Array.last group)

    let append (entry: NoteEntry) =
        let flag = if entry.Deleted then "1" else "0"
        index.Insert(index.Length, $"{entry.Id.Value}\t{entry.Name}\t{flag}\n")

    member _.Root = root
    member _.IndexDocument = index

    /// Live notes, most recently named last.
    member _.Notes = entries () |> Array.filter (fun e -> not e.Deleted)

    member _.AllNotes = entries ()

    member _.PathOf(id: NoteId) = Path.Combine(root, id.Value + ".pegasus")

    member this.Create(name: string) =
        let entry =
            { Id = NoteId.New()
              Name = name
              Deleted = false }

        append entry
        // Touch the file so the note exists on disk before anyone types in it.
        (new Store.NoteFile(this.PathOf entry.Id, entry.Id) :> IDisposable).Dispose()
        entry

    member _.Rename(id: NoteId, name: string) =
        append
            { Id = id
              Name = name
              Deleted = false }

    /// Tombstones the entry. The file stays -- EmuSen_Pegasus.md §10.6.
    member _.Delete(id: NoteId) =
        match entries () |> Array.tryFind (fun e -> e.Id = id) with
        | Some existing -> append { existing with Deleted = true }
        | None -> ()

    /// Opens a note's replica, replaying its log, and keeps the log appended to.
    member this.OpenNote(id: NoteId) =
        let file = new Store.NoteFile(this.PathOf id, id)
        let doc = new DocumentActor()

        for update in file.Recovered do
            doc.ApplyRemote update

        let sub = doc.LocalUpdate.Subscribe file.Append
        doc, file, sub

    interface IDisposable with
        member _.Dispose() =
            indexSub.Dispose()
            (index :> IDisposable).Dispose()
            (indexFile :> IDisposable).Dispose()
