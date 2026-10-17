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
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Systems;
using EmuSen.DianaSh.DianaSh.Dev;
using EmuSen.Hotaru;
using EmuSen.Shell.Views;
using EmuSen.LunaP;

namespace EmuSen.Shell
{
    // emusen-shell: DianaSh with EmuSen's commands, in the terminal or its own window - see EmuSen_Frontend_Driver.md §0.
    class Program
    {
        // Backs both `core <name> <path>` paths off one shared registry - see EmuSen_Multicore.md §3.
        private static IReadOnlyDictionary<string, CoreDescriptor> _coreRegistry => CoreCatalog.Registry;

        static void Main(string[] args)
        {
            // Before Avalonia's GLX screen exists, which reads it - see EmuSen_DevicePicture.md §18.
            EmuSen.Serenity.DevicePictureEnvironment.Prepare();

            // A config file that won't parse falls back to defaults either way; this stops it doing so silently - see §6.2.
            EmuSen.Galaxia.ConfigDiagnostics.Sink = m =>
            {
                Console.WriteLine("[config] " + m);
                EmuSen.Galaxia.Library.ErrorLog.Warning("config", m);
            };

            // LunaP keeps windows.json and luna.json where Galaxia keeps everything else, and reports through the same sink - see EmuSen_LunaP.md §19.2.
            EmuSen.LunaP.Settings.LunaSettings.Store = new EmuSen.LunaP.Settings.JsonSettingsStore(EmuSen.Galaxia.ConfigStore.Directory);
            EmuSen.LunaP.Settings.LunaSettings.Diagnostics = EmuSen.Galaxia.ConfigDiagnostics.Report;

            // Before any window exists, since GraphicsSettings decides its size.
            EmuSen.Audio.AudioSettings.LoadFromDisk();
            EmuSen.Graphics.GraphicsSettings.LoadFromDisk();

            TextWriter originalOut = Console.Out;

            // Disposed by FlushAndDispose on every exit path, or its last buffered lines are lost - see EmuSen_Frontend_Driver.md §0.2.
            CategorizedLogWriter? logWriter = null;

            // FlushAndDispose flushes its collapsed trace before the log writer closes - see EmuSen_Frontend_Driver.md §0.2.
            ICore? core = null;

            // The shutdown guard: several exit paths reach FlushAndDispose, and only the first may run it - see EmuSen_Frontend_Driver.md §0.2.
            int shutdownGuard = 0;

            void FlushAndDispose()
            {
                if (Interlocked.Exchange(ref shutdownGuard, 1) != 0) return;

                Console.SetOut(originalOut);
                logWriter?.Dispose();
            }

            // Every exit path that does not return through Main: SIGTERM, Ctrl+C, a fault on another thread - see EmuSen_Frontend_Driver.md §0.2.
            AppDomain.CurrentDomain.ProcessExit += (_, _) => FlushAndDispose();
            AppDomain.CurrentDomain.UnhandledException += (_, e) => EmuSen.Galaxia.Library.ErrorLog.Error("unhandled", (e.ExceptionObject as Exception)?.Message ?? "unhandled fault", e.ExceptionObject as Exception);
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

            // The shell is all a launch shows until a ROM resolves; null means it ended with none - see EmuSen_Frontend_Driver.md §3.
            HotaruLaunch? launch = RunStandaloneShell(initialRomPath);
            if (launch is null) return;

            try
            {
                HotaruSession session = HotaruSession.Build(launch, originalOut);
                core = session.Core;
                logWriter = session.LogWriter;

                // Blocks until the window closes; GameWindow's own Closing handler does the teardown - see EmuSen_Frontend_Driver.md §1.
                BuildAvaloniaApp(session.OpenWindow).StartWithClassicDesktopLifetime(args);
            }
            catch (Exception ex)
            {
                Console.WriteLine($"\n[CPU HALT] {ex.Message}");
            }
            finally
            {
                // The path a normal close and a CPU halt both take; the handlers above cover the rest - see EmuSen_Frontend_Driver.md §0.2.
                FlushAndDispose();
            }
        }

        // The windowed counterpart to RunStandaloneShell - see EmuSen_Frontend_Driver.md §3c.
        private static void RunWindowedShell(
            string[] args, string? initialRomPath, TextWriter originalOut,
            Action<ICore> onCore, Action<CategorizedLogWriter> onLogWriter)
        {
            DianaShShellWindow? shell = null;

            Window LaunchGame(HotaruLaunch launch)
            {
                HotaruSession session = HotaruSession.Build(launch, originalOut);
                onCore(session.Core);
                onLogWriter(session.LogWriter);

                var game = session.OpenWindow();
                shell!.AttachLiveShell(game);
                // Closing the game ends the session, exactly as it does for a console launch.
                game.Closing += (_, _) => shell!.Close();
                return game;
            }

            Window BuildShell()
            {
                shell = new DianaShShellWindow(
                    LaunchGame,
                    line => TryResolveCoreCommand(line, m => shell!.AppendLine(m)),
                    _coreRegistry.Values.Select(d => d.DisplayName).Distinct());

                if (initialRomPath != null) StartInitialRom(shell, initialRomPath, LaunchGame);
                return shell;
            }

            BuildAvaloniaApp(BuildShell).StartWithClassicDesktopLifetime(args);
        }

        // A ROM on the command line is `hotaru <rom>` typed for the player, in this shell as in the terminal's - see `man hotaru`.
        private static void StartInitialRom(DianaShShellWindow shell, string initialRomPath, Func<HotaruLaunch, Window> launchGame)
        {
            if (HotaruCommand.Refusal(initialRomPath, sandboxed: false, out _) is { } refused)
            {
                shell.AppendLine(refused.Output);
                return;
            }

            shell.AppendLine($"[ROM] Loading: {initialRomPath}");
            shell.Opened += (_, _) =>
            {
                try { launchGame(HotaruLaunch.Rom(initialRomPath)).Show(); }
                catch (Exception ex) { shell.ReportLaunchFailure($"[CPU HALT] {ex.Message}"); }
            };
        }

        // The platform/font/X11 sequence lives in LunaApp - see EmuSen_LunaP.md §3.
        private static AppBuilder BuildAvaloniaApp(Func<Window> mainWindow) =>
            LunaApp.Configure(() => new App(mainWindow));

        // The terminal's launch prompt: DianaSh with no target until `hotaru` or `core` names a ROM - see EmuSen_Frontend_Driver.md §3.
        private static HotaruLaunch? RunStandaloneShell(string? initialRomPath)
        {
            string? requested = null;
            DianaShInterpreter shell = DianaShInterpreter.CreateDefault(null,
                new IDianaShCommand[] { new HotaruCommand((rom, _) => requested = rom) },
                supportedCheatSystems: () => CoreCatalog.SupportedCheatSystems);
            Console.WriteLine(shell.GetWelcomeBanner(_coreRegistry.Values.Select(d => d.DisplayName).Distinct()));
            Console.WriteLine("--- DianaSh (type 'help', 'hotaru <rom>' or 'core <name> <path>' to launch a game, 'shutdown' to quit) ---");

            if (initialRomPath != null)
            {
                if (HotaruCommand.Refusal(initialRomPath, sandboxed: false, out _) is null)
                {
                    Console.WriteLine($"[ROM] Loading: {initialRomPath}");
                    return HotaruLaunch.Rom(initialRomPath);
                }
                Console.WriteLine($"[ERROR] ROM not found: {initialRomPath}");
            }

            while (true)
            {
                Console.Write(shell.IsAwaitingMoreInput ? "> " : "DianaSh #: ");
                string? line = ConsoleLineReader.ReadLine(shell.History.Entries);
                if (line is null) return null;
                string trimmed = line.Trim();

                if (!shell.IsAwaitingMoreInput && trimmed.StartsWith("core ", StringComparison.OrdinalIgnoreCase))
                {
                    HotaruLaunch? validated = TryResolveCoreCommand(trimmed);
                    if (validated != null) return validated;
                    continue; // TryResolveCoreCommand already printed why it failed
                }

                // `shutdown` is an ordinary command whose host action ends this prompt; `hotaru` hands its ROM back the same way.
                (_, string output, HostAction? action) = shell.Submit(line);
                if (output.Length > 0) Console.WriteLine(output);
                if (action is HostAction.Shutdown) return null;
                if (requested != null) return HotaruLaunch.Rom(requested);
            }
        }

        // The launch prompt's `core <name> <path>`, refused and reported, or the ROM to launch - see EmuSen_Frontend_Driver.md §3.
        internal static HotaruLaunch? TryResolveCoreCommand(string trimmedLine, Action<string>? report = null)
        {
            report ??= Console.WriteLine;

            string[] parts = trimmedLine.Split(' ', StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length != 3)
            {
                report("Usage: core <corename> <path-to-rom> | core <corename> --console");
                return null;
            }

            string coreName = parts[1];
            string romPath = parts[2];
            if (romPath == CoreCommand.ConsoleFlag)
            {
                if (CoreCommand.ConsoleRefusal(_coreRegistry, HotaruSession.StartsEmpty, coreName, out string console) is { } no)
                {
                    report(no.Output);
                    return null;
                }
                return HotaruLaunch.Empty(console);
            }

            if (CoreCommand.Refusal(_coreRegistry, coreName, romPath, out string resolved) is { } refused)
            {
                report(refused.Output);
                return null;
            }

            return HotaruLaunch.Rom(resolved);
        }
    }
}
