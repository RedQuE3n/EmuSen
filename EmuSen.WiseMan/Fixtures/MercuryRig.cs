using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Fixtures
{
    // MercuryRT under WiseMan's machine tests: reads and edits through the committed layout listings, setup through a state round trip, time through frames, steps and ticks - see Mercury_Native.md §12.1, §14.5.
    public sealed unsafe class MercuryRig : IDisposable
    {
        private static readonly delegate* unmanaged<nint, int> StepOf = (delegate* unmanaged<nint, int>)MercuryNative.Engine.Export("mercuryrt_core_step");
        private static readonly delegate* unmanaged<nint, int, int> TickOf = (delegate* unmanaged<nint, int, int>)MercuryNative.Engine.Export("mercuryrt_core_tick");
        private static readonly delegate* unmanaged<nint, int> StopOf = (delegate* unmanaged<nint, int>)MercuryNative.Engine.Export("mercuryrt_core_stop");
        private static readonly delegate* unmanaged<nint, byte*, nuint, long> SerialOf = (delegate* unmanaged<nint, byte*, nuint, long>)MercuryNative.Engine.Export("mercuryrt_core_serial");

        // Interrupt bits of IE and IF, and the PPU's modes as the state numbers them - see Pan Docs, "Interrupts" and "STAT".
        public const int VBlank = 0, LcdStat = 1, Timer = 2, Serial = 3, Joypad = 4;
        public const int HBlankMode = 0, VBlankMode = 1, OamScanMode = 2, DrawingMode = 3;

        // The line's timing in T-cycles: 456 a line, 80 of OAM scan, 172 of drawing with SCX zero; 154 lines.
        public const int CyclesPerScanline = 456, OamScanCycles = 80, BaseDrawingCycles = 172, TotalScanlines = 154;
        public const int ScreenWidth = MercuryRtCore.ScreenWidthPixels, ScreenHeight = MercuryRtCore.ScreenHeightPixels;
        public const int VramBankStride = 0x2000, WramBankSize = 0x1000, RomBankSize = 0x4000;

        // The flag register's four flags.
        public const int FlagZ = 0x80, FlagN = 0x40, FlagH = 0x20, FlagC = 0x10;

        private readonly string _rom;
        private readonly bool _temporary;
        private readonly MercuryRtCore _core;

        private MercuryRig(string rom, bool temporary, GbModel model)
        {
            _rom = rom;
            _temporary = temporary;
            Rom = File.ReadAllBytes(rom);
            Assert.True(MercuryRtCore.Available, MercuryNative.Engine.Report);
            Assert.True(StepOf != null && TickOf != null && StopOf != null && SerialOf != null, "libmercuryrt lacks the rig's extensions");
            _core = new MercuryRtCore { Model = model };
            _core.LoadRom(rom);
        }

        // The cartridge image, whose header picks the layout listing a state follows.
        public byte[] Rom { get; }

        // An image written to a temporary file the rig deletes.
        public static MercuryRig Create(byte[] rom, GbModel model = GbModel.Auto) => Open(SyntheticGbRom.WriteTemp(rom), model, temporary: true);

        // A ROM file in place, which the rig leaves alone.
        public static MercuryRig Open(string path, GbModel model = GbModel.Auto, bool temporary = false) => new(path, temporary, model);

        // The engine as a frontend holds it.
        public MercuryRtCore Core => _core;

        private T Run<T>(Func<nint, T> call)
        {
            lock (_core.Machine.Core.Gate) return call(_core.Machine.Core.Handle);
        }

        // The machine without its CPU, for a number of T-cycles.
        public void Tick(int cycles) => Run(h => TickOf(h, cycles));

        // The CPU's read and write, through the bus's decode and blocking.
        public byte Read(int address) => _core.ReadSpace(MercuryRtCore.SpaceCpuBus, address & 0xFFFF);
        public void Write(int address, byte value) => _core.WriteSpace(MercuryRtCore.SpaceCpuBus, address & 0xFFFF, value);

        // A named space as the debugger reads and writes it: VRAM, WRAM and OAM by their whole arrays, no blocking.
        public byte ReadSpace(string space, int address) => _core.ReadSpace(space, address);
        public void WriteSpace(string space, int address, byte value) => _core.WriteSpace(space, address, value);

        // One instruction with the frame loop's acknowledge and stall; its T-cycles; an illegal opcode throws NotSupportedException.
        public int Step()
        {
            int cycles = Run(h => StepOf(h));
            if (cycles == -20) throw new NotSupportedException("MercuryRT stopped on an opcode no SM83 has.");
            return cycles;
        }

        // STOP with KEY1 armed: the speed switch.
        public void Stop() => Run(h => StopOf(h));

        public IReadOnlyList<byte> SerialLog => Run(h =>
        {
            long n = SerialOf(h, null, 0);
            var log = new byte[n];
            fixed (byte* data = log) SerialOf(h, data, (nuint)log.Length);
            return log;
        });

        public string SerialText => string.Concat(SerialLog.Select(b => (char)b));

        public void SetChannelMuted(int channel, bool muted) => _core.SetChannelMuted(channel, muted);
        public bool IsChannelMuted(int channel) => _core.IsChannelMuted(channel);

        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public IDebugTarget CreateDebugTarget() => _core.CreateDebugTarget();
        public bool IsHaltedAtBreakpoint => _core.IsHaltedAtBreakpoint;
        public int HaltedAddress => _core.HaltedAddress;

        public void RunFrame() => _core.RunFrame();

        // LoadRom again on the same image, which resets the machine.
        public void Reload() => _core.LoadRom(_rom);
        public long TotalFrames => _core.TotalFrames;
        public string CoreName => _core.CoreName;
        public int AudioSampleRate => _core.AudioSampleRate;
        public byte[] GetFrameBufferRgba() => _core.GetFrameBufferRgba();
        public short[] DequeueAudioSamples(int maxFrames) => _core.DequeueAudioSamples(maxFrames);
        public void SetButton(int port, PadButton button, bool pressed) => _core.SetButton(port, button, pressed);
        public bool SkipRendering { set => _core.SkipRendering = value; }
        public void SaveState(Stream stream) => _core.SaveState(stream);
        public void SaveSram() => _core.SaveSram();
        public void LoadState(Stream stream) => _core.LoadState(stream);

        public byte[] State()
        {
            using var stream = new MemoryStream();
            SaveState(stream);
            return stream.ToArray();
        }

        // A field of the engine's state by its listing path, read through the committed layout - see Mercury_Native.md §14.5.
        public long Field(string path)
        {
            byte[] state = State();
            return Listing(state).Get(state, path);
        }

        public bool Flag(string path) => Field(path) != 0;

        public byte FieldByte(string path, int index)
        {
            byte[] state = State();
            return Listing(state).Bytes(state, path)[index];
        }

        public byte[] FieldBytes(string path)
        {
            byte[] state = State();
            return Listing(state).Bytes(state, path).ToArray();
        }

        // A field edit as a state round trip: the engine's state, the field rewritten through the listing, loaded back.
        public void SetField(string path, long value)
        {
            byte[] state = State();
            Listing(state).Set(state, path, value);
            LoadState(new MemoryStream(state));
        }

        public void SetFieldByte(string path, int index, byte value)
        {
            byte[] state = State();
            Listing(state).Bytes(state, path)[index] = value;
            LoadState(new MemoryStream(state));
        }

        // A register pair, high byte first: "BC", "DE" or "HL".
        public int Pair(string pair) => (int)(Field("Cpu." + pair[0]) << 8 | Field("Cpu." + pair[1]));

        // An interrupt raised in IF, as a device raises it.
        public void Request(int interrupt) => SetField("Bus.InterruptFlags", Field("Bus.InterruptFlags") | (1L << interrupt));

        // The cycles a DMA took from the CPU, taken and cleared as the frame loop takes them.
        public int TakePendingStall()
        {
            int stall = (int)Field("Bus._stallCycles");
            if (stall != 0) SetField("Bus._stallCycles", 0);
            return stall;
        }

        // The listing for this state, written from MercuryRT's own state_layout when recording and none is committed.
        private MercuryStateLayout Listing(byte[] state)
        {
            try { return MercuryLayouts.For(Rom, state); }
            catch (InvalidDataException) when (Environment.GetEnvironmentVariable(MercuryLayouts.RecordVariable) == "1")
            {
                MercuryLayouts.Record(Rom[0x147], _core.Machine.Core.Layout());
                return MercuryLayouts.For(Rom, state);
            }
        }

        public void Dispose()
        {
            _core.Dispose();
            if (_temporary) try { File.Delete(_rom); } catch (IOException) { }
        }
    }
}
