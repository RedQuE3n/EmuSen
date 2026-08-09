module EmuSen.Pegasus.Program

open System
open Avalonia
open Avalonia.Controls.ApplicationLifetimes
open EmuSen.LunaP
open EmuSen.Pegasus.Controller

type App() =
    inherit Application()

    let pad = new Notepad(defaultWorkspaceRoot, Environment.UserName)

    override this.Initialize() = Shell.applyTheme this

    override this.OnFrameworkInitializationCompleted() =
        match this.ApplicationLifetime with
        | :? IClassicDesktopStyleApplicationLifetime as desktop ->
            // Start on a note so the editor is usable immediately.
            match pad.Notes |> Array.tryHead with
            | Some first -> pad.Open first.Id
            | None -> pad.CreateNote "scratch" |> ignore

            desktop.MainWindow <- Shell.PegasusWindow pad
            desktop.ShutdownRequested.Add(fun _ -> (pad :> IDisposable).Dispose())
        | _ -> ()

        base.OnFrameworkInitializationCompleted()

[<EntryPoint>]
let main argv =
    // LunaApp, not a hand-rolled AppBuilder: it applies the saved theme and
    // picks X11, which UsePlatformDetect does not do on Wayland.
    LunaApp.Configure<App>().StartWithClassicDesktopLifetime argv
