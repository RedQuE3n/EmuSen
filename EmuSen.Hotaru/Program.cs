using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using Avalonia;
using EmuSen.Common;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.Hotaru.Views;

namespace EmuSen.Hotaru
{
    class Program
    {
        // Backs both `core <corename> <path>` code paths: RunStandaloneShell's
        // own pre-window handling below (TryResolveCoreCommand), and, once a
        // window exists, EmuSen.DianaOS.Commands.CoreCommand (registered as
        // part of debugCmd's own extraCommands in Main). One shared registry,
        // not two - `CoreDescriptor` itself lives in EmuSen.DianaOS.Commands
        // now (promoted there for CoreCommand's own use, see that file's own
        // comment on why it's core-agnostic despite the name) rather than
        // staying a private nested type here. Only one entry today because
        // only one core is actually implemented (`VenusCore`, SNES) -
        // registered under both its internal codename and the console name
        // most people would actually type.
        private static readonly Dictionary<string, EmuSen.DianaOS.Commands.CoreDescriptor> _coreRegistry = new(StringComparer.OrdinalIgnoreCase)
        {
            ["venus"] = new EmuSen.DianaOS.Commands.CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
            ["snes"] = new EmuSen.DianaOS.Commands.CoreDescriptor("SNES (Venus)", new[] { ".smc", ".sfc" }),
        };

        static void Main(string[] args)
        {
            TextWriter originalOut = Console.Out;

            // Disposed by FlushAndDispose() below on every exit path
            // (normal completion, Ctrl+C, `kill <pid>`, or an unhandled
            // exception on some other thread) - CategorizedLogWriter's
            // file writes happen on a background thread with no
            // AutoFlush, so something has to explicitly wait for that
            // queue to drain before the process exits, or the last batch
            // of buffered log lines is silently lost.
            CategorizedLogWriter? logWriter = null;

            // Needed by FlushAndDispose to flush Cpu/Spc700's
            // RepeatCollapsingTrace state before the log writer closes -
            // see that call's own comment below.
            VenusCore? core = null;

            // Guards FlushAndDispose against running twice - Ctrl+C alone
            // can reach it via both the SIGINT registration below and the
            // ProcessExit that follows once the default handler decides to
            // terminate, and a crash can reach it via UnhandledException
            // too. Dispose() itself isn't safe to call twice
            // (CategorizedLogWriter closes file handles the second call
            // would touch again), so only the first caller should
            // actually run it.
            int shutdownGuard = 0;

            void FlushAndDispose()
            {
                if (Interlocked.Exchange(ref shutdownGuard, 1) != 0) return;

                // Must run before logWriter.Dispose() below, while
                // Console.Out is still routed through it - otherwise a
                // loop CpuVerboseLogging/Spc700VerboseLogging was still
                // mid-repeat on would never reach the file at all. See
                // DebugTools.RepeatCollapsingTrace<TKey>.Flush().
                core?.Cpu?.FlushVerboseTrace();
                core?.Spc700?.FlushVerboseTrace();

                Console.SetOut(originalOut);
                logWriter?.Dispose();
            }

            // Covers every exit path that doesn't unwind through Main's
            // own return: `kill <pid>` (SIGTERM) or Ctrl+C (SIGINT) from a
            // terminal, and an unhandled exception thrown on a thread
            // other than this one. Cannot catch a hard `kill -9`/SIGKILL -
            // nothing in userspace can intercept that signal at all.
            AppDomain.CurrentDomain.ProcessExit += (_, _) => FlushAndDispose();
            AppDomain.CurrentDomain.UnhandledException += (_, _) => FlushAndDispose();
            using PosixSignalRegistration sigTerm = PosixSignalRegistration.Create(PosixSignal.SIGTERM, ctx =>
            {
                FlushAndDispose();
                ctx.Cancel = false; // let the process actually terminate after flushing
            });
            using PosixSignalRegistration sigInt = PosixSignalRegistration.Create(PosixSignal.SIGINT, ctx =>
            {
                FlushAndDispose();
                ctx.Cancel = false;
            });

            // ROM path resolution - see EmuSen_Frontend_Driver.md §1.
            // DianaOS is the ONLY thing shown at launch, whether or not a
            // ROM path was given on the command line - RunStandaloneShell
            // runs as plain console I/O, entirely before Avalonia (or any
            // window at all) ever starts. Returns null only if the user
            // actually asked to shut down (or hit EOF) with no ROM ever
            // resolved, in which case there's nothing left to do.
            string? romPathFromShell = RunStandaloneShell(args.Length > 0 ? args[0] : null);
            if (romPathFromShell is null) return;
            string romPath = romPathFromShell;

            try
            {
                // Separate from var/games/ (battery-backed cartridge SRAM,
                // owned by Cartridge.SavePath) - a save state is a full
                // snapshot of emulator state, a different kind of artifact
                // with a different lifetime.
                string statePath = Path.Combine(Directory.GetCurrentDirectory(), "var", "lib", Path.GetFileNameWithoutExtension(romPath) + ".state");

                core = new VenusCore(headless: false);

                string logDir = Path.Combine(Directory.GetCurrentDirectory(), "var", "log", core.CoreName, $"console_{DateTime.Now:yyyyMMdd_HHmmss}");
                Directory.CreateDirectory(logDir);
                logWriter = new CategorizedLogWriter(originalOut, logDir);
                Console.SetOut(logWriter);

                core.LoadRom(romPath);
                DebugSettings.CpuVerboseLogging = true;
                DebugSettings.Spc700VerboseLogging = true;

                // The debug toolchain itself (SnesDebugTarget,
                // DianaOSInterpreter, the power-on watch registration) is
                // built inside GameWindow now, not here - it has to be
                // REBUILDABLE after a `core <name> <path>` swap
                // (HostAction.LoadCore), and GameWindow is the only thing
                // that ever reacts to that signal. Main just builds the
                // extraCommands list once: none of these three commands'
                // own delegates depend on anything that changes across a
                // swap - `core` itself is never replaced, only reloaded in
                // place (VenusCore.LoadRom is a re-initializer, not a
                // constructor-only step), so CoretopCommand/StateCommand's
                // closures over `core`/`statePath` and CoreCommand's own
                // static registry all stay valid for the whole process.
                // CoretopCommand replaces the standard registry's plain,
                // window-less default via extraCommands' override-by-name
                // behavior, same mechanism EmuSen.Mistress9 uses; State/
                // CoreCommand are both new registrations, not overrides.
                IDianaOSCommand[] extraCommands =
                {
                    new EmuSen.DianaOS.Commands.CoretopCommand(DebugWindows.ShowCoretopWindow),
                    new EmuSen.DianaOS.Commands.StateCommand(core.SaveState, core.LoadState, () => statePath),
                    new EmuSen.DianaOS.Commands.CoreCommand(_coreRegistry),
                };

                // Everything below hands off to GameWindow (Views/GameWindow.axaml.cs) -
                // the window, emulation thread, audio, input, and every
                // hotkey/debug-prompt concern now live there instead of a
                // Raylib-driven loop in this method. AppBuilder.Configure<App>
                // with a factory Func<App> (rather than the parameterless
                // Configure<App>()) is what lets the already-constructed
                // core/extraCommands/statePath above get threaded into the
                // Avalonia app instead of reconstructed inside
                // OnFrameworkInitializationCompleted. Blocks until the
                // window closes. GameWindow's own Closing handler already
                // runs core.SaveSram() and disposes audio/gamepad before
                // this returns - see Views/GameWindow.axaml.cs's own
                // Shutdown().
                BuildAvaloniaApp(core, extraCommands, statePath)
                    .StartWithClassicDesktopLifetime(args);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
            }
            finally
            {
                // Normal completion and the catch block's own CPU-HALT
                // print both funnel through here; the signal/ProcessExit
                // handlers above cover the exit paths that never reach
                // this finally at all.
                FlushAndDispose();
            }
        }

        // Mirrors EmuSen.Mistress9's own Program.cs/BuildAvaloniaApp idiom.
        // See Man pages/EmuSen_Project_Overview_v2.md §2a for why Linux
        // stays on UseWayland() for now.
        private static AppBuilder BuildAvaloniaApp(
            VenusCore core, IEnumerable<IDianaOSCommand> extraCommands, string statePath)
        {
            var builder = AppBuilder.Configure(() => new App(core, extraCommands, statePath))
                .UsePlatformDetect()
                .WithInterFont()
                .LogToTrace();

            if (OperatingSystem.IsLinux())
            {
                builder = builder.UseWayland();
            }

            return builder;
        }

        // Hotaru's entire launch experience: no core, no window, no audio
        // device yet, just DianaOS against a null IDebugTarget - the shell
        // is the first and only thing shown, whether Hotaru was launched
        // bare or with a ROM path on the command line. Deliberately its
        // own small loop rather than reusing GameWindow's own debug
        // prompt with a null core - that prompt's 'step'/'state'
        // shortcuts and "resume the game" framing don't mean anything
        // here, and threading null-core checks through it would make the
        // actually-in-a-game case harder to follow for no real benefit.
        //
        // `initialRomPath` is whatever came in as a CLI arg, if anything -
        // a valid file resolves and returns immediately, an invalid one
        // reports why and falls through into the same interactive loop a
        // bare launch gets.
        //
        // Returns the ROM path to actually launch, or null if the user
        // asked to shut down instead (or hit EOF).
        private static string? RunStandaloneShell(string? initialRomPath)
        {
            DianaOSInterpreter shell = DianaOSInterpreter.CreateDefault(null);
            Console.WriteLine(shell.GetWelcomeBanner(_coreRegistry.Values.Select(d => d.DisplayName).Distinct()));
            Console.WriteLine("--- DianaOS (type 'help', 'core <name> <path>' to launch a game, 'shutdown' to quit) ---");

            if (initialRomPath != null)
            {
                if (File.Exists(initialRomPath))
                {
                    Console.WriteLine($"[ROM] Loading: {initialRomPath}");
                    return initialRomPath;
                }
                Console.WriteLine($"[ERROR] ROM not found: {initialRomPath}");
            }

            while (true)
            {
                Console.Write(shell.IsAwaitingMoreInput ? "> " : "DianaOS #: ");
                string? line = ConsoleLineReader.ReadLine(shell.History.Entries);
                if (line is null) return null;
                string trimmed = line.Trim();

                if (!shell.IsAwaitingMoreInput && trimmed.StartsWith("core ", StringComparison.OrdinalIgnoreCase))
                {
                    string? validatedRomPath = TryResolveCoreCommand(trimmed);
                    if (validatedRomPath != null) return validatedRomPath;
                    continue; // TryResolveCoreCommand already printed why it failed
                }

                // 'shutdown'/'quit' is a real DianaOS command now
                // (EmuSen.DianaOS.Commands.ShutdownCommand, already in the
                // standard registry CreateDefault built above) - no
                // separate bypass needed here anymore, just react to the
                // same HostAction.Shutdown GameWindow's own RunDebugPrompt
                // reacts to (see that method's own comment).
                (_, string output, HostAction? action) = shell.Submit(line);
                if (output.Length > 0) Console.WriteLine(output);
                if (action is HostAction.Shutdown) return null;
            }
        }

        // `core <corename> <path>` - not a DianaOSInterpreter command (see
        // RunStandaloneShell's own comment); handled as plain string
        // parsing here instead. Prints its own error and returns null for
        // anything wrong so the caller can just loop back to the prompt -
        // only a fully validated ROM path is ever returned.
        private static string? TryResolveCoreCommand(string trimmedLine)
        {
            string[] parts = trimmedLine.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 3)
            {
                Console.WriteLine("Usage: core <corename> <path-to-rom>");
                return null;
            }

            string coreName = parts[1];
            string romPath = parts[2];

            if (!_coreRegistry.TryGetValue(coreName, out EmuSen.DianaOS.Commands.CoreDescriptor? descriptor))
            {
                Console.WriteLine($"core: unknown core '{coreName}'. Supported: {string.Join(", ", new SortedSet<string>(_coreRegistry.Keys, StringComparer.OrdinalIgnoreCase))}");
                return null;
            }

            if (!File.Exists(romPath))
            {
                Console.WriteLine($"core: ROM not found: {romPath}");
                return null;
            }

            string extension = Path.GetExtension(romPath);
            if (!descriptor.SupportsExtension(extension))
            {
                Console.WriteLine($"core: '{(extension.Length > 0 ? extension : "(no extension)")}' is not a supported ROM type for {descriptor.DisplayName} - expected: {string.Join(", ", descriptor.Extensions)}");
                return null;
            }

            return romPath;
        }
    }
}
