using System;
using System.IO;
using System.Linq;
using EmuSen.Common;
using EmuSen.Cores;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Session;
using EmuSen.Crystal.Systems;
using EmuSen.Galaxia.Library;

namespace EmuSen.Crystal.Managed
{
    // Opening a session, written from EmulatorSession and CoreFactory: the engine chosen by the settings' layers, firmware asked first, the debugger's provider made over it - see EmuSen_Backend.md §14.
    internal sealed unsafe partial class ManagedRuntime
    {
        public ManagedSession Open(OpenParamsV1* p)
        {
            if (p == null) throw new CrystalException(SessionAbi.Null, "null open parameters");
            if (p->size < OpenParamsV1.Size10) throw CrystalException.BadStruct("emusen_crystal_open_params", p->size, OpenParamsV1.Size10);
            const uint known = CrystalNumbers.OpenNoBattery | CrystalNumbers.OpenSingleRows;
            if ((p->flags & ~known) != 0) throw new CrystalException(SessionAbi.NotSupported, $"open flags 0x{p->flags & ~known:X} are not known to 1.{CrystalNumbers.Minor}");
            // The presenter's device, read only from a 1.1 struct that holds it; each machine made is lent it afresh.
            var with = new OpenedWith(p->flags, p->audio_buffer_max_samples, p->size >= sizeof(OpenParamsV1) ? (nint)p->device : 0);
            return Open(ManagedCrystal.Text(p->path, p->path_len), ManagedCrystal.Text(p->console, p->console_len), ManagedCrystal.Text(p->engine, p->engine_len), with);
        }

        // A game or an empty console opened with the options a session's open params gave, as the session service's open_game reuses them.
        public ManagedSession Open(string? path, string? console, string? engine, OpenedWith with)
        {
            if (path is null == console is null) throw new CrystalException(SessionAbi.BadArgument, "a session opens a game or starts a console: one of the two is given");
            // A package's folder, or a file a package claims, opens as that package - see EmuSen_GamePackages.md §3.5.
            Galaxia.Media.PackageResolution? resolved = null;
            if (path is not null)
            {
                try { resolved = Galaxia.Media.GamePackages.Resolve(path, Galaxia.Media.PackagePlaces.Current()); }
                catch (IOException refused) { throw new CrystalException(CrystalNumbers.Io, refused.Message); }
                path = resolved.Game;
            }
            // A patched game's saves and states are named by its list's key, read once here as the package is - see EmuSen_GamePackages.md §5.5.
            string? saveKey = Galaxia.Media.GamePackage.SaveKeyAt(resolved?.Package);
            var options = new CoreOptions
            {
                PackageRoot = resolved?.Package,
                SaveKey = saveKey,
                BatteryRamDisabled = (with.Flags & CrystalNumbers.OpenNoBattery) != 0,
                AudioBufferMaxSamples = with.AudioBufferMaxSamples > 0 ? (int)Math.Min(with.AudioBufferMaxSamples, int.MaxValue) : CoreOptions.Default.AudioBufferMaxSamples,
            };
            var refusals = new LendRefusals();
            Func<IDeviceLend?>? lender = with.Device == 0 ? null : () => PlatformLend.Make(with.Device, refusals.Note);
            CoreBundle bundle = path is not null ? OpenGame(path, engine, options, lender) : StartConsole(console!, engine, options, lender, out console);
            string name = (bundle.Core as Cores.Native.CoreEngine)?.Console is { Length: > 0 } running ? running : console ?? CoreCatalog.ConsoleForRom(path!) ?? bundle.Core.CoreName;
            var session = new ManagedSession(this, NextId(), bundle, path, name, path is null ? console : null, with) { PackageRoot = resolved?.Package, SaveKey = saveKey };
            session.ApplyStored(key => GetSettingOf(session, key));
            Add(session);
            return session;
        }

        // A session over an engine already made, for a test's engine no factory makes.
        internal ManagedSession Attach(CoreBundle bundle, string gamePath, string console)
        {
            var session = new ManagedSession(this, NextId(), bundle, gamePath, console, null, default);
            Add(session);
            return session;
        }

        private static CoreBundle OpenGame(string path, string? engine, CoreOptions options, Func<IDeviceLend?>? lender)
        {
            if (!File.Exists(path)) throw new CrystalException(CrystalNumbers.Io, $"{path} does not exist");
            string? chosen = engine ?? EngineFor(path, options.PackageRoot);
            if (EmulatorSession.MissingFirmwareFor(path, chosen) is { Count: > 0 } missing)
                throw new CrystalException(CrystalNumbers.Firmware, $"{Path.GetFileName(path)} needs firmware the library does not hold: {string.Join("; ", missing.Select(m => m.Describe()))}");
            try { return CoreFactory.Load(path, headless: true, new CheatRegistry(), chosen, lender, options); }
            catch (Cores.Native.CoreNotApprovedException e) { throw new CrystalException(CrystalNumbers.NotApproved, e.Message); }
            catch (NotSupportedException e) { throw new CrystalException(CrystalNumbers.NoEngine, e.Message); }
            catch (IOException e) when (options.PackageRoot is not null) { throw new CrystalException(CrystalNumbers.Io, e.Message); }
        }

        private static CoreBundle StartConsole(string asked, string? engine, CoreOptions options, Func<IDeviceLend?>? lender, out string console)
        {
            console = ConsoleNamed(asked) ?? throw new CrystalException(CrystalNumbers.NoEngine, $"no console named \"{asked}\" starts with nothing in it in this build");
            string? chosen = engine ?? ConsoleEngine(console);
            try { return CoreFactory.LoadConsole(console, new CheatRegistry(), chosen, lender, options); }
            catch (Cores.Native.CoreNotApprovedException e) { throw new CrystalException(CrystalNumbers.NotApproved, e.Message); }
            catch (NotSupportedException e) { throw new CrystalException(CrystalNumbers.NoEngine, e.Message); }
        }

        // What a session was opened with besides its game: the open flags, the sound buffer and the presenter's device.
        public readonly record struct OpenedWith(uint Flags, uint AudioBufferMaxSamples, nint Device);

        // Why no device was lent, said once a session in the error log, where a frontend's own report used to say it.
        private sealed class LendRefusals
        {
            private bool _said;

            public void Note(string why)
            {
                if (_said) return;
                _said = true;
                ErrorLog.Warning("picture", why);
            }
        }

        // A console started empty, by its name or one of its systems' ids.
        private static string? ConsoleNamed(string asked) =>
            CoreCatalog.ConsolesThatStartEmpty.FirstOrDefault(c => string.Equals(c.Console, asked, StringComparison.OrdinalIgnoreCase) || string.Equals(c.SystemId, asked, StringComparison.OrdinalIgnoreCase))?.Console
            ?? CoreCatalog.EmptyStartFor(asked)?.Console;
    }
}
