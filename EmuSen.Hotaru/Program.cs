using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using Avalonia;
using Avalonia.Controls;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Venus;
using EmuSen.Debug;
using EmuSen.DianaOS;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using EmuSen.Hotaru.Views;
using EmuSen.LunaP;

namespace EmuSen.Hotaru
{
    class Program
    {
        // Backs both `core <name> <path>` paths off one shared registry - see EmuSen_Multicore.md §3.
        private static IReadOnlyDictionary<string, EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.CoreDescriptor> _coreRegistry
            => EmuSen.Cores.CoreCatalog.Registry;

        static void Main(string[] args)
        {
            // A config file that won't parse falls back to defaults either way;
            // this is what stops it doing so silently - see §6.2.
            EmuSen.Galaxia.ConfigDiagnostics.Sink = m => Console.WriteLine("[config] " + m);

            // Before any window exists, since GraphicsSettings decides its size.
            EmuSen.Audio.AudioSettings.LoadFromDisk();
            EmuSen.Graphics.GraphicsSettings.LoadFromDisk();

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
            ICore? core = null;

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
                (core as ITraceFlushable)?.FlushVerboseTrace();

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

            string? initialRomPath = LaunchMode.RomPathFrom(args);

            // A launch with no terminal to type into gets the shell as a window - see EmuSen_Frontend_Driver.md §3c.
            if (LaunchMode.Decide(args, LaunchMode.InteractiveTerminal(), LaunchMode.DisplayAvailable()) == ShellMode.Window)
            {
                try
                {
                    RunWindowedShell(args, initialRomPath, originalOut, c => core = c, w => logWriter = w);
                }
                finally
                {
                    FlushAndDispose();
                }
                return;
            }

            // ROM path resolution - see EmuSen_Frontend_Driver.md §1.
            // DianaOS is the ONLY thing shown at launch, whether or not a
            // ROM path was given on the command line - RunStandaloneShell
            // runs as plain console I/O, entirely before Avalonia (or any
            // window at all) ever starts. Returns null only if the user
            // actually asked to shut down (or hit EOF) with no ROM ever
            // resolved, in which case there's nothing left to do.
            string? romPathFromShell = RunStandaloneShell(initialRomPath);
            if (romPathFromShell is null) return;
            string romPath = romPathFromShell;

            try
            {
                Session session = BuildSession(romPath, originalOut);
                core = session.Core;
                logWriter = session.LogWriter;

                // Blocks until the window closes; GameWindow's own Closing handler does the teardown - see §1.
                BuildAvaloniaApp(() => new GameWindow(session.Core, session.ExtraCommands, session.StatePath))
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

        // Everything a GameWindow needs, built the same way whichever shell resolved the ROM - see §3c.
        private sealed record Session(ICore Core, IDianaOSCommand[] ExtraCommands, string StatePath, CategorizedLogWriter LogWriter);

        private static Session BuildSession(string romPath, TextWriter originalOut)
        {
            // Separate from Usr/Home/Saves' battery-backed cartridge SRAM - see §1.
            string statePath = Path.Combine(DianaOSSandbox.SaveStatesDirectory, Path.GetFileNameWithoutExtension(romPath) + ".state");

            ICore core = CoreFactory.Create(romPath, headless: false);

            // Needs core.CoreName, so it cannot be built any earlier - see §1 step 3.
            string logDir = Path.Combine(DianaOSSandbox.LogsDirectory, core.CoreName, $"console_{DateTime.Now:yyyyMMdd_HHmmss}");
            Directory.CreateDirectory(logDir);
            var logWriter = new CategorizedLogWriter(originalOut, logDir);
            Console.SetOut(logWriter);

            core.LoadRom(romPath);
            DebugSettings.CpuVerboseLogging = true;
            DebugSettings.Spc700VerboseLogging = true;

            // Built once here; GameWindow rebuilds the interpreter around them after a `core` swap - see §1.
            IDianaOSCommand[] extraCommands =
            {
                new EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.CoretopCommand(DebugWindows.ShowCoretopWindow),
                new EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.StateCommand(core.SaveState, core.LoadState, () => statePath),
                new EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.CoreCommand(_coreRegistry),
            };

            return new Session(core, extraCommands, statePath, logWriter);
        }

        // The windowed counterpart to RunStandaloneShell - see EmuSen_Frontend_Driver.md §3c.
        private static void RunWindowedShell(
            string[] args, string? initialRomPath, TextWriter originalOut,
            Action<ICore> onCore, Action<CategorizedLogWriter> onLogWriter)
        {
            DianaOSShellWindow? shell = null;

            Window LaunchGame(string romPath)
            {
                Session session = BuildSession(romPath, originalOut);
                onCore(session.Core);
                onLogWriter(session.LogWriter);

                var game = new GameWindow(session.Core, session.ExtraCommands, session.StatePath);
                shell!.AttachLiveShell(game);
                // Closing the game ends the session, exactly as it does for a console launch.
                game.Closing += (_, _) => shell!.Close();
                return game;
            }

            Window BuildShell()
            {
                shell = new DianaOSShellWindow(
                    LaunchGame,
                    line => TryResolveCoreCommand(line, m => shell!.AppendLine(m)),
                    _coreRegistry.Values.Select(d => d.DisplayName).Distinct());

                if (initialRomPath != null) StartInitialRom(shell, initialRomPath, LaunchGame);
                return shell;
            }

            BuildAvaloniaApp(BuildShell).StartWithClassicDesktopLifetime(args);
        }

        // A CLI ROM path is a shortcut for typing `core`, in this shell as much as the console one - see §3.
        private static void StartInitialRom(DianaOSShellWindow shell, string initialRomPath, Func<string, Window> launchGame)
        {
            if (!File.Exists(initialRomPath))
            {
                shell.AppendLine($"[ERROR] ROM not found: {initialRomPath}");
                return;
            }

            shell.AppendLine($"[ROM] Loading: {initialRomPath}");
            shell.Opened += (_, _) =>
            {
                try { launchGame(initialRomPath).Show(); }
                catch (Exception ex) { shell.ReportLaunchFailure($"[CPU HALT] {ex.Message}"); }
            };
        }

        // The platform/font/X11 sequence this used to spell out lives in LunaApp now - see EmuSen_LunaP.md §3.
        private static AppBuilder BuildAvaloniaApp(Func<Window> mainWindow) =>
            LunaApp.Configure(() => new App(mainWindow));

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
            DianaOSInterpreter shell = DianaOSInterpreter.CreateDefault(null,
                supportedCheatSystems: () => EmuSen.Cores.CoreCatalog.SupportedCheatSystems);
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
                // (EmuSen.DianaOS.DianaOS.Bin.Commands.Unix.ShutdownCommand, already in the
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
        // `report` is where the refusals go: the terminal for a console launch, the shell window for a windowed one.
        internal static string? TryResolveCoreCommand(string trimmedLine, Action<string>? report = null)
        {
            report ??= Console.WriteLine;

            string[] parts = trimmedLine.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 3)
            {
                report("Usage: core <corename> <path-to-rom>");
                return null;
            }

            string coreName = parts[1];
            string romPath = parts[2];

            if (!_coreRegistry.TryGetValue(coreName, out EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.CoreDescriptor? descriptor))
            {
                report($"core: unknown core '{coreName}'. Supported: {string.Join(", ", new SortedSet<string>(_coreRegistry.Keys, StringComparer.OrdinalIgnoreCase))}");
                return null;
            }

            if (!File.Exists(romPath))
            {
                report($"core: ROM not found: {romPath}");
                return null;
            }

            string extension = Path.GetExtension(romPath);
            if (!descriptor.SupportsExtension(extension))
            {
                report($"core: '{(extension.Length > 0 ? extension : "(no extension)")}' is not a supported ROM type for {descriptor.DisplayName} - expected: {string.Join(", ", descriptor.Extensions)}");
                return null;
            }

            return romPath;
        }
    }
}
