using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Native
{
    // A port's library through the core ABI v1 at its fixed name beside the assemblies, honouring its switch variable as the pre-stable loader did - see EmuSen_CoreAPI.md §26.
    public sealed class PortLibrary
    {
        private readonly Lazy<(CoreLibrary? Library, string Report)> _loaded;
        private readonly Lazy<nint> _romPatch;

        // <requiredCapabilities> and <requiredExports> are what the port's shim calls; a library without them is refused, its engine falling back.
        public PortLibrary(string crate, string variable, ulong requiredCapabilities, params string[] requiredExports)
        {
            Crate = crate;
            Variable = variable;
            _loaded = new(() => Load(requiredCapabilities, requiredExports));
            _romPatch = new(() => Export($"{crate}_core_rom_patch"));
        }

        public string Crate { get; }
        public string Variable { get; }

        public string FileName =>
            OperatingSystem.IsWindows() ? $"{Crate}.dll" : OperatingSystem.IsMacOS() ? $"lib{Crate}.dylib" : $"lib{Crate}.so";

        public CoreLibrary? Library => _loaded.Value.Library;
        public bool Available => Library is not null;

        // Why the library is or is not in use, in the pre-stable loader's words where it had them.
        public string Report => _loaded.Value.Report;

        public nint Export(string name) => Library?.Export(name) ?? 0;

        // The port's patch-table extension, the exhaustive patch tests' view of what the core holds.
        public nint RomPatchExport => _romPatch.Value;

        private (CoreLibrary?, string) Load(ulong requiredCapabilities, string[] requiredExports)
        {
            if (Environment.GetEnvironmentVariable(Variable) == "0") return (null, $"turned off by {Variable}=0");
            string path = Path.Combine(AppContext.BaseDirectory, FileName);
            if (!File.Exists(path)) return (null, $"{FileName} not found beside the assemblies");
            var library = CoreLibrary.Open(path);
            if (!library.Available) return (null, library.Report);
            ulong missing = requiredCapabilities & ~library.Capabilities;
            if (missing != 0)
                return (null, $"{FileName} lacks the capabilities {string.Join(", ", CoreInterface.CapabilityNames.Where(c => (missing & c.Bit) != 0).Select(c => c.Name))} its engine needs");
            foreach (string name in requiredExports)
                if (library.Export(name) == 0) return (null, $"{FileName} lacks the export {name} its engine needs");
            return (library, library.Report);
        }
    }

    // A port's machine as its shim's tests read it: the state, the picture and battery copies, and the patch table through the port's extension.
    public sealed unsafe class PortMachine(CoreMachine machine, PortLibrary port)
    {
        public CoreMachine Core => machine;

        public int StateSize => machine.StateSize();

        public byte[] Save()
        {
            var state = new byte[StateSize];
            machine.Save(state);
            return state;
        }

        public void Save(byte[] into) => machine.Save(into);

        public long CopyFrame(byte[] into) => machine.CopyFrame(into);

        public (byte[] Data, uint Flags) Battery(uint which) => machine.Battery(which);

        // The byte a CPU read of address returns for an original byte under the current patches, or -1 for none.
        public int RomPatch(int address, byte original)
        {
            nint export = port.RomPatchExport;
            if (export == 0) throw new NotSupportedException($"{port.FileName} has no {port.Crate}_core_rom_patch.");
            return ((delegate* unmanaged<nint, uint, uint, int>)export)(machine.Handle, (uint)address, original);
        }
    }

    // A port's shim over the generic v1 adapter: its oracle's exception types, state pre-checks, battery rule and pad routing, and its mirror debugger - see EmuSen_CoreAPI.md §13.1, §26.
    public abstract class PortEngine : CoreEngine
    {
        private readonly PortLibrary _port;
        private readonly string _engine;
        private readonly string _stateName;
        private readonly uint[] _pads;
        private readonly NativeDebugBridge _bridge;
        private BatterySave _battery = BatterySave.None;
        private PortMachine? _view;
        private bool _halted;
        private int _haltedAt;

        // <engine> names the library in messages ("MoonRT") and <stateName> its oracle's state ("Moon"); the mirror's registries are read here, so a subclass initialises its mirror where it declares it.
        protected PortEngine(PortLibrary port, string engine, string stateName, int pads)
            : base(port.Library ?? throw new InvalidOperationException($"{engine} is not in use: {port.Report}"))
        {
            _port = port;
            _engine = engine;
            _stateName = stateName;
            _pads = new uint[pads];
            _bridge = new NativeDebugBridge(() => IsRomLoaded ? base.Machine : null, Breakpoints, Watches, MirrorCallStack,
                new[] { MirrorCoverage }, new[] { 0x10000 / 8 }, ReportedName, ReportedSpace, Refusal);
            MirrorCallStack.FrameNumberProvider = () => _bridge.EventFrame;
            base.Cheats = MirrorCheats;
        }

        // The C# core the debugger reads, refreshed from the machine's state; its registries are this engine's.
        protected abstract ICore MirrorCore { get; }
        protected abstract bool MirrorLoaded { get; }
        protected abstract CheatRegistry MirrorCheats { get; set; }
        protected abstract CallStackRegistry MirrorCallStack { get; }
        protected abstract CoverageRegistry MirrorCoverage { get; }

        // The C# core's space names, numbered as the core's ABI numbers them, and the one whose reads are the CPU's.
        protected abstract IReadOnlyList<string> SpaceNames { get; }
        protected virtual int CpuBusSpace => -1;

        // The spaces the core logs a store under, as the C# core's write observer names them, and back.
        protected abstract string? ReportedName(uint id);
        protected abstract uint? ReportedSpace(string name);

        // The pad bit a button is, or -1; and the machine's pad for a frontend's port, or -1 for one the console ignores.
        protected abstract int ButtonBit(PadButton button);
        protected virtual int PortFor(int port) => Math.Clamp(port, 0, _pads.Length - 1);

        protected abstract uint MuteMask();

        // The oracle's refusals before the library sees the image, and its battery file for this game.
        protected abstract BatterySave Prepare(string path, byte[] image);

        // After a machine is adopted: the mirror's load, and whatever else the console keeps per game.
        protected virtual void Loaded(string path) { }

        // The C# core's refusals, with its messages, before the bytes reach the machine.
        protected abstract void CheckState(byte[] state);

        // The console's own exception for a status of its band, or null; and its words for one.
        protected abstract Exception? OwnRefusal(int status, ulong detail);
        protected abstract string? OwnWords(long status);

        public string Words(long status) => OwnWords(status) ?? NativeMachine.Shared(status, _stateName) ?? $"status {status}";

        // The console's exception first, then the reproduced C# exceptions, whose types are the oracle's - see EmuSen_NativeCores.md §3.3.
        protected Exception Refusal(int status, ulong detail) => OwnRefusal(status, detail) ?? status switch
        {
            NativeInterface.FaultBase - 1 => new IndexOutOfRangeException("Index was outside the bounds of the array."),
            NativeInterface.FaultBase - 2 => new DivideByZeroException("Attempted to divide by zero."),
            NativeInterface.FaultBase - 3 => new ArgumentOutOfRangeException(),
            NativeInterface.FaultBase - 4 => new InvalidOperationException($"{_engine} reproduced an InvalidOperationException."),
            NativeInterface.FaultBase - 5 => new OverflowException(),
            _ => new InvalidOperationException($"{_engine} refused: {Words(status)}."),
        };

        public NativeDebugBridge Debug => _bridge;

        // The instruction the processor is about to run, live, as the registry compares it.
        public int Pc => _bridge.ProgramCounter();

        public new PortMachine Machine
        {
            get
            {
                CoreMachine m = base.Machine;
                return _view is { } v && ReferenceEquals(v.Core, m) ? v : _view = new PortMachine(m, _port);
            }
        }

        public double LastFrameMilliseconds => LastFramePhases[0].Milliseconds;

        // Halted in front of a breakpoint, with the frame left open for the next RunFrame to resume, as the C# cores keep it.
        public override bool IsHaltedAtBreakpoint => _halted;
        public override int HaltedAddress => _haltedAt;

        // A console's RESET, which the C# cores let end a halt.
        protected void ClearHalt() => _halted = false;

        public override CheatRegistry Cheats
        {
            get => MirrorCheats;
            set
            {
                MirrorCheats = value;
                base.Cheats = value;
            }
        }

        // The console's rules first, then the adapter's load with this game's battery file, then the held pads and the mirror.
        public override void LoadRom(string path)
        {
            byte[] image = File.ReadAllBytes(path);
            _battery = Prepare(path, image);
            try { base.LoadRom(path); }
            catch (CoreRefusedException e) { throw Refusal(e.Status, e.Detail); }
            CoreMachine m = base.Machine;
            for (int pad = 0; pad < _pads.Length; pad++) m.SetButtons(pad, _pads[pad], 0xFF);
            _halted = false;
            Loaded(path);
        }

        protected override BatterySave BatteryFile(string path, string console, uint which, string suffix) => which == 0 ? _battery : BatterySave.None;

        // The whole pad sent each time, as the console's pad is in no state - see EmuSen_NativeCores.md §3.8.
        public override void SetButton(int port, PadButton button, bool pressed)
        {
            int bit = ButtonBit(button);
            if (bit < 0) return;
            int pad = PortFor(port);
            if (pad < 0) return;
            _pads[pad] = pressed ? _pads[pad] | (1u << bit) : _pads[pad] & ~(1u << bit);
            if (IsRomLoaded) base.Machine.SetButtons(pad, _pads[pad], 0xFF);
        }

        // Through the bridge when anything can halt or record, with the first instruction checked unless resuming; else the plain frame.
        protected override bool AdvanceFrame(CoreMachine m)
        {
            bool resuming = _halted;
            _halted = false;
            if (_bridge.Armed)
            {
                if (_bridge.RunFrame(resuming, out int haltedAt)) return true;
                (_halted, _haltedAt) = (true, haltedAt);
                return false;
            }
            try { m.Advance(); }
            catch (CoreRefusedException e) { throw Refusal(e.Status, e.Detail); }
            return true;
        }

        // The C# core's refusals with its messages; past them a refusal is the core's own words.
        public override void LoadState(Stream stream)
        {
            if (!IsRomLoaded) throw new InvalidOperationException("LoadState() called before LoadRom().");
            using var copy = new MemoryStream();
            stream.CopyTo(copy);
            byte[] state = copy.ToArray();
            CheckState(state);
            base.LoadState(new MemoryStream(state, writable: false));
        }

        protected int SpaceNumber(string spaceName)
        {
            IReadOnlyList<string> names = SpaceNames;
            for (int i = 0; i < names.Count; i++) if (names[i] == spaceName) return i;
            return -1;
        }

        // A read of the CPU's bus first brings the ROM patches up to the registry, so it answers as the C# core would.
        public override byte ReadSpace(string spaceName, int address)
        {
            int space = SpaceNumber(spaceName);
            if (!IsRomLoaded || space < 0) return 0;
            CoreMachine m = base.Machine;
            if (space == CpuBusSpace) RefreshCheats(m);
            Span<byte> one = stackalloc byte[1];
            m.ReadSpace((uint)space, address, one);
            return one[0];
        }

        // A store to the CPU's bus while a debugger listens is reported, as the C# core's bus reports it.
        public override void WriteSpace(string spaceName, int address, byte value)
        {
            int space = SpaceNumber(spaceName);
            if (!IsRomLoaded || space < 0) return;
            CoreMachine m = base.Machine;
            if (space == CpuBusSpace && _bridge.Listening)
            {
                _bridge.Observed(() => m.WriteSpace((uint)space, address, new[] { value }));
                return;
            }
            m.WriteSpace((uint)space, address, stackalloc byte[] { value });
        }

        public int SpaceSize(string spaceName)
        {
            int space = SpaceNumber(spaceName);
            return !IsRomLoaded || space < 0 ? 0 : (int)base.Machine.SpaceSize((uint)space);
        }

        public void SyncMirror()
        {
            if (!IsRomLoaded || !MirrorLoaded) return;
            MirrorCore.LoadState(new MemoryStream(Machine.Save()));
        }

        public void SyncMutes()
        {
            if (!IsRomLoaded || !MirrorLoaded) return;
            base.Machine.SetMutes(MuteMask());
        }
    }
}
