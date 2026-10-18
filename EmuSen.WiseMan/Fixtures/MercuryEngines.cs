using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Fixtures
{
    // Which console a cartridge runs on, as the Model setting names it - see Mercury_Model.md §1.
    public enum GbModel
    {
        Auto,
        GameBoy,
        GameBoyColor,
    }

    // MercuryRT as the tests hold it: the generic engine over its v1 library, its own extensions for the rig, and the facts its machine info states - see EmuSen_Backend.md §17.10.
    public static unsafe class MercuryEngines
    {
        public const string SpaceRom = "ROM", SpaceVram = "VRAM", SpaceCartRam = "CARTRAM", SpaceWram = "WRAM", SpaceOam = "OAM", SpaceHram = "HRAM", SpaceCpuBus = "CPUBUS";
        public static readonly string[] SpaceNameList = { SpaceRom, SpaceVram, SpaceCartRam, SpaceWram, SpaceOam, SpaceHram, SpaceCpuBus, "BGPAL", "OBJPAL" };

        public const int CpuClockHz = 4194304, CyclesPerFrame = 70224, ScreenWidthPixels = 160, ScreenHeightPixels = 144;
        public const uint StateMagic = 0x4352454D;
        public const int StateFormatVersion = 11, OldestReadableVersion = 5;

        public const string ModelKey = "Model", ModelAuto = "Auto", ModelGameBoy = "Game Boy", ModelGameBoyColor = "Game Boy Color";

        // The illegal-opcode status, whose detail word is (opcode << 16) | pc.
        public const int IllegalOpcode = -20;

        public static readonly IReadOnlyList<PadButton> PadButtons = new[] { PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        public static string ModelName(GbModel model) => model switch
        {
            GbModel.GameBoy => ModelGameBoy,
            GbModel.GameBoyColor => ModelGameBoyColor,
            _ => ModelAuto,
        };

        // The folder the host files a game's battery save under: the system its file runs as, GBC for a .gbc file, GB for any other.
        public static string SaveFolder(string rom) => string.Equals(Path.GetExtension(rom), ".gbc", StringComparison.OrdinalIgnoreCase) ? BatterySave.GameBoyColor : BatterySave.GameBoy;

        // The library the build puts beside the tests, opened by its path rather than through discovery.
        public static string LibraryPath => Path.Combine(AppContext.BaseDirectory,
            OperatingSystem.IsWindows() ? "mercuryrt.dll" : OperatingSystem.IsMacOS() ? "libmercuryrt.dylib" : "libmercuryrt.so");

        private static readonly Lazy<CoreLibrary> Opened = new(() => CoreLibrary.Open(LibraryPath));

        public static CoreLibrary Library => Opened.Value;

        public static bool Available => Library.Available;

        public static string Report => Library.Report;

        // The engine a test runs a game on, on the console the model names.
        public static CoreEngine Create(CoreOptions? options = null, GbModel model = GbModel.Auto) =>
            new(Library, new Dictionary<string, string> { [ModelKey] = ModelName(model) }) { Options = options ?? CoreOptions.Default };

        // One of MercuryRT's own extensions on the engine's machine, under its gate.
        public static T Call<T>(CoreEngine core, string export, Func<nint, nint, T> call)
        {
            nint f = Library.Export(export);
            if (f == 0) throw new NotSupportedException($"{Path.GetFileName(LibraryPath)} has no {export}.");
            CoreMachine m = core.Machine;
            lock (m.Gate) return call(f, m.Handle);
        }

        // One instruction with the frame loop's acknowledge and stall: its T-cycles, or IllegalOpcode.
        public static int Step(CoreEngine core) => Call(core, "mercuryrt_core_step", (f, h) => ((delegate* unmanaged<nint, int>)f)(h));

        public static void Tick(CoreEngine core, int cycles) => Call(core, "mercuryrt_core_tick", (f, h) => ((delegate* unmanaged<nint, int, int>)f)(h, cycles));

        public static void Stop(CoreEngine core) => Call(core, "mercuryrt_core_stop", (f, h) => ((delegate* unmanaged<nint, int>)f)(h));

        // The bytes the ROM clocked out of the link port, oldest first.
        public static byte[] Serial(CoreEngine core) => Call(core, "mercuryrt_core_serial", (f, h) =>
        {
            var serial = (delegate* unmanaged<nint, byte*, nuint, long>)f;
            long n = serial(h, null, 0);
            var log = new byte[n];
            fixed (byte* data = log) serial(h, data, (nuint)log.Length);
            return log;
        });

        // The byte a CPU read of address returns for an original byte under the current patches, or -1 for none.
        public static int RomPatch(CoreEngine core, int address, byte original) => Call(core, "mercuryrt_core_rom_patch", (f, h) => ((delegate* unmanaged<nint, uint, uint, int>)f)(h, (uint)address, original));

        // The machine's state now and the listing that names its bytes.
        public static (byte[] State, StateLayout Layout) Snapshot(CoreEngine core)
        {
            CoreMachine m = core.Machine;
            var state = new byte[m.StateSize()];
            m.Save(state);
            return (state, StateLayout.FromText("MercuryRT", m.Layout()));
        }

        // An illegal opcode as the frame refuses it: its status and detail word.
        public static bool IsIllegalOpcode(Exception e) => e is CoreRefusedException { Status: IllegalOpcode };
    }
}
