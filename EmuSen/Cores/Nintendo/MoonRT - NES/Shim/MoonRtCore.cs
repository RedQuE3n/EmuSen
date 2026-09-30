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
    // MoonRT behind the NES's ICore, on the common native host: the machine in Rust, the registries, saves and cheats' rules in C# - see Moon_Native.md §8.3.
    public sealed class MoonRtCore : NativeRtCore<MoonMachine>, ICore, IFrameProfiler, ICheatRegistryHost, IStateFormat, IFrameBufferPool, IEngineFeatures, IDisposable
    {
        private readonly NativeDebugBridge _debug;

        // The debugger's bridge over the mirror's registries, its pushes stamped with the frame being drained - see Moon_Native.md §8.4.
        public MoonRtCore() : base(MoonMachine.FrameBytes, ports: 2)
        {
            _debug = new NativeDebugBridge(() => _machine, Mirror.Breakpoints, Mirror.Watches, Mirror.CallStack,
                new[] { Mirror.Coverage }, new[] { 0x10000 / 8 }, ReportedName, ReportedSpace);
            Mirror.CallStack.FrameNumberProvider = () => _debug.EventFrame;
            DebugBridge = _debug;
        }

        // The spaces the core logs a store under, as C#'s write observer names them.
        private static readonly (uint Id, string Name)[] Reported = { (0, MoonCore.SpaceRam), (2, MoonCore.SpacePrgRam), (8, "PPUREG"), (9, "APUREG") };

        private static string? ReportedName(uint id)
        {
            foreach (var (i, name) in Reported) if (i == id) return name;
            return null;
        }

        private static uint? ReportedSpace(string name)
        {
            foreach (var (i, n) in Reported) if (string.Equals(n, name, StringComparison.OrdinalIgnoreCase)) return i;
            return null;
        }

        public CallStackRegistry CallStack => Mirror.CallStack;

        public NativeDebugBridge Debug => _debug;

        // The instruction the processor is about to run, live, as the registry compares it.
        public int Pc => _debug.ProgramCounter();

        // The C# machine the debugger reads, refreshed from MoonRT's state; its registries are this core's - see Moon_Native.md §2.4.
        public MoonCore Mirror { get; } = new();

        protected override ICore MirrorCore => Mirror;
        protected override bool MirrorLoaded => Mirror.Bus is not null;
        protected override CheatRegistry MirrorCheats { get => Mirror.Cheats; set => Mirror.Cheats = value; }
        protected override IReadOnlyList<string> SpaceNames => MoonMachine.SpaceNames;
        protected override int CpuBusSpace => 7;
        protected override long PatchLow => 0x4020;
        protected override long PatchHigh => 0xFFFF;

        public static bool Available => MoonMachine.Complete;

        public override string CoreName => "NES";
        public override int ScreenWidth => Moon.Video.Ppu.ScreenWidth;
        public override int ScreenHeight => Moon.Video.Ppu.ScreenHeight;
        public override double FrameRateHz => Mirror.FrameRateHz;
        public override IReadOnlyList<PadButton> SupportedButtons => MoonCore.PadButtons;
        public override int StateVersion => MoonCore.StateVersion;

        public WatchRegistry Watches => Mirror.Watches;
        public override FrameLogRegistry FrameLog => Mirror.FrameLog;
        public override BreakpointRegistry Breakpoints => Mirror.Breakpoints;
        public CoverageRegistry Coverage => Mirror.Coverage;
        public LabelRegistry Labels => Mirror.Labels;

        // MoonCore.LoadRom: the header parsed by the C# Cartridge so its exceptions are C#'s own, the battery save opened as C# opens it.
        protected override (BatterySave Battery, string Settings) Prepare(string path, byte[] image)
        {
            Cartridge header = Cartridge.FromImage(image);
            return (BatterySave.Open(path, BatterySave.Nes, hasRam: header.HasBattery && header.PrgRam.Length > 0), "");
        }

        protected override MoonMachine CreateMachine(byte[] image, string settings, IReadOnlyList<(uint Which, byte[] Data)> files) =>
            new(image, files.Count > 0 ? files[0].Data : null);

        protected override void Loaded(string path) => Mirror.LoadRom(path);

        // The RESET button - see Moon_Core.md §6.
        public void Reset()
        {
            if (_machine is null) throw new InvalidOperationException("Reset() called before LoadRom().");
            _machine.Reset();
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

        // The debugger's view: the mirror loaded from MoonRT's state, its reads and writes sent to MoonRT - see Moon_Native.md §2.4.
        public MoonDebugTarget CreateDebugTarget()
        {
            _debug.Listen();
            return new(Mirror, () => (LastFrameMilliseconds, 0.0), new MoonDebugHost(
                ReadSpace, WriteSpace, SyncMirror, ApplyCheats, () => TotalFrames, (_, _) => SyncMutes(), () => Pc));
        }

        // A store to the CPU's bus while a debugger listens is reported, as C#'s bus reports it.
        public override void WriteSpace(string spaceName, int address, byte value)
        {
            if (_machine is not null && spaceName == MoonCore.SpaceCpuBus && _debug.Listening)
            {
                _debug.Observed(() => base.WriteSpace(spaceName, address, value));
                return;
            }
            base.WriteSpace(spaceName, address, value);
        }

        protected override uint MuteMask()
        {
            uint mask = 0;
            for (int i = 0; i < 5; i++) if (Mirror.Apu!.IsChannelMuted(i)) mask |= 1u << i;
            return mask;
        }
    }
}
