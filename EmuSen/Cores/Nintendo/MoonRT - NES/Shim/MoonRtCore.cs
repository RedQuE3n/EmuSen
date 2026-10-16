using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // MoonRT, the NES's core, behind ICore: the generic v1 adapter with the NES's refusals, state checks, battery rule, pad routing and its own debugger - see Moon_Native.md §9.4, EmuSen_CoreAPI.md §26.
    public sealed class MoonRtCore : PortEngine
    {
        public const int Width = 256, Height = 240;

        // "MOON" little-endian, and the versions this build writes and reads - see Moon_Core.md §5.
        public const uint StateMagic = 0x4E4F4F4D;
        public const int CurrentStateVersion = 6, OldestReadableVersion = 3;

        // The master clock over the clocks of a frame, 262 lines of 341 dots of 4.
        public const double FrameRate = 21477272.0 / (262 * 341 * 4);

        private uint _mutes;
        private StateLayout? _layout;
        private CoreMachine? _layoutOf;

        public MoonRtCore() : base(MoonNative.Engine, "MoonRT", "Moon", pads: 2)
        {
            Breakpoints.CallStack = CallStack;
            CallStack.EntryPointObserver = Coverage.RecordEntryPoint;
        }

        protected override IReadOnlyList<string> SpaceNames => MoonMachine.SpaceNames;
        protected override int CpuBusSpace => 7;

        public static bool Available => MoonNative.Engine.Available;

        public override string CoreName => "NES";
        public override int ScreenWidth => Width;
        public override int ScreenHeight => Height;
        public override double FrameRateHz => FrameRate;
        public override int StateVersion => CurrentStateVersion;

        public CallStackRegistry CallStack => MirrorCallStack;
        public CoverageRegistry Coverage => MirrorCoverage;
        public LabelRegistry Labels { get; } = new();

        // The loaded image's header, read before the library sees it.
        public NesHeader? Header { get; private set; }

        // The machine's state listing, read once per loaded game; the debugger reads registers through it.
        public StateLayout? Layout
        {
            get
            {
                if (!IsRomLoaded) return null;
                CoreMachine m = Machine.Core;
                if (!ReferenceEquals(m, _layoutOf)) (_layout, _layoutOf) = (new StateLayout(m.Layout()), m);
                return _layout;
            }
        }

        // The spaces the core logs a store under, as the debugger names them.
        private static readonly (uint Id, string Name)[] Reported = { (0, NesSystem.Ram), (2, NesSystem.PrgRam), (8, "PPUREG"), (9, "APUREG") };

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

        // A battery file only for a board MoonRT builds whose header claims one; a malformed header is left for the library to refuse.
        protected override BatterySave Prepare(string path, byte[] image)
        {
            try { Header = NesHeader.Read(image); }
            catch (InvalidDataException)
            {
                Header = null;
                return BatterySave.None;
            }
            return MoonBoards.IsImplemented(Header.Mapper) ? BatterySave.Open(path, BatterySave.Nes, hasRam: Header.HasBattery) : BatterySave.None;
        }

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

        // Ports 0 and 1 are pads 1 and 2 and any other port is ignored.
        protected override int PortFor(int port) => port is 0 or 1 ? port : -1;

        public int ControllerPorts => NesSystem.Ports;

        protected override void CheckState(byte[] state)
        {
            if (state.Length < 8) throw new EndOfStreamException("Unable to read beyond the end of the stream.");
            if (BitConverter.ToUInt32(state, 0) != StateMagic) throw new InvalidDataException("Not a Moon save state.");
            int version = BitConverter.ToInt32(state, 4);
            if (version is < OldestReadableVersion or > CurrentStateVersion) throw new InvalidDataException($"Save state version {version} is not one this build reads ({OldestReadableVersion} to {CurrentStateVersion}).");
        }

        protected override Exception? OwnRefusal(int status, ulong detail) => Own(status);

        protected override string? OwnWords(long status) => StatusWords(status);

        // The exception a status of MoonRT's band stands for - see Moon_Native.md §6.2, D4.
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

        public new MoonRtDebugTarget CreateDebugTarget()
        {
            Debug.Listen();
            return new MoonRtDebugTarget(this);
        }

        public bool IsChannelMuted(int index) => (uint)index < 5 && (_mutes & (1u << index)) != 0;

        public void SetChannelMuted(int index, bool muted)
        {
            if ((uint)index >= 5) return;
            _mutes = muted ? _mutes | (1u << index) : _mutes & ~(1u << index);
            SyncMutes();
        }

        protected override uint MuteMask() => _mutes;
    }
}
