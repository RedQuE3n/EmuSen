using EmuSen.Cores.Native;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Crystal.Debugger;
using EmuSen.Galaxia.Input;

namespace EmuSen.WiseMan.Fixtures
{
    // MercuryRT under WiseMan's machine tests: the generic engine, reads and edits through the committed layout listings, setup through a state round trip, time through frames, steps and ticks - see Mercury_Native.md §12.1, §14.5.
    public sealed class MercuryRig : IDisposable
    {
        // Interrupt bits of IE and IF, and the PPU's modes as the state numbers them - see Pan Docs, "Interrupts" and "STAT".
        public const int VBlank = 0, LcdStat = 1, Timer = 2, Serial = 3, Joypad = 4;
        public const int HBlankMode = 0, VBlankMode = 1, OamScanMode = 2, DrawingMode = 3;

        // The line's timing in T-cycles: 456 a line, 80 of OAM scan, 172 of drawing with SCX zero; 154 lines.
        public const int CyclesPerScanline = 456, OamScanCycles = 80, BaseDrawingCycles = 172, TotalScanlines = 154;
        public const int ScreenWidth = MercuryEngines.ScreenWidthPixels, ScreenHeight = MercuryEngines.ScreenHeightPixels;
        public const int VramBankStride = 0x2000, WramBankSize = 0x1000, RomBankSize = 0x4000;

        // The flag register's four flags.
        public const int FlagZ = 0x80, FlagN = 0x40, FlagH = 0x20, FlagC = 0x10;

        private readonly string _rom;
        private readonly bool _temporary;
        private readonly CoreEngine _core;
        private uint _mutes;

        private MercuryRig(string rom, bool temporary, GbModel model)
        {
            _rom = rom;
            _temporary = temporary;
            Rom = File.ReadAllBytes(rom);
            Assert.True(MercuryEngines.Available, MercuryEngines.Report);
            _core = MercuryEngines.Create(TestCores.NoBattery, model);
            _core.LoadRom(rom);
        }

        // The cartridge image, whose header picks the layout listing a state follows.
        public byte[] Rom { get; }

        // An image written to a temporary file the rig deletes.
        public static MercuryRig Create(byte[] rom, GbModel model = GbModel.Auto) => Open(SyntheticGbRom.WriteTemp(rom), model, temporary: true);

        // A ROM file in place, which the rig leaves alone.
        public static MercuryRig Open(string path, GbModel model = GbModel.Auto, bool temporary = false) => new(path, temporary, model);

        // The engine as a frontend holds it.
        public CoreEngine Core => _core;

        // The machine without its CPU, for a number of T-cycles.
        public void Tick(int cycles) => MercuryEngines.Tick(_core, cycles);

        // The CPU's read and write, through the bus's decode and blocking.
        public byte Read(int address) => _core.ReadSpace(MercuryEngines.SpaceCpuBus, address & 0xFFFF);
        public void Write(int address, byte value) => _core.WriteSpace(MercuryEngines.SpaceCpuBus, address & 0xFFFF, value);

        // A named space as the debugger reads and writes it: VRAM, WRAM and OAM by their whole arrays, no blocking.
        public byte ReadSpace(string space, int address) => _core.ReadSpace(space, address);
        public void WriteSpace(string space, int address, byte value) => _core.WriteSpace(space, address, value);

        // One instruction with the frame loop's acknowledge and stall; its T-cycles; an illegal opcode throws NotSupportedException.
        public int Step()
        {
            int cycles = MercuryEngines.Step(_core);
            if (cycles == MercuryEngines.IllegalOpcode) throw new NotSupportedException("MercuryRT stopped on an opcode no SM83 has.");
            return cycles;
        }

        // STOP with KEY1 armed: the speed switch.
        public void Stop() => MercuryEngines.Stop(_core);

        public IReadOnlyList<byte> SerialLog => MercuryEngines.Serial(_core);

        public string SerialText => string.Concat(SerialLog.Select(b => (char)b));

        // A sound channel silenced, sent to the machine at once, as the debugger's mute is.
        public void SetChannelMuted(int channel, bool muted)
        {
            if ((uint)channel >= 4) return;
            _mutes = muted ? _mutes | (1u << channel) : _mutes & ~(1u << channel);
            _core.Machine.SetMutes(_mutes);
        }

        public bool IsChannelMuted(int channel) => (uint)channel < 4 && (_mutes & (1u << channel)) != 0;

        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public IDebugTarget CreateDebugTarget() => _core.CreateDebugTarget();
        public bool IsHaltedAtBreakpoint => _core.IsHaltedAtBreakpoint;
        public int HaltedAddress => _core.HaltedAddress;

        // A frame; an illegal opcode throws NotSupportedException, the word the corpus's verdicts and the goldens' transcripts record.
        public void RunFrame()
        {
            try { _core.RunFrame(); }
            catch (CoreRefusedException e) when (MercuryEngines.IsIllegalOpcode(e)) { throw new NotSupportedException(e.Message, e); }
        }

        // LoadRom again on the same image, which resets the machine and its mutes.
        public void Reload()
        {
            _core.LoadRom(_rom);
            _mutes = 0;
        }
        public long TotalFrames => _core.TotalFrames;
        // The console the machine runs as, GB or GBC, from its machine info; a state from the other console changes it.
        public string CoreName => _core.Machine.Info.System.ToUpperInvariant();
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
        private StateLayout Listing(byte[] state)
        {
            try { return MercuryLayouts.For(Rom, state); }
            catch (InvalidDataException) when (Environment.GetEnvironmentVariable(MercuryLayouts.RecordVariable) == "1")
            {
                MercuryLayouts.Record(Rom[0x147], _core.Machine.Layout());
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
