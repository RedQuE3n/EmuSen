using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Debug;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT behind the NES's ICore: the generic v1 adapter, with C# Moon's exceptions, state checks, battery rule and mirror debugger kept for its oracle - see Moon_Native.md §8.3, EmuSen_CoreAPI.md §26.
    public sealed class MoonRtCore : PortEngine
    {
        public MoonRtCore() : base(MoonNative.Engine, "MoonRT", "Moon", pads: 2) { }

        // The C# machine the debugger reads, refreshed from MoonRT's state; its registries are this core's - see Moon_Native.md §2.4.
        public MoonCore Mirror { get; } = new();

        protected override ICore MirrorCore => Mirror;
        protected override bool MirrorLoaded => Mirror.Bus is not null;
        protected override CheatRegistry MirrorCheats { get => Mirror.Cheats; set => Mirror.Cheats = value; }
        protected override CallStackRegistry MirrorCallStack => Mirror.CallStack;
        protected override CoverageRegistry MirrorCoverage => Mirror.Coverage;
        protected override IReadOnlyList<string> SpaceNames => MoonMachine.SpaceNames;
        protected override int CpuBusSpace => 7;

        public static bool Available => MoonNative.Engine.Available;

        public override string CoreName => "NES";
        public override int ScreenWidth => Moon.Video.Ppu.ScreenWidth;
        public override int ScreenHeight => Moon.Video.Ppu.ScreenHeight;
        public override double FrameRateHz => Mirror.FrameRateHz;
        public override int StateVersion => MoonCore.StateVersion;

        public override WatchRegistry Watches => Mirror.Watches;
        public override FrameLogRegistry FrameLog => Mirror.FrameLog;
        public override BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CallStackRegistry CallStack => Mirror.CallStack;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;

        // The spaces the core logs a store under, as C#'s write observer names them.
        private static readonly (uint Id, string Name)[] Reported = { (0, MoonCore.SpaceRam), (2, MoonCore.SpacePrgRam), (8, "PPUREG"), (9, "APUREG") };

        protected override string? ReportedName(uint id)
        {
            foreach (var (i, name) in Reported) if (i == id) return name;
            return null;
        }

        protected override uint? ReportedSpace(string name)
        {
            foreach (var (i, n) in Reported) if (string.Equals(n, name, StringComparison.OrdinalIgnoreCase)) return i;
            return null;
        }

        // MoonCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save opened as C# opens it.
        protected override BatterySave Prepare(string path, byte[] image)
        {
            Cartridge header = Cartridge.FromImage(image);
            return BatterySave.Open(path, BatterySave.Nes, hasRam: header.HasBattery && header.PrgRam.Length > 0);
        }

        protected override void Loaded(string path) => Mirror.LoadRom(path);

        // The RESET button - see Moon_Core.md §6.
        public void Reset()
        {
            if (!IsRomLoaded) throw new InvalidOperationException("Reset() called before LoadRom().");
            try { Machine.Core.Reset(); }
            catch (CoreRefusedException e) { throw Refusal(e.Status, 0); }
            ClearHalt();
        }

        protected override int ButtonBit(PadButton button) => button switch
        {
            PadButton.A => 0,
            PadButton.B => 1,
            PadButton.Select => 2,
            PadButton.Start => 3,
            PadButton.Up => 4,
            PadButton.Down => 5,
            PadButton.Left => 6,
            PadButton.Right => 7,
            _ => -1,
        };

        // Port 0 is pad 1 and every other port pad 2, as MoonCore.SetButton has it.
        protected override int PortFor(int port) => port == 0 ? 0 : 1;

        // MoonCore.LoadState's two refusals with its messages.
        protected override void CheckState(byte[] state)
        {
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != MoonCore.StateMagic) throw new InvalidDataException("Not a Moon save state.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < MoonCore.OldestReadableVersion or > MoonCore.StateVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({MoonCore.OldestReadableVersion} to {MoonCore.StateVersion}).");
        }

        protected override Exception? OwnRefusal(int status, ulong detail) => Own(status);

        protected override string? OwnWords(long status) => StatusWords(status);

        // The C# exception MoonCore.LoadRom or RunFrame would have thrown for a status of MoonRT's band - see Moon_Native.md §6.2, D4.
        internal static Exception? Own(int status) => status switch
        {
            -9 => new InvalidDataException("Not an iNES image: missing the \"NES\\x1A\" magic."),
            -10 => new NotSupportedException("The iNES mapper is not implemented - see Moon_Memory.md §4 for what is."),
            -11 => new InvalidDataException("The header claims more PRG than the file holds."),
            NativeInterface.FaultBase - 3 => new ArgumentOutOfRangeException("masterDelta", "A clock cannot run backwards."),
            _ => null,
        };

        internal static string? StatusWords(long status) => status switch
        {
            -9 => "not an iNES image",
            -10 => "a mapper no board implements",
            -11 => "a header claiming more PRG than the file holds",
            NativeInterface.FaultBase - 1 => "an index outside an array",
            NativeInterface.FaultBase - 2 => "a division by zero",
            NativeInterface.FaultBase - 3 => "a clock running backwards",
            _ => null,
        };

        // The debugger's view: the mirror loaded from MoonRT's state, its reads and writes sent to MoonRT - see Moon_Native.md §2.4.
        public new MoonDebugTarget CreateDebugTarget()
        {
            Debug.Listen();
            return new(Mirror, () => (LastFrameMilliseconds, 0.0), new MoonDebugHost(
                ReadSpace, WriteSpace, SyncMirror, ApplyCheats, () => TotalFrames, (_, _) => SyncMutes(), () => Pc));
        }

        protected override uint MuteMask()
        {
            uint mask = 0;
            for (int i = 0; i < 5; i++) if (Mirror.Apu!.IsChannelMuted(i)) mask |= 1u << i;
            return mask;
        }
    }
}
