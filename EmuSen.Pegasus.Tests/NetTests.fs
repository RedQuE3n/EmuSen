module EmuSen.Pegasus.Tests.NetTests

open System
open System.Threading
open System.Threading.Tasks
open Xunit
open EmuSen.Pegasus
open EmuSen.Pegasus

let private peer name colour =
    { Id = PeerId.New()
      Name = name
      Color = colour }

/// Waits for a condition rather than sleeping a fixed time, so the suite is not
/// timing-fragile on a loaded machine.
let private waitFor (timeoutMs: int) (condition: unit -> bool) =
    let deadline = DateTime.UtcNow.AddMilliseconds(float timeoutMs)

    while not (condition ()) && DateTime.UtcNow < deadline do
        Thread.Sleep 10

    condition ()

/// Brings up a host and a joiner on loopback, both pumping. No window, no
/// second process -- the discipline described in EmuSen_Pegasus.md §5.
type private Pair() =
    let code = Crypto.newJoinCode ()
    let alice = new DocumentActor()
    let bob = new DocumentActor()
    let aliceInfo = peer "alice" "#ff0000"
    let bobInfo = peer "bob" "#0000ff"
    let host = new Host(0, code, aliceInfo, alice)
    do host.Start()

    let accepted = host.AcceptAsync CancellationToken.None

    let joiner =
        (Client.connectAsync "127.0.0.1" host.Port code bobInfo bob CancellationToken.None)
            .GetAwaiter()
            .GetResult()

    let hostSession = accepted.GetAwaiter().GetResult()
    let hostRun = hostSession.RunAsync()
    let joinerRun = joiner.RunAsync()

    member _.Alice = alice
    member _.Bob = bob
    member _.HostSession = hostSession
    member _.JoinerSession = joiner
    member _.JoinCode = code
    member _.HostPort = host.Port

    interface IDisposable with
        member _.Dispose() =
            (hostSession :> IDisposable).Dispose()
            (joiner :> IDisposable).Dispose()
            (host :> IDisposable).Dispose()
            Task.WaitAll([| hostRun :> Task; joinerRun :> Task |], 2000) |> ignore
            (alice :> IDisposable).Dispose()
            (bob :> IDisposable).Dispose()

[<Fact>]
let ``an edit on one peer reaches the other`` () =
    use pair = new Pair()
    pair.Alice.Insert(0, "hello from alice")
    Assert.True(waitFor 5000 (fun () -> pair.Bob.Text = "hello from alice"))

[<Fact>]
let ``edits flow in both directions`` () =
    use pair = new Pair()
    pair.Alice.Insert(0, "AAA")
    Assert.True(waitFor 5000 (fun () -> pair.Bob.Text = "AAA"))
    pair.Bob.Insert(pair.Bob.Length, "BBB")
    Assert.True(waitFor 5000 (fun () -> pair.Alice.Text = "AAABBB"))

[<Fact>]
let ``a peer that connects late receives the existing document`` () =
    // Alice types before anyone joins; the initial SyncStep1/SyncStep2 exchange
    // is what has to carry it across.
    let code = Crypto.newJoinCode ()
    use alice = new DocumentActor()
    alice.Insert(0, "written before bob arrived")
    use bob = new DocumentActor()

    use host = new Host(0, code, peer "alice" "#f00", alice)
    host.Start()
    let accepted = host.AcceptAsync CancellationToken.None

    use joiner =
        (Client.connectAsync "127.0.0.1" host.Port code (peer "bob" "#00f") bob CancellationToken.None)
            .GetAwaiter()
            .GetResult()

    use hostSession = accepted.GetAwaiter().GetResult()
    hostSession.RunAsync() |> ignore
    joiner.RunAsync() |> ignore

    Assert.True(waitFor 5000 (fun () -> bob.Text = "written before bob arrived"))

[<Fact>]
let ``concurrent edits made while connected converge`` () =
    use pair = new Pair()
    pair.Alice.Insert(0, "shared. ")
    Assert.True(waitFor 5000 (fun () -> pair.Bob.Text = "shared. "))

    // Both type without waiting for the other.
    pair.Alice.Insert(pair.Alice.Length, "ALICE ")
    pair.Bob.Insert(pair.Bob.Length, "BOB ")

    Assert.True(waitFor 5000 (fun () -> pair.Alice.Text = pair.Bob.Text))
    Assert.Contains("ALICE", pair.Alice.Text)
    Assert.Contains("BOB", pair.Alice.Text)

[<Fact>]
let ``presence carries a peer's caret to the other side`` () =
    use pair = new Pair()
    let seen = ResizeArray<Presence>()
    use _sub = pair.HostSession.PresenceChanged.Subscribe seen.Add
    pair.JoinerSession.SendPresence(7, 3).GetAwaiter().GetResult()

    Assert.True(waitFor 5000 (fun () -> seen.Count > 0))
    Assert.Equal(7, seen[0].Caret)
    Assert.Equal(3, seen[0].Anchor)
    Assert.Equal("bob", seen[0].Peer.Name)

[<Fact>]
let ``the peers exchange identities on connect`` () =
    // Read from RemotePeer rather than the event: Hello arrives once, and a
    // subscriber attaching after the session started would never see it.
    use pair = new Pair()
    Assert.True(waitFor 5000 (fun () -> pair.JoinerSession.RemotePeer.IsSome))
    Assert.Equal("alice", pair.JoinerSession.RemotePeer.Value.Name)
    Assert.True(waitFor 5000 (fun () -> pair.HostSession.RemotePeer.IsSome))
    Assert.Equal("bob", pair.HostSession.RemotePeer.Value.Name)

[<Fact>]
let ``a wrong join code is refused at the handshake`` () =
    use alice = new DocumentActor()
    use bob = new DocumentActor()
    use host = new Host(0, "7-lantern-quartz", peer "alice" "#f00", alice)
    host.Start()
    let accepted = host.AcceptAsync CancellationToken.None

    let connecting =
        Client.connectAsync "127.0.0.1" host.Port "7-lantern-cobalt" (peer "bob" "#00f") bob CancellationToken.None

    // The joiner answers the challenge wrongly, so the host rejects it. The
    // joiner itself cannot tell yet -- it learns when the stream closes.
    let failed =
        try
            accepted.GetAwaiter().GetResult() |> ignore
            false
        with :? ProtocolError ->
            true

    Assert.True failed
    try
        (connecting.GetAwaiter().GetResult() :> IDisposable).Dispose()
    with _ ->
        ()
