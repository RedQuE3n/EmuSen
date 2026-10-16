using System;
using System.IO;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Galaxia.Library;
using EmuSen.Hotaru.Views;

namespace EmuSen.Hotaru
{
    // Everything a GameWindow needs, built the same way whichever prompt asked for the ROM - see EmuSen_Frontend_Driver.md §1.
    public sealed record HotaruSession(ICore Core, IDianaShCommand[] ExtraCommands, string StatePath, CategorizedLogWriter LogWriter)
    {
        // What the game's own prompt answers to `hotaru`, since its shell already has a game.
        public const string GameAlreadyRunning = "a game is already running in this shell - `core <name> <path>` swaps it";

        // Takes over Console.Out for the game's log, as Hotaru's Main always did - see EmuSen_Frontend_Driver.md §1.
        public static HotaruSession Build(string romPath, TextWriter originalOut)
        {
            // Separate from home/Saves' battery-backed cartridge SRAM - see EmuSen_Frontend_Driver.md §1.
            string statePath = SaveLibrary.StatePathFor(romPath);

            string? engine = CoreFactory.ConfiguredEngine(romPath);
            ICore core = CoreFactory.Create(romPath, headless: false, engine);

            // Needs core.CoreName, so it cannot be built any earlier - see EmuSen_Frontend_Driver.md §1 step 3.
            string logDir = Path.Combine(DianaShSandbox.LogsDirectory, core.CoreName, $"console_{DateTime.Now:yyyyMMdd_HHmmss}");
            Directory.CreateDirectory(logDir);
            var logWriter = new CategorizedLogWriter(originalOut, logDir);
            Console.SetOut(logWriter);

            CoreFactory.LoadInto(core, romPath);
            if (CoreFactory.EngineNotice(romPath, engine, core) is { } notice) Console.WriteLine($"[core] {notice}");

            // Built once here; GameWindow rebuilds the interpreter around them after a `core` swap - see EmuSen_Frontend_Driver.md §1.
            IDianaShCommand[] extraCommands =
            {
                new CoretopCommand(DebugWindows.ShowCoretopWindow),
                new StateCommand(core.SaveState, core.LoadState, () => statePath),
                new CoreCommand(CoreCatalog.Registry),
                new HotaruCommand(null, GameAlreadyRunning),
            };

            return new HotaruSession(core, extraCommands, statePath, logWriter);
        }

        public GameWindow OpenWindow() => new(Core, ExtraCommands, StatePath);
    }
}
