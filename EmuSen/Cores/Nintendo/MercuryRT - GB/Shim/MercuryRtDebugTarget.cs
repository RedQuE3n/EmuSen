using EmuSen.Cores.Native;
using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Cores.Nintendo.MercuryRT
{
    // The Game Boy's debugger over MercuryRT: what it shows read from the machine's state through its layout listing and from its spaces, what halts from the core's registries - see Mercury_Debug.md and Mercury_Native.md §15.
    public sealed class MercuryRtDebugTarget : IDebugTarget
    {
        public const int SpriteCount = 40;
        public const int VramBankStride = 0x2000;
        private const int WramBankSize = 0x1000;

        // The four greys the core draws a Game Boy picture in, lightest first.
        public static readonly byte[] DmgShades = { 0xFF, 0xFF, 0xFF, 0xAA, 0xAA, 0xAA, 0x55, 0x55, 0x55, 0x00, 0x00, 0x00 };

        private readonly MercuryRtCore _core;
        private readonly List<IDebugMemorySpace> _spaces = new();
        private readonly DebugCpu[] _cpus;
        private byte[] _state = Array.Empty<byte>();
        private StateLayout? _layout;

        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites;
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes;
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels;
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad;

        public MercuryRtDebugTarget(MercuryRtCore core)
        {
            _core = core;

            // The state is read before the first snapshot is taken - see EmuSen_CoreAPI.md §30.2.
            Refresh();
            foreach (string name in MercuryRtCore.SpaceNameList)
                _spaces.Add(new DelegateDebugMemorySpace(name, () => core.SpaceSize(name), a => core.ReadSpace(name, a),
                    name == MercuryRtCore.SpaceRom ? null : (a, v) => core.WriteSpace(name, a, v), name == MercuryRtCore.SpaceCpuBus));

            _cpuRegisters = new(ReadCpuRegisters, ReadCpuRegisters());
            _videoRegisters = new(ReadVideoRegisters, ReadVideoRegisters());
            _apuRegisters = new(ReadApuRegisters, ReadApuRegisters());
            _coprocessorRegisters = new(() => Array.Empty<DebugRegisterValue>(), Array.Empty<DebugRegisterValue>());
            _sprites = new(ReadSprites, ReadSprites());
            _palettes = new(ReadPalettes, ReadPalettes());
            _audioChannels = new(ReadAudioChannels, ReadAudioChannels());

            // MercuryRT does not time its own subsystems, and an empty list is the documented way to say so.
            _hardwareLoad = new(() => Array.Empty<DebugLoadInfo>(), Array.Empty<DebugLoadInfo>());

            _cpus = new[]
            {
                new DebugCpu("cpu", "SM83", core.Breakpoints)
                {
                    Coverage = core.Coverage,
                    CallStack = core.CallStack,
                    CodeSpace = MercuryRtCore.SpaceCpuBus,
                    Registers = _cpuRegisters,
                    ProgramCounter = () => core.Pc,
                },
            };
            core.Listen();
        }

        public string CoreName => _core.CoreName;
        public long FrameCount => _core.TotalFrames;
        public int MaxSprites => SpriteCount;

        // One byte per map cell on a DMG; the CGB's attribute byte is in the other VRAM bank, not the next address.
        public int TilemapEntryStride => 1;

        public WatchRegistry Watches => _core.Watches;
        public FrameLogRegistry FrameLog => _core.FrameLog;
        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public CheatRegistry Cheats => _core.Cheats;
        public CoverageRegistry? Coverage => _core.Coverage;
        public CallStackRegistry? CallStack => _core.CallStack;
        public LabelRegistry? Labels => _core.Labels;
        public IReadOnlyList<DebugCpu> DebugCpus => _cpus;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => _cpuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => _videoRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => _apuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => _coprocessorRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _sprites;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _palettes;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _audioChannels;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _hardwareLoad;

        // Five vectors of eight bytes each, which is why a handler is nearly always a jump.
        public IReadOnlyList<InterruptVector> InterruptVectors => new[]
        {
            new InterruptVector("VBlank", 0x0040),
            new InterruptVector("LCD STAT", 0x0048),
            new InterruptVector("Timer", 0x0050),
            new InterruptVector("Serial", 0x0058),
            new InterruptVector("Joypad", 0x0060),
        };

        // The state the views read, taken again at each refresh, as a frontend's refresh is the moment its windows read.
        private void Refresh()
        {
            if (!_core.IsRomLoaded)
            {
                (_state, _layout) = (Array.Empty<byte>(), null);
                return;
            }
            (_state, _layout) = _core.Snapshot();
        }

        private bool Loaded => _layout is not null;
        private long F(string path) => _layout!.Get(_state, path);
        private bool B(string path) => F(path) != 0;
        private ReadOnlySpan<byte> Bytes(string path) => _layout!.Bytes(_state, path);
        private byte Bus(int address) => _core.ReadSpace(MercuryRtCore.SpaceCpuBus, address);

        // The console is a Game Boy Color; colour mode is that console running a cartridge made for it - see Mercury_Model.md §2.
        private bool CgbHardware => Loaded && _state[8] != 0;
        private bool Cgb => CgbHardware && _core.Cartridge.ColourGame;
        private bool DmgCompat => CgbHardware && !_core.Cartridge.ColourGame;

        public void RefreshProviders()
        {
            Refresh();
            _cpuRegisters.Refresh();
            _videoRegisters.Refresh();
            _apuRegisters.Refresh();
            _coprocessorRegisters.Refresh();
            _sprites.Refresh();
            _palettes.Refresh();
            _audioChannels.Refresh();
            _hardwareLoad.Refresh();
        }

        // The Apply button, which must not wait for a frame the pause is not running - see EmuSen_Cheats.md §6.
        public void ApplyCheats() => _core.ApplyCheats();

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => _spaces;

        private IDebugMemorySpace? FindSpace(string name)
        {
            foreach (var space in _spaces)
                if (string.Equals(space.Name, name, StringComparison.OrdinalIgnoreCase)) return space;
            return null;
        }

        private IReadOnlyList<DebugRegisterValue> ReadCpuRegisters()
        {
            if (!Loaded) return Array.Empty<DebugRegisterValue>();
            byte f = (byte)F("Cpu.F");
            return new[]
            {
                new DebugRegisterValue("A", (ulong)F("Cpu.A"), 8),
                new DebugRegisterValue("F", f, 8),
                new DebugRegisterValue("BC", (ulong)(F("Cpu.B") << 8 | F("Cpu.C")), 16),
                new DebugRegisterValue("DE", (ulong)(F("Cpu.D") << 8 | F("Cpu.E")), 16),
                new DebugRegisterValue("HL", (ulong)(F("Cpu.H") << 8 | F("Cpu.L")), 16),
                new DebugRegisterValue("SP", (ulong)F("Cpu.SP"), 16),
                new DebugRegisterValue("PC", (ulong)F("Cpu.PC"), 16),
                new DebugRegisterValue("Z", (ulong)((f >> 7) & 1), 1),
                new DebugRegisterValue("N", (ulong)((f >> 6) & 1), 1),
                new DebugRegisterValue("H", (ulong)((f >> 5) & 1), 1),
                new DebugRegisterValue("C", (ulong)((f >> 4) & 1), 1),
                new DebugRegisterValue("IME", (ulong)F("Cpu.Ime"), 1),
                new DebugRegisterValue("HALT", (ulong)F("Cpu.Halted"), 1),
                new DebugRegisterValue("IE", (ulong)F("Bus.InterruptEnable"), 8),
                new DebugRegisterValue("IF", (ulong)F("Bus.InterruptFlags"), 8),
                new DebugRegisterValue("Cycles", (ulong)F("Cpu.<Cycles>k__BackingField"), 64),
            };
        }

        // STAT and the colour palette indices as the CPU reads them; the rest as the state holds them.
        private IReadOnlyList<DebugRegisterValue> ReadVideoRegisters()
        {
            if (!Loaded) return Array.Empty<DebugRegisterValue>();
            var values = new List<DebugRegisterValue>
            {
                new("LCDC", (ulong)F("Bus.Ppu.Lcdc"), 8),
                new("STAT", Bus(0xFF41), 8),
                new("SCY", (ulong)F("Bus.Ppu.Scy"), 8),
                new("SCX", (ulong)F("Bus.Ppu.Scx"), 8),
                new("LY", (ulong)F("Bus.Ppu.Ly"), 8),
                new("LYC", (ulong)F("Bus.Ppu.Lyc"), 8),
                new("BGP", (ulong)F("Bus.Ppu.Bgp"), 8),
                new("OBP0", (ulong)F("Bus.Ppu.Obp0"), 8),
                new("OBP1", (ulong)F("Bus.Ppu.Obp1"), 8),
                new("WY", (ulong)F("Bus.Ppu.Wy"), 8),
                new("WX", (ulong)F("Bus.Ppu.Wx"), 8),
                new("Mode", (ulong)F("Bus.Ppu.Mode"), 2),
                new("Dot", (ulong)F("Bus.Ppu.Dot"), 16),
                new("WindowLine", (ulong)F("Bus.Ppu.WindowLine"), 8),
            };
            if (Cgb)
            {
                values.Add(new DebugRegisterValue("VBK", (ulong)F("Bus.VramBank"), 1));
                values.Add(new DebugRegisterValue("SVBK", (ulong)F("Bus.WramBank"), 3));
                values.Add(new DebugRegisterValue("BCPS", Bus(0xFF68), 8));
                values.Add(new DebugRegisterValue("OCPS", Bus(0xFF6A), 8));
                values.Add(new DebugRegisterValue("KEY1", (ulong)(B("Bus.DoubleSpeed") ? 0x80 : 0x00), 8));
            }
            return values;
        }

        // The sound registers as the CPU reads them, and the timer and board that ride along here - see Mercury_Debug.md §3.
        private IReadOnlyList<DebugRegisterValue> ReadApuRegisters()
        {
            if (!Loaded) return Array.Empty<DebugRegisterValue>();
            var values = new List<DebugRegisterValue>();
            for (int address = 0xFF10; address <= 0xFF26; address++) values.Add(new DebugRegisterValue($"${address:X4}", Bus(address), 8));
            values.Add(new DebugRegisterValue("Power", (ulong)F("Bus.Apu.PoweredOn"), 1));
            values.Add(new DebugRegisterValue("SeqStep", (ulong)F("Bus.Apu.SequencerStep"), 3));
            values.Add(new DebugRegisterValue("DIV", (ulong)(F("Bus._divCounter") >> 8), 8));
            values.Add(new DebugRegisterValue("TIMA", (ulong)F("Bus._tima"), 8));
            values.Add(new DebugRegisterValue("TMA", (ulong)F("Bus._tma"), 8));
            values.Add(new DebugRegisterValue("TAC", (ulong)F("Bus._tac"), 8));
            values.Add(new DebugRegisterValue($"Board:{_core.Cartridge.Board}", 0, 8));
            return values;
        }

        // Level is a 0-100 rescale of the channel's own volume, which for the wave is a shift - see Mercury_Apu.md §3.4.
        private IReadOnlyList<DebugAudioChannelInfo> ReadAudioChannels()
        {
            if (!Loaded) return Array.Empty<DebugAudioChannelInfo>();
            const string wave = "Bus.Apu.Wave.", noise = "Bus.Apu.Noise.";
            int waveVolume = (int)F(wave + "VolumeCode");
            return new[]
            {
                Pulse(0, "Pulse 1", "Bus.Apu.Pulse1."),
                Pulse(1, "Pulse 2", "Bus.Apu.Pulse2."),
                new DebugAudioChannelInfo(2, "Wave", B(wave + "Enabled") && B(wave + "DacEnabled"),
                    waveVolume == 0 ? 0 : Scale(4 - waveVolume, 3), _core.IsChannelMuted(2),
                    $"Freq={F(wave + "Frequency")} Vol={waveVolume} Len={F(wave + "Length.Counter")} Dac={B(wave + "DacEnabled")}"),
                new DebugAudioChannelInfo(3, "Noise", B(noise + "Enabled") && B(noise + "DacEnabled"),
                    Scale((int)F(noise + "Envelope.Volume"), 15), _core.IsChannelMuted(3),
                    $"Shift={F(noise + "ClockShift")} Div={F(noise + "DivisorCode")} Short={B(noise + "ShortMode")} " +
                    $"Vol={F(noise + "Envelope.Volume")} Len={F(noise + "Length.Counter")}"),
            };
        }

        private DebugAudioChannelInfo Pulse(int index, string name, string p) =>
            new(index, name, B(p + "Enabled") && B(p + "DacEnabled"), Scale((int)F(p + "Envelope.Volume"), 15), _core.IsChannelMuted(index),
                $"Duty={F(p + "Duty")} Freq={F(p + "Frequency")} Vol={F(p + "Envelope.Volume")} Len={F(p + "Length.Counter")}" +
                (B(p + "HasSweep") ? $" Sweep={F(p + "SweepPeriod")}/{F(p + "SweepShift")}{(B(p + "SweepNegate") ? "-" : "+")}" : ""));

        private static int Scale(int value, int max) => (int)Math.Round(value / (double)max * 100.0);

        private IReadOnlyList<DebugSpriteInfo> ReadSprites()
        {
            if (!Loaded) return Array.Empty<DebugSpriteInfo>();
            ReadOnlySpan<byte> oam = Bytes("Bus.Oam");
            int height = (F("Bus.Ppu.Lcdc") & 0x04) != 0 ? 16 : 8;
            bool cgb = Cgb;
            var sprites = new List<DebugSpriteInfo>();
            for (int i = 0; i < SpriteCount; i++)
            {
                int y = oam[i * 4] - 16;
                int x = oam[(i * 4) + 1] - 8;
                byte tile = oam[(i * 4) + 2];
                byte attributes = oam[(i * 4) + 3];

                // Parking a sprite off the top or bottom is the standard way to hide one.
                if (y <= -height || y >= MercuryRtCore.ScreenHeightPixels) continue;

                int palette = cgb ? attributes & 0x07 : (attributes & 0x10) != 0 ? 1 : 0;
                sprites.Add(new DebugSpriteInfo(i, x, y, 8, height, tile, palette, (attributes & 0x80) != 0 ? 0 : 1, (attributes & 0x20) != 0, (attributes & 0x40) != 0));
            }
            return sprites;
        }

        // Four DMG palettes of four greys, or sixteen colour ones - see Mercury_Debug.md §3.1.
        private IReadOnlyList<DebugPaletteInfo> ReadPalettes()
        {
            if (!Loaded) return Array.Empty<DebugPaletteInfo>();
            byte[] bg = Bytes("Bus.Ppu.BgPaletteRam").ToArray(), obj = Bytes("Bus.Ppu.ObjPaletteRam").ToArray();
            byte bgp = (byte)F("Bus.Ppu.Bgp"), obp0 = (byte)F("Bus.Ppu.Obp0"), obp1 = (byte)F("Bus.Ppu.Obp1");
            var palettes = new List<DebugPaletteInfo>();
            if (Cgb)
            {
                for (int p = 0; p < 8; p++) palettes.Add(ColorPalette(p, bg, p));
                for (int p = 0; p < 8; p++) palettes.Add(ColorPalette(8 + p, obj, p));
                return palettes;
            }
            if (DmgCompat)
            {
                palettes.Add(CompatibilityPalette(0, bgp, bg, 0));
                palettes.Add(CompatibilityPalette(1, obp0, obj, 0));
                palettes.Add(CompatibilityPalette(2, obp1, obj, 1));
                return palettes;
            }
            palettes.Add(ShadePalette(0, bgp));
            palettes.Add(ShadePalette(1, obp0));
            palettes.Add(ShadePalette(2, obp1));
            return palettes;
        }

        // A DMG palette register's shades through a colour palette, as the Color shows a Game Boy game - see Mercury_Model.md §4.1.
        private static DebugPaletteInfo CompatibilityPalette(int index, byte register, byte[] paletteRam, int palette)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);
            for (int c = 0; c < 4; c++) colors.Add(Rgb(paletteRam, (palette * 8) + (((register >> (c * 2)) & 0x03) * 2)));
            return new DebugPaletteInfo(index, colors);
        }

        private static DebugPaletteInfo ShadePalette(int index, byte register)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);
            for (int c = 0; c < 4; c++)
            {
                int offset = ((register >> (c * 2)) & 0x03) * 3;
                colors.Add((DmgShades[offset], DmgShades[offset + 1], DmgShades[offset + 2]));
            }
            return new DebugPaletteInfo(index, colors);
        }

        private static DebugPaletteInfo ColorPalette(int index, byte[] paletteRam, int palette)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);
            for (int c = 0; c < 4; c++) colors.Add(Rgb(paletteRam, (palette * 8) + (c * 2)));
            return new DebugPaletteInfo(index, colors);
        }

        private static (byte, byte, byte) Rgb(byte[] paletteRam, int entry)
        {
            int rgb555 = paletteRam[entry] | (paletteRam[entry + 1] << 8);
            return (Expand(rgb555 & 0x1F), Expand((rgb555 >> 5) & 0x1F), Expand((rgb555 >> 10) & 0x1F));
        }

        private static byte Expand(int channel) => (byte)((channel << 3) | (channel >> 2));

        public void SetChannelMuted(int index, bool muted) => _core.SetChannelMuted(index, muted);

        // MercuryRT's own queue, peeked, unlike ICore.DequeueAudioSamples - see EmuSen_Audio_Sync.md §7.
        public (short[] Samples, int SampleRate) GetAudioSamples() => (_core.IsRomLoaded ? _core.Machine.Core.PeekAudio() : Array.Empty<short>(), _core.AudioSampleRate);

        // HDMA is one channel with two modes, so the table has one row - see Mercury_Cgb.md §4.
        public IReadOnlyList<DebugDmaChannel> DmaChannels
        {
            get
            {
                if (!Cgb) return Array.Empty<DebugDmaChannel>();
                int blocks = (int)F("Bus.HdmaBlocksLeft");
                bool hblank = B("Bus.HdmaIsHBlankDriven");
                return new[]
                {
                    new DebugDmaChannel(0, blocks > 0 && !hblank, hblank, (byte)(hblank ? 0x80 : 0x00), 0x00, (int)F("Bus.HdmaSource"), blocks * 16,
                        hblank && blocks > 0, 0, (byte)Math.Min(byte.MaxValue, blocks), 0x8000 | (int)F("Bus.HdmaDestination")),
                };
            }
        }

        public string? NameDmaDestination(byte register) => "VRAM";

        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            var space = FindSpace(spaceName);
            if (space is null || space.Size == 0) return Array.Empty<DisassembledInstruction>();
            var result = new List<DisassembledInstruction>();
            int pc = address;
            for (int i = 0; i < count; i++)
            {
                byte opcode = space.Read(pc);
                int length = Sm83Disassembler.LengthOf(opcode);
                var bytes = new byte[length];
                for (int b = 0; b < length; b++) bytes[b] = space.Read(pc + b);
                byte low = length > 1 ? bytes[1] : (byte)0;
                byte high = length > 2 ? bytes[2] : (byte)0;
                result.Add(new DisassembledInstruction(pc, bytes, Sm83Disassembler.MnemonicOf(opcode, low), Sm83Disassembler.FormatOperand(opcode, pc, low, high)));
                pc += length;
            }
            return result;
        }

        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr)
        {
            if (instr.Bytes.Count == 0) return null;
            byte opcode = instr.Bytes[0];
            int length = Sm83Disassembler.LengthOf(opcode);
            if (instr.Bytes.Count < length) return null;
            int operand = length switch
            {
                2 => instr.Bytes[1],
                3 => instr.Bytes[1] | (instr.Bytes[2] << 8),
                _ => -1,
            };
            return Sm83Disassembler.ClassifyStaticReference(opcode, operand, instr.Address);
        }

        // A map cell is the tile number, and on a CGB the attribute that goes with it.
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address)
        {
            byte tile = space.Read(address);
            if (!Cgb) return $"${tile:X2}";
            byte attributes = space.Read(address + VramBankStride);
            string flips = $"{((attributes & 0x20) != 0 ? "X" : "-")}{((attributes & 0x40) != 0 ? "Y" : "-")}";
            return $"${tile:X2} p{attributes & 0x07} b{(attributes >> 3) & 0x01} {flips}{((attributes & 0x80) != 0 ? " pri" : "")}";
        }

        // 16 bytes per tile, interleaved by row rather than split into two planes the way an NES tile is.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2) throw new ArgumentException($"A Game Boy tile is always 2bpp; got {bpp}.", nameof(bpp));
            var pixels = new byte[64];
            for (int row = 0; row < 8; row++)
            {
                byte low = space.Read(address + (row * 2));
                byte high = space.Read(address + (row * 2) + 1);
                for (int column = 0; column < 8; column++)
                {
                    int bit = 7 - column;
                    pixels[(row * 8) + column] = (byte)(((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1));
                }
            }
            return pixels;
        }

        // Grey by pixel value: a tile has no palette of its own until a map entry picks one.
        public (byte[] Rgba, int Width, int Height) RenderTileSheet()
        {
            if (!Loaded) return (Array.Empty<byte>(), 0, 0);
            ReadOnlySpan<byte> vram = Bytes("Bus.Vram");
            const int tilesPerRow = 16;
            int tileCount = vram.Length / 16;
            int rows = (tileCount + tilesPerRow - 1) / tilesPerRow;
            int width = tilesPerRow * 8, height = rows * 8;
            var rgba = new byte[width * height * 4];
            for (int tile = 0; tile < tileCount; tile++)
            {
                int originX = (tile % tilesPerRow) * 8, originY = (tile / tilesPerRow) * 8;
                for (int row = 0; row < 8; row++)
                {
                    byte low = vram[(tile * 16) + (row * 2)], high = vram[(tile * 16) + (row * 2) + 1];
                    for (int column = 0; column < 8; column++)
                    {
                        int bit = 7 - column;
                        int shade = (((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1)) * 3;
                        int offset = (((originY + row) * width) + originX + column) * 4;
                        (rgba[offset], rgba[offset + 1], rgba[offset + 2], rgba[offset + 3]) = (DmgShades[shade], DmgShades[shade + 1], DmgShades[shade + 2], 0xFF);
                    }
                }
            }
            return (rgba, width, height);
        }

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch()
        {
            var palettes = ReadPalettes();
            if (palettes.Count == 0) return (Array.Empty<byte>(), 0, 0);
            const int swatch = 16, columns = 4;
            int width = columns * swatch, height = palettes.Count * swatch;
            var rgba = new byte[width * height * 4];
            for (int p = 0; p < palettes.Count; p++)
            {
                for (int c = 0; c < columns; c++)
                {
                    var (r, g, b) = palettes[p].Colors[c];
                    for (int y = 0; y < swatch; y++)
                    {
                        for (int x = 0; x < swatch; x++)
                        {
                            int offset = ((((p * swatch) + y) * width) + (c * swatch) + x) * 4;
                            (rgba[offset], rgba[offset + 1], rgba[offset + 2], rgba[offset + 3]) = (r, g, b, 0xFF);
                        }
                    }
                }
            }
            return (rgba, width, height);
        }

        // Only the ranges whose destination does not depend on live mapper or bank state.
        public PhysicalAddress? ResolvePhysical(int cpuAddress)
        {
            if (!Loaded) return null;
            if (cpuAddress < 0x4000) return new PhysicalAddress(MercuryRtCore.SpaceRom, cpuAddress);
            if (cpuAddress < 0x8000) return null;
            if (cpuAddress < 0xA000) return new PhysicalAddress(MercuryRtCore.SpaceVram, ((int)F("Bus.VramBank") * VramBankStride) + (cpuAddress - 0x8000));
            if (cpuAddress < 0xC000) return null;
            if (cpuAddress < 0xD000) return new PhysicalAddress(MercuryRtCore.SpaceWram, cpuAddress - 0xC000);
            if (cpuAddress < 0xE000) return new PhysicalAddress(MercuryRtCore.SpaceWram, (Math.Max(1, (int)F("Bus.WramBank")) * WramBankSize) + (cpuAddress - 0xD000));
            if (cpuAddress < 0xFE00) return null;
            if (cpuAddress < 0xFEA0) return new PhysicalAddress(MercuryRtCore.SpaceOam, cpuAddress - 0xFE00);
            if (cpuAddress < 0xFF00) return new PhysicalAddress("PROHIBITED", cpuAddress - 0xFEA0, false);
            if (cpuAddress < 0xFF80) return new PhysicalAddress("IOREG", cpuAddress - 0xFF00, false);
            if (cpuAddress < 0xFFFF) return new PhysicalAddress(MercuryRtCore.SpaceHram, cpuAddress - 0xFF80);
            return new PhysicalAddress("IE", 0, false);
        }

        public string GetSummaryText()
        {
            if (!Loaded) return "No ROM loaded.";
            GbCartridge cart = _core.Cartridge;
            var text = new StringBuilder();
            text.AppendLine($"{_core.CoreName} (MercuryRT) - frame {FrameCount}, LY {F("Bus.Ppu.Ly")}, mode {F("Bus.Ppu.Mode")}");
            text.AppendLine($"{cart.Board}, {cart.RomBanks}x16K ROM, {(cart.RamLength == 0 ? "no RAM" : $"{cart.RamLength / 1024}K RAM")}" +
                            $"{(cart.HasBattery ? ", battery" : "")}{(cart.HasTimer ? ", RTC" : "")}");
            text.AppendLine($"CPU  A={F("Cpu.A"):X2} F={F("Cpu.F"):X2} BC={F("Cpu.B"):X2}{F("Cpu.C"):X2} DE={F("Cpu.D"):X2}{F("Cpu.E"):X2} HL={F("Cpu.H"):X2}{F("Cpu.L"):X2} " +
                            $"SP={F("Cpu.SP"):X4} PC={F("Cpu.PC"):X4}");
            text.AppendLine($"     IME={F("Cpu.Ime")} HALT={F("Cpu.Halted")} IE={F("Bus.InterruptEnable"):X2} IF={F("Bus.InterruptFlags"):X2}");
            text.AppendLine($"PPU  LCDC={F("Bus.Ppu.Lcdc"):X2} STAT={Bus(0xFF41):X2} SCY={F("Bus.Ppu.Scy"):X2} SCX={F("Bus.Ppu.Scx"):X2} " +
                            $"WY={F("Bus.Ppu.Wy"):X2} WX={F("Bus.Ppu.Wx"):X2}");
            if (Cgb)
            {
                long blocks = F("Bus.HdmaBlocksLeft");
                text.AppendLine($"CGB  VBK={F("Bus.VramBank")} SVBK={F("Bus.WramBank")} speed={(B("Bus.DoubleSpeed") ? "double" : "normal")} " +
                                $"HDMA={(blocks == 0 ? "idle" : $"{blocks} blocks")}");
            }
            text.Append($"APU  power={(B("Bus.Apu.PoweredOn") ? "on" : "off")} vol={F("Bus.Apu.LeftVolume")}/{F("Bus.Apu.RightVolume")} " +
                        $"pan={F("Bus.Apu.Panning"):X2} buffered={_core.Machine.Core.BufferedSamples / 2} frames");
            return text.ToString();
        }
    }
}
