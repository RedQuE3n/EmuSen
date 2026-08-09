module EmuSen.Pegasus.Tests.UiTests

open System
open System.IO
open System.Threading
open Avalonia
open Avalonia.Controls
open Avalonia.Controls.Primitives
open Avalonia.Headless
open Avalonia.LogicalTree
open Avalonia.Threading
open Avalonia.Themes.Fluent
open Xunit
open EmuSen.Pegasus
open EmuSen.Pegasus.Controller

type private HeadlessApp() =
    inherit Application()
    // Shell.applyTheme, not a bare FluentTheme: loading a different theme than
    // the application loads is what let a blank window pass the suite.
    override this.Initialize() = Shell.applyTheme this

// Avalonia may only be initialised once per process, so every test shares this.
// LunaApp is deliberately not used here -- it resolves the saved theme through
// ConfigStore, which a test has no business touching. See EmuSen_Pegasus.md §9.
let started =
    lazy
        (AppBuilder
            .Configure<HeadlessApp>()
            .UseHeadless(AvaloniaHeadlessPlatformOptions(UseHeadlessDrawing = true))
            .SetupWithoutStarting()
         |> ignore)

let tempRoot () =
    let dir = Path.Combine(Path.GetTempPath(), "pegasus-ui", Guid.NewGuid().ToString "N")
    Directory.CreateDirectory dir |> ignore
    dir

let private editorOf (window: Window) =
    window.GetLogicalDescendants()
    |> Seq.choose (fun c ->
        match box c with
        | :? TextBox as t when t.AcceptsReturn -> Some t
        | _ -> None)
    |> Seq.head

let private pump (predicate: unit -> bool) =
    let deadline = DateTime.UtcNow.AddSeconds 5.0

    while not (predicate ()) && DateTime.UtcNow < deadline do
        Dispatcher.UIThread.RunJobs()
        Thread.Sleep 10

    Dispatcher.UIThread.RunJobs()
    predicate ()

[<Fact>]
let ``the window renders an editor bound to the open note`` () =
    started.Force()
    use pad = new Notepad(tempRoot (), "alice")
    pad.CreateNote "scratch" |> ignore

    let window = Shell.PegasusWindow pad
    window.Show()
    Dispatcher.UIThread.RunJobs()

    let editor = editorOf window
    editor.Text <- "typed into the window"
    Dispatcher.UIThread.RunJobs()

    Assert.Equal("typed into the window", pad.Text)
    window.Close()

[<Fact>]
let ``every control in the window is actually templated`` () =
    // The blank-window regression: without LunaP's theme the controls exist in
    // the logical tree and render nothing, so every other test here still
    // passed. Applying a template is the thing that was missing, so it is the
    // thing asserted. See EmuSen_Pegasus.md §11.
    started.Force()
    use pad = new Notepad(tempRoot (), "alice")
    pad.CreateNote "scratch" |> ignore

    let window = Shell.PegasusWindow pad
    window.Show()
    Dispatcher.UIThread.RunJobs()
    window.Measure(Size(1000.0, 680.0))
    window.Arrange(Rect(0.0, 0.0, 1000.0, 680.0))
    Dispatcher.UIThread.RunJobs()

    let untemplated =
        window.GetLogicalDescendants()
        |> Seq.choose (fun c ->
            match box c with
            | :? TemplatedControl as t when isNull t.Template -> Some(t.GetType().Name)
            | _ -> None)
        |> Seq.distinct
        |> Seq.toArray

    Assert.True(
        untemplated.Length = 0,
        $"""controls with no template, so they render blank: {String.Join(", ", untemplated)}"""
    )

    window.Close()

[<Fact>]
let ``the window is a LunaP ToolWindow and carries a placement key`` () =
    // The point of the move: chrome, theme and geometry come from the shared
    // toolkit rather than being rebuilt here. EmuSen_Pegasus.md §8.
    started.Force()
    use pad = new Notepad(tempRoot (), "alice")
    pad.CreateNote "scratch" |> ignore
    let window = Shell.PegasusWindow pad
    Assert.IsAssignableFrom<EmuSen.LunaP.Windowing.ToolWindow>(window) |> ignore
    Assert.Equal("pegasus", window.WindowKey)
    window.Close()

[<Fact>]
let ``a remote edit appears in the rendered editor`` () =
    started.Force()
    use hostPad = new Notepad(tempRoot (), "alice")
    use joinPad = new Notepad(tempRoot (), "bob")
    hostPad.CreateNote "shared" |> ignore
    joinPad.CreateNote "shared" |> ignore

    let window = Shell.PegasusWindow joinPad
    window.Show()
    Dispatcher.UIThread.RunJobs()

    let code, port = hostPad.StartHosting()
    joinPad.Join("127.0.0.1", port, code)
    Assert.True(pump (fun () -> hostPad.Connection <> Offline && joinPad.Connection <> Offline))

    hostPad.Edit "written on the other machine"

    // The whole path: mailbox -> socket -> mailbox -> dispatcher -> TextBox.
    let editor = editorOf window
    Assert.True(pump (fun () -> editor.Text = "written on the other machine"))

    hostPad.Disconnect()
    joinPad.Disconnect()
    window.Close()

[<Fact>]
let ``a note survives closing and reopening the notepad`` () =
    started.Force()
    let root = tempRoot ()

    let noteId =
        use pad = new Notepad(root, "alice")
        let entry = pad.CreateNote "durable"
        pad.Edit "this must still be here"
        pad.Checkpoint()
        entry.Id

    use reopened = new Notepad(root, "alice")
    reopened.Open noteId
    Assert.Equal("this must still be here", reopened.Text)
    Assert.Contains(reopened.Notes, fun n -> n.Name = "durable")
