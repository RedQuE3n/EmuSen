using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Session;
using EmuSen.Crystal.Systems;

namespace EmuSen.Crystal.Managed
{
    // Opening a session, written from EmulatorSession and CoreFactory: the engine chosen by the settings' layers, firmware asked first, the debugger's provider made over it - see EmuSen_Backend.md §14.
    internal sealed unsafe partial class ManagedRuntime
    {
        public ManagedSession Open(OpenParamsV1* p)
        {
            if (p == null) throw new CrystalException(SessionAbi.Null, "null open parameters");
            if (p->size < sizeof(OpenParamsV1)) throw CrystalException.BadStruct("emusen_crystal_open_params", p->size, sizeof(OpenParamsV1));
            const uint known = CrystalNumbers.OpenNoBattery | CrystalNumbers.OpenSingleRows;
            if ((p->flags & ~known) != 0) throw new CrystalException(SessionAbi.NotSupported, $"open flags 0x{p->flags & ~known:X} are not known to 1.{CrystalNumbers.Minor}");
            string? path = ManagedCrystal.Text(p->path, p->path_len);
            string? console = ManagedCrystal.Text(p->console, p->console_len);
            string? engine = ManagedCrystal.Text(p->engine, p->engine_len);
            if (path is null == console is null) throw new CrystalException(SessionAbi.BadArgument, "a session opens a game or starts a console: one of the two is given");
            var options = new CoreOptions
            {
                BatteryRamDisabled = (p->flags & CrystalNumbers.OpenNoBattery) != 0,
                AudioBufferMaxSamples = p->audio_buffer_max_samples > 0 ? (int)Math.Min(p->audio_buffer_max_samples, int.MaxValue) : CoreOptions.Default.AudioBufferMaxSamples,
            };
            CoreBundle bundle = path is not null ? OpenGame(path, engine, options) : StartConsole(console!, engine, options, out console);
            string name = (bundle.Core as Cores.Native.CoreEngine)?.Console is { Length: > 0 } running ? running : console ?? CoreCatalog.ConsoleForRom(path!) ?? bundle.Core.CoreName;
            var session = new ManagedSession(this, NextId(), bundle, path, name, path is null ? console : null, p->flags);
            session.ApplyStored(key => GetSettingOf(session, key));
            Add(session);
            return session;
        }

        // A session over an engine already made, for a test's engine no factory makes.
        internal ManagedSession Attach(CoreBundle bundle, string gamePath, string console)
        {
            var session = new ManagedSession(this, NextId(), bundle, gamePath, console, null, 0);
            Add(session);
            return session;
        }

        private static CoreBundle OpenGame(string path, string? engine, CoreOptions options)
        {
            if (!File.Exists(path)) throw new CrystalException(CrystalNumbers.Io, $"{path} does not exist");
            string? chosen = engine ?? EngineFor(path);
            if (EmulatorSession.MissingFirmwareFor(path, chosen) is { Count: > 0 } missing)
                throw new CrystalException(CrystalNumbers.Firmware, $"{Path.GetFileName(path)} needs firmware the library does not hold: {string.Join("; ", missing.Select(m => m.Describe()))}");
            try { return CoreFactory.Load(path, headless: true, new CheatRegistry(), chosen, lender: null, options); }
            catch (NotSupportedException e) { throw new CrystalException(CrystalNumbers.NoEngine, e.Message); }
        }

        private static CoreBundle StartConsole(string asked, string? engine, CoreOptions options, out string console)
        {
            console = ConsoleNamed(asked) ?? throw new CrystalException(CrystalNumbers.NoEngine, $"no console named \"{asked}\" starts with nothing in it in this build");
            string? chosen = engine ?? ConsoleEngine(console);
            try { return CoreFactory.LoadConsole(console, new CheatRegistry(), chosen, lender: null, options); }
            catch (NotSupportedException e) { throw new CrystalException(CrystalNumbers.NoEngine, e.Message); }
        }

        // A console started empty, by its name or one of its systems' ids.
        private static string? ConsoleNamed(string asked) =>
            CoreCatalog.ConsolesThatStartEmpty.FirstOrDefault(c => string.Equals(c.Console, asked, StringComparison.OrdinalIgnoreCase) || string.Equals(c.SystemId, asked, StringComparison.OrdinalIgnoreCase))?.Console
            ?? CoreCatalog.EmptyStartFor(asked)?.Console;
    }
}
