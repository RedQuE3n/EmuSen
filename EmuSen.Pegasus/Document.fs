namespace EmuSen.Pegasus

open System
open System.Security.Cryptography
open YDotNet.Document
open YDotNet.Document.Options
open YDotNet.Document.StickyIndexes

/// Client ids are squeezed between two YDotNet 0.6.0 defects: the default
/// constructor draws from about 6 bits and collides, and any id at or above
/// 2^32 breaks delta sync outright. EmuSen_Pegasus.md §4.5 and §4.7.
module ClientId =

    /// Exclusive upper bound. At 2^32 and above, StateDiffV1 ignores the state
    /// vector and replicas diverge -- proven in Pegasus_Design.md §4.7.
    [<Literal>]
    let ExclusiveMax = 4294967296UL

    let fresh () =
        uint64 (BitConverter.ToUInt32(RandomNumberGenerator.GetBytes 4, 0)) % (ExclusiveMax - 1UL)
        + 1UL

/// Everything the caller can ask of a replica. One case per operation so the
/// mailbox loop is exhaustively matched.
type private Command =
    | Insert of index: int * text: string * AsyncReplyChannel<unit>
    | Delete of index: int * length: int * AsyncReplyChannel<unit>
    | ApplyRemote of update: byte[] * AsyncReplyChannel<unit>
    | ReadText of AsyncReplyChannel<string>
    | ReadLength of AsyncReplyChannel<int>
    | ReadStateVector of AsyncReplyChannel<byte[]>
    | ReadDiffSince of stateVector: byte[] * AsyncReplyChannel<byte[]>
    | TrackCaret of index: int * AsyncReplyChannel<StickyIndex>
    | ReadCaret of sticky: StickyIndex * AsyncReplyChannel<int>
    | Shutdown of AsyncReplyChannel<unit>

/// A Yjs replica of one note, owned by a single mailbox.
///
/// YDotNet throws on overlapping transactions and its Doc.Text handle may only
/// be taken outside one, so every read and write is funnelled through here and
/// no Doc handle escapes. See EmuSen_Pegasus.md §4.2.
type DocumentActor(?seed: byte[], ?clientId: uint64) as this =
    // Never Doc() -- see ClientId above and EmuSen_Pegasus.md §4.5.
    let doc = new Doc(DocOptions(Id = defaultArg clientId (ClientId.fresh ())))

    // Must be taken before any transaction is opened -- Pegasus_Design.md §4.2.
    let body = doc.Text "body"

    let localUpdate = Event<byte[]>()
    let changed = Event<unit>()

    // Set only while a remote update is being applied, and read by the observer
    // that fires synchronously inside Commit on this same mailbox thread; it is
    // what stops a remote update being echoed back to its sender.
    let mutable applyingRemote = false

    let subscription =
        doc.ObserveUpdatesV1(fun e ->
            if not applyingRemote then
                localUpdate.Trigger e.Update)

    let write (f: Transactions.Transaction -> unit) =
        use tx = doc.WriteTransaction null
        f tx
        tx.Commit()

    let read (f: Transactions.Transaction -> 'a) =
        use tx = doc.ReadTransaction()
        f tx

    let agent =
        MailboxProcessor<Command>.Start(fun inbox ->
            let rec loop () =
                async {
                    let! cmd = inbox.Receive()

                    match cmd with
                    | Insert(index, text, reply) ->
                        write (fun tx -> body.Insert(tx, uint32 index, text, null))
                        changed.Trigger()
                        reply.Reply()
                        return! loop ()
                    | Delete(index, length, reply) ->
                        write (fun tx -> body.RemoveRange(tx, uint32 index, uint32 length))
                        changed.Trigger()
                        reply.Reply()
                        return! loop ()
                    | ApplyRemote(update, reply) ->
                        applyingRemote <- true

                        try
                            write (fun tx -> tx.ApplyV1 update |> ignore)
                        finally
                            applyingRemote <- false

                        changed.Trigger()
                        reply.Reply()
                        return! loop ()
                    | ReadText reply ->
                        reply.Reply(read (fun tx -> body.String tx))
                        return! loop ()
                    | ReadLength reply ->
                        reply.Reply(read (fun tx -> int (body.Length tx)))
                        return! loop ()
                    | ReadStateVector reply ->
                        reply.Reply(read (fun tx -> Array.copy (tx.StateVectorV1())))
                        return! loop ()
                    | ReadDiffSince(sv, reply) ->
                        reply.Reply(read (fun tx -> Array.copy (tx.StateDiffV1 sv)))
                        return! loop ()
                    | TrackCaret(index, reply) ->
                        let sticky =
                            use tx = doc.WriteTransaction null
                            let s = body.StickyIndex(tx, uint32 index, StickyAssociationType.After)
                            tx.Commit()
                            s

                        reply.Reply sticky
                        return! loop ()
                    | ReadCaret(sticky, reply) ->
                        reply.Reply(read (fun tx -> int (sticky.Read tx)))
                        return! loop ()
                    | Shutdown reply ->
                        reply.Reply()
                        return ()
                }

            loop ())

    do
        match seed with
        | Some bytes when bytes.Length > 0 -> this.ApplyRemote bytes
        | _ -> ()

    /// This replica's Yjs client id, distinct per replica by construction.
    member _.ClientId = doc.Id

    /// Updates originating from this peer, to be sent onward. Remote updates
    /// are deliberately not raised here.
    member _.LocalUpdate = localUpdate.Publish

    /// Raised after any change from any source, for the UI to refresh on.
    member _.Changed = changed.Publish

    member _.Insert(index, text) = agent.PostAndReply(fun r -> Insert(index, text, r))
    member _.Delete(index, length) = agent.PostAndReply(fun r -> Delete(index, length, r))
    member _.ApplyRemote(update) = agent.PostAndReply(fun r -> ApplyRemote(update, r))
    member _.Text = agent.PostAndReply ReadText
    member _.Length = agent.PostAndReply ReadLength
    member _.StateVector = agent.PostAndReply ReadStateVector
    member _.DiffSince(stateVector) = agent.PostAndReply(fun r -> ReadDiffSince(stateVector, r))

    /// The whole document as one update, which is what a snapshot record holds.
    member this.Snapshot = this.DiffSince null

    /// A caret that survives concurrent edits, rather than a raw offset.
    member _.TrackCaret(index) = agent.PostAndReply(fun r -> TrackCaret(index, r))
    member _.ReadCaret(sticky) = agent.PostAndReply(fun r -> ReadCaret(sticky, r))

    /// Replace the whole text, used when the UI hands back an edited buffer.
    member this.ReplaceAll(text: string) =
        let current = this.Text

        if current <> text then
            let prefix =
                let limit = min current.Length text.Length

                let rec scan i =
                    if i < limit && current[i] = text[i] then scan (i + 1) else i

                scan 0

            let suffix =
                let limit = min (current.Length - prefix) (text.Length - prefix)

                let rec scan i =
                    if i < limit && current[current.Length - 1 - i] = text[text.Length - 1 - i] then
                        scan (i + 1)
                    else
                        i

                scan 0

            let removed = current.Length - prefix - suffix
            if removed > 0 then this.Delete(prefix, removed)
            let inserted = text.Substring(prefix, text.Length - prefix - suffix)
            if inserted.Length > 0 then this.Insert(prefix, inserted)

    interface IDisposable with
        member _.Dispose() =
            agent.PostAndReply Shutdown
            subscription.Dispose()
            (agent :> IDisposable).Dispose()
            doc.Dispose()

/// Where a caret belongs after the buffer changed underneath it. Pure, so the
/// rule is tested without a window -- EmuSen_Pegasus.md §5.
module Caret =

    let adjust (oldText: string) (newText: string) (caret: int) =
        let caret = max 0 (min caret oldText.Length)

        let mutable prefix = 0

        while prefix < oldText.Length
              && prefix < newText.Length
              && oldText[prefix] = newText[prefix] do
            prefix <- prefix + 1

        if caret <= prefix then
            // The change begins at or after the caret; it does not move.
            caret
        else
            let mutable suffix = 0

            while suffix < oldText.Length - prefix
                  && suffix < newText.Length - prefix
                  && oldText[oldText.Length - 1 - suffix] = newText[newText.Length - 1 - suffix] do
                suffix <- suffix + 1

            let delta = newText.Length - oldText.Length

            if caret >= oldText.Length - suffix then
                // Wholly after the changed span, so it shifts by the size change.
                caret + delta
            else
                // Inside the replaced span; the old position no longer exists.
                max prefix (min (caret + delta) (newText.Length - suffix))
