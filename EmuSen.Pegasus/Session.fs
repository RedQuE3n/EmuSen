namespace EmuSen.Pegasus

open System
open System.Buffers.Binary
open System.IO
open System.Net
open System.Net.Sockets
open System.Threading
open EmuSen.Pegasus

/// Length-prefixed sealed frames over a stream. Wire layout in
/// EmuSen_Pegasus.md §10.3.
module Framing =

    let private readExactly (stream: Stream) (count: int) (ct: CancellationToken) =
        task {
            let buffer = Array.zeroCreate<byte> count
            let mutable read = 0

            while read < count do
                let! n = stream.ReadAsync(Memory(buffer, read, count - read), ct)

                if n = 0 then
                    raise (EndOfStreamException "peer closed the connection")

                read <- read + n

            return buffer
        }

    let writeFrame (stream: Stream) (key: byte[]) (frame: Frame) (ct: CancellationToken) =
        task {
            let sealedBytes = Crypto.seal key (Codec.encode frame)
            let prefix = Array.zeroCreate<byte> 4
            BinaryPrimitives.WriteInt32LittleEndian(Span prefix, sealedBytes.Length)
            do! stream.WriteAsync(ReadOnlyMemory prefix, ct)
            do! stream.WriteAsync(ReadOnlyMemory sealedBytes, ct)
            do! stream.FlushAsync ct
        }

    let readFrame (stream: Stream) (key: byte[]) (ct: CancellationToken) =
        task {
            let! prefix = readExactly stream 4 ct
            let length = BinaryPrimitives.ReadInt32LittleEndian(ReadOnlySpan prefix)

            // Checked before allocating, so a hostile length cannot exhaust memory.
            if length <= 0 || length > Codec.MaxFrameBytes then
                raise (ProtocolError $"frame length {length} is out of range")

            let! sealedBytes = readExactly stream length ct
            return Codec.decode (Crypto.openSealed key sealedBytes)
        }

/// Proves both ends derived the same key from the join code, without sending it.
module Handshake =

    let private challengeLength = 32

    let asHost (stream: Stream) (key: byte[]) (ct: CancellationToken) =
        task {
            let challenge = Crypto.newChallenge ()
            do! stream.WriteAsync(ReadOnlyMemory challenge, ct)
            do! stream.FlushAsync ct
            let response = Array.zeroCreate<byte> 32
            let mutable read = 0

            while read < 32 do
                let! n = stream.ReadAsync(Memory(response, read, 32 - read), ct)
                if n = 0 then raise (ProtocolError "peer closed during handshake")
                read <- read + n

            if not (Crypto.verifyChallenge key challenge response) then
                raise (ProtocolError "join code did not match")
        }

    let asJoiner (stream: Stream) (key: byte[]) (ct: CancellationToken) =
        task {
            let challenge = Array.zeroCreate<byte> challengeLength
            let mutable read = 0

            while read < challengeLength do
                let! n = stream.ReadAsync(Memory(challenge, read, challengeLength - read), ct)
                if n = 0 then raise (ProtocolError "peer closed during handshake")
                read <- read + n

            let response = Crypto.respondToChallenge key challenge
            do! stream.WriteAsync(ReadOnlyMemory response, ct)
            do! stream.FlushAsync ct
        }

/// One connected peer. Both host and joiner drive this identically, which is
/// what keeps a future relay from needing a third role -- EmuSen_Pegasus.md §10.1.
type Session(stream: Stream, key: byte[], self: PeerInfo, document: DocumentActor) =
    let cts = new CancellationTokenSource()
    let peerJoined = Event<PeerInfo>()
    let presenceChanged = Event<Presence>()
    let faulted = Event<exn>()
    let closed = Event<unit>()

    // Serialises writes; two frames must never interleave on the stream.
    let writeLock = new SemaphoreSlim(1, 1)

    let send frame =
        task {
            do! writeLock.WaitAsync cts.Token

            try
                do! Framing.writeFrame stream key frame cts.Token
            finally
                writeLock.Release() |> ignore
        }

    let mutable updateSub: IDisposable = null
    let mutable remotePeer: PeerInfo option = None

    /// Retained, because Hello arrives once and a late subscriber would miss it.
    member _.RemotePeer = remotePeer

    member _.PeerJoined = peerJoined.Publish
    member _.PresenceChanged = presenceChanged.Publish
    member _.Faulted = faulted.Publish
    member _.Closed = closed.Publish

    member _.Send(frame) = send frame

    member this.SendPresence(caret, anchor) =
        send (
            Awareness
                { Peer = self
                  Caret = caret
                  Anchor = anchor }
        )

    /// Greets, offers our state vector, then pumps frames until the peer goes.
    member this.RunAsync() =
        task {
            try
                // Local edits go out as they happen; remote ones never echo back.
                // Never block here -- this fires on the document's mailbox thread.
                updateSub <-
                    document.LocalUpdate.Subscribe(fun update ->
                        let sending = send (Update update)

                        sending.ContinueWith(
                            (fun (t: Threading.Tasks.Task) -> faulted.Trigger t.Exception),
                            Threading.Tasks.TaskContinuationOptions.OnlyOnFaulted
                        )
                        |> ignore)

                do! send (Hello self)
                do! send (SyncStep1 document.StateVector)

                while not cts.IsCancellationRequested do
                    let! frame = Framing.readFrame stream key cts.Token

                    match frame with
                    | Hello peer ->
                        remotePeer <- Some peer
                        peerJoined.Trigger peer
                    | SyncStep1 stateVector ->
                        // Answer with exactly what they lack.
                        do! send (SyncStep2(document.DiffSince stateVector))
                    | SyncStep2 diff
                    | Update diff -> document.ApplyRemote diff
                    | Awareness presence -> presenceChanged.Trigger presence
                    | Bye -> cts.Cancel()
            with
            | :? OperationCanceledException -> ()
            | :? EndOfStreamException -> ()
            | e -> faulted.Trigger e

            closed.Trigger()
        }

    interface IDisposable with
        member _.Dispose() =
            cts.Cancel()
            if not (isNull updateSub) then updateSub.Dispose()
            cts.Dispose()
            writeLock.Dispose()
            stream.Dispose()

/// Accepts one joiner on a TCP port.
type Host(port: int, joinCode: string, self: PeerInfo, document: DocumentActor) =
    let key = Crypto.deriveKey joinCode
    let listener = new TcpListener(IPAddress.Any, port)

    member _.Port =
        (listener.LocalEndpoint :?> IPEndPoint).Port

    member _.Start() = listener.Start()

    member this.AcceptAsync(ct: CancellationToken) =
        task {
            let! client = listener.AcceptTcpClientAsync ct
            client.NoDelay <- true
            let stream = client.GetStream()
            do! Handshake.asHost stream key ct
            return new Session(stream, key, self, document)
        }

    interface IDisposable with
        member _.Dispose() = listener.Stop()

/// Connects to a host.
module Client =

    let connectAsync (host: string) (port: int) (joinCode: string) (self: PeerInfo) (document: DocumentActor) (ct: CancellationToken) =
        task {
            let key = Crypto.deriveKey joinCode
            let client = new TcpClient()
            do! client.ConnectAsync(host, port, ct)
            client.NoDelay <- true
            let stream = client.GetStream()
            do! Handshake.asJoiner stream key ct
            return new Session(stream, key, self, document)
        }
