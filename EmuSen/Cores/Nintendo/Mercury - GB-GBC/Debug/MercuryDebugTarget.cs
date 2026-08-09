using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Mercury.Cpu.Disassembler;
using EmuSen.Cores.Nintendo.Mercury.Memory;
using EmuSen.Cores.Nintendo.Mercury.Video;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Mercury.Debug
{
    // A space backed by a name the core already knows how to read and write - see Mercury_Debug.md §2.
    internal sealed class MercuryDebugMemorySpace : IDebugMemorySpace
    {
        private readonly MercuryCore _core;
        private readonly string _space;

        public string Name { get; }
        public int Size { get; }
        public bool IsWritable { get; }
        public bool HasSideEffects { get; }

        public MercuryDebugMemorySpace(MercuryCore core, string space, string name, bool isWritable, bool hasSideEffects)
        {
            _core = core;
            _space = space;
            Name = name;
            Size = core.SpaceSize(space);
            IsWritable = isWritable;
            HasSideEffects = hasSideEffects;
        }

        public byte Read(int address) => _core.ReadSpace(_space, Wrap(address));

        public void Write(int address, byte value)
        {
            if (IsWritable) _core.WriteSpace(_space, Wrap(address), value);
        }

        private int Wrap(int address) => Size == 0 ? 0 : ((address % Size) + Size) % Size;
    }

    // The Game Boy's IDebugTarget, and what finally lets CoreFactory hand Mercury out. See Mercury_Debug.md.
    public sealed class MercuryDebugTarget : IDebugTarget, IWriteObserver
    {
        private readonly MercuryCore _core;

        private readonly List<IDebugMemorySpace> _spaces = new();

        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites;
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes;
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels;
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad;

        public MercuryDebugTarget(MercuryCore core)
        {
            _core = core;

            BuildSpaces();

            _cpuRegisters = new(ReadCpuRegistersLive, ReadCpuRegistersLive());
            _videoRegisters = new(ReadVideoRegistersLive, ReadVideoRegistersLive());
            _apuRegisters = new(ReadApuRegistersLive, ReadApuRegistersLive());
            _coprocessorRegisters = new(() => Array.Empty<DebugRegisterValue>(), Array.Empty<DebugRegisterValue>());
            _sprites = new(ReadSpritesLive, ReadSpritesLive());
            _palettes = new(ReadPalettesLive, ReadPalettesLive());
            _audioChannels = new(() => Array.Empty<DebugAudioChannelInfo>(), Array.Empty<DebugAudioChannelInfo>());

            // Mercury does not time its own subsystems, and an empty list is the documented way to say so.
            _hardwareLoad = new(() => Array.Empty<DebugLoadInfo>(), Array.Empty<DebugLoadInfo>());

            if (core.Bus != null) core.Bus.WriteObserver = this;
        }

        public string CoreName => _core.CoreName;

        public WatchRegistry Watches => _core.Watches;
        public FrameLogRegistry FrameLog => _core.FrameLog;
        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public CheatRegistry Cheats => _core.Cheats;
        public CoverageRegistry? Coverage => _core.Coverage;
        public LabelRegistry? Labels => _core.Labels;

        public long FrameCount => _core.TotalFrames;

        public int MaxSprites => Ppu.SpriteCount;

        // One byte per map cell on a DMG; the CGB's attribute byte is in the other VRAM bank, not the next address.
        public int TilemapEntryStride => 1;

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

        public IReadOnlyList<DebugCpu> DebugCpus => new[]
        {
            new DebugCpu("cpu", "SM83", _core.Breakpoints)
            {
                Coverage = _core.Coverage,
                CodeSpace = MercuryCore.SpaceCpuBus,
                Registers = _cpuRegisters,
                ProgramCounter = () => _core.Cpu?.PC ?? 0,
            },
        };

        // HDMA is one channel with two modes, so the table has one row - see Mercury_Cgb.md §4.
        public IReadOnlyList<DebugDmaChannel> DmaChannels
        {
            get
            {
                var bus = _core.Bus;
                if (bus is null || !bus.Cgb) return Array.Empty<DebugDmaChannel>();

                return new[]
                {
                    new DebugDmaChannel(
                        0,
                        bus.HdmaBlocksLeft > 0 && !bus.HdmaIsHBlankDriven,
                        bus.HdmaIsHBlankDriven,
                        (byte)(bus.HdmaIsHBlankDriven ? 0x80 : 0x00),
                        0x00,
                        bus.HdmaSource,
                        bus.HdmaBlocksLeft * 16,
                        bus.HdmaIsHBlankDriven && bus.HdmaBlocksLeft > 0,
                        0,
                        (byte)Math.Min(byte.MaxValue, bus.HdmaBlocksLeft),
                        0x8000 | bus.HdmaDestination),
                };
            }
        }

        public string? NameDmaDestination(byte register) => "VRAM";

        public void RefreshProviders()
        {
            _cpuRegisters.Refresh();
            _videoRegisters.Refresh();
            _apuRegisters.Refresh();
            _coprocessorRegisters.Refresh();
            _sprites.Refresh();
            _palettes.Refresh();
            _audioChannels.Refresh();
            _hardwareLoad.Refresh();
        }

        public void OnWrite(string spaceName, int address, byte value) =>
            Watches.RecordWrite(spaceName, address, value, DescribeWriteSite);

        private string DescribeWriteSite() =>
            _core.Cpu is null ? "" : $"PC=${_core.Cpu.LastInstructionPC:X4}";

        private void BuildSpaces()
        {
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceRom, "ROM", false, false));
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceVram, "VRAM", true, false));
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceCartRam, "CARTRAM", true, false));
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceWram, "WRAM", true, false));
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceOam, "OAM", true, false));
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceHram, "HRAM", true, false));

            // $FF00 answers the joypad matrix and $FF41 recomputes STAT, so a bulk scan here is not free.
            _spaces.Add(new MercuryDebugMemorySpace(_core, MercuryCore.SpaceCpuBus, "CPUBUS", true, true));
        }

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => _spaces;

        private IDebugMemorySpace? FindSpace(string name)
        {
            foreach (var space in _spaces)
            {
                if (string.Equals(space.Name, name, StringComparison.OrdinalIgnoreCase)) return space;
            }
            return null;
        }

        private IReadOnlyList<DebugRegisterValue> ReadCpuRegistersLive()
        {
            var cpu = _core.Cpu;
            if (cpu is null) return Array.Empty<DebugRegisterValue>();

            return new[]
            {
                new DebugRegisterValue("A", cpu.A, 8),
                new DebugRegisterValue("F", cpu.F, 8),
                new DebugRegisterValue("BC", cpu.BC, 16),
                new DebugRegisterValue("DE", cpu.DE, 16),
                new DebugRegisterValue("HL", cpu.HL, 16),
                new DebugRegisterValue("SP", cpu.SP, 16),
                new DebugRegisterValue("PC", cpu.PC, 16),
                new DebugRegisterValue("Z", (ulong)((cpu.F & Cpu.Core.Cpu.FlagZ) != 0 ? 1 : 0), 1),
                new DebugRegisterValue("N", (ulong)((cpu.F & Cpu.Core.Cpu.FlagN) != 0 ? 1 : 0), 1),
                new DebugRegisterValue("H", (ulong)((cpu.F & Cpu.Core.Cpu.FlagH) != 0 ? 1 : 0), 1),
                new DebugRegisterValue("C", (ulong)((cpu.F & Cpu.Core.Cpu.FlagC) != 0 ? 1 : 0), 1),
                new DebugRegisterValue("IME", (ulong)(cpu.Ime ? 1 : 0), 1),
                new DebugRegisterValue("HALT", (ulong)(cpu.Halted ? 1 : 0), 1),
                new DebugRegisterValue("IE", _core.Bus?.InterruptEnable ?? 0, 8),
                new DebugRegisterValue("IF", _core.Bus?.InterruptFlags ?? 0, 8),
                new DebugRegisterValue("Cycles", (ulong)cpu.Cycles, 64),
            };
        }

        private IReadOnlyList<DebugRegisterValue> ReadVideoRegistersLive()
        {
            var bus = _core.Bus;
            if (bus is null) return Array.Empty<DebugRegisterValue>();

            var ppu = bus.Ppu;
            var values = new List<DebugRegisterValue>
            {
                new("LCDC", ppu.Lcdc, 8),
                new("STAT", ppu.ReadStat(), 8),
                new("SCY", ppu.Scy, 8),
                new("SCX", ppu.Scx, 8),
                new("LY", ppu.Ly, 8),
                new("LYC", ppu.Lyc, 8),
                new("BGP", ppu.Bgp, 8),
                new("OBP0", ppu.Obp0, 8),
                new("OBP1", ppu.Obp1, 8),
                new("WY", ppu.Wy, 8),
                new("WX", ppu.Wx, 8),
                new("Mode", (ulong)(int)ppu.Mode, 2),
                new("Dot", (ulong)ppu.Dot, 16),
                new("WindowLine", (ulong)ppu.WindowLine, 8),
            };

            if (bus.Cgb)
            {
                values.Add(new DebugRegisterValue("VBK", (ulong)bus.VramBank, 1));
                values.Add(new DebugRegisterValue("SVBK", (ulong)bus.WramBank, 3));
                values.Add(new DebugRegisterValue("BCPS", ppu.ReadBgPaletteIndex(), 8));
                values.Add(new DebugRegisterValue("OCPS", ppu.ReadObjPaletteIndex(), 8));
                values.Add(new DebugRegisterValue("KEY1", (ulong)(bus.DoubleSpeed ? 0x80 : 0x00), 8));
            }

            return values;
        }

        // No APU yet, so this window carries the timer and the board instead of silence - see Mercury_Debug.md §3.
        private IReadOnlyList<DebugRegisterValue> ReadApuRegistersLive()
        {
            var bus = _core.Bus;
            if (bus is null) return Array.Empty<DebugRegisterValue>();

            var values = new List<DebugRegisterValue>
            {
                new("DIV", bus.Div, 8),
                new("TIMA", bus.Tima, 8),
                new("TMA", bus.Tma, 8),
                new("TAC", bus.Tac, 8),
            };

            if (_core.Cart is { } cart)
            {
                values.Add(new DebugRegisterValue($"Board:{cart.Mapper.Name}", 0, 8));
            }

            return values;
        }

        private IReadOnlyList<DebugSpriteInfo> ReadSpritesLive()
        {
            var bus = _core.Bus;
            if (bus is null) return Array.Empty<DebugSpriteInfo>();

            var ppu = bus.Ppu;
            int height = ppu.SpriteHeight;
            var sprites = new List<DebugSpriteInfo>();

            for (int i = 0; i < Ppu.SpriteCount; i++)
            {
                int y = bus.Oam[i * 4] - 16;
                int x = bus.Oam[(i * 4) + 1] - 8;
                byte tile = bus.Oam[(i * 4) + 2];
                byte attributes = bus.Oam[(i * 4) + 3];

                // Parking a sprite off the top or bottom is the standard way to hide one.
                if (y <= -height || y >= Ppu.ScreenHeight) continue;

                int palette = bus.Cgb ? attributes & 0x07 : (attributes & 0x10) != 0 ? 1 : 0;

                sprites.Add(new DebugSpriteInfo(
                    i,
                    x,
                    y,
                    8,
                    height,
                    tile,
                    palette,
                    (attributes & 0x80) != 0 ? 0 : 1,
                    (attributes & 0x20) != 0,
                    (attributes & 0x40) != 0));
            }

            return sprites;
        }

        // Four DMG palettes of four greys, or sixteen colour ones - see Mercury_Debug.md §3.1.
        private IReadOnlyList<DebugPaletteInfo> ReadPalettesLive()
        {
            var bus = _core.Bus;
            if (bus is null) return Array.Empty<DebugPaletteInfo>();

            var ppu = bus.Ppu;
            var palettes = new List<DebugPaletteInfo>();

            if (bus.Cgb)
            {
                for (int p = 0; p < 8; p++) palettes.Add(ColorPalette(p, ppu.BgPaletteRam, p));
                for (int p = 0; p < 8; p++) palettes.Add(ColorPalette(8 + p, ppu.ObjPaletteRam, p));
                return palettes;
            }

            palettes.Add(ShadePalette(0, ppu.Bgp));
            palettes.Add(ShadePalette(1, ppu.Obp0));
            palettes.Add(ShadePalette(2, ppu.Obp1));
            return palettes;
        }

        private static DebugPaletteInfo ShadePalette(int index, byte register)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);

            for (int c = 0; c < 4; c++)
            {
                int shade = (register >> (c * 2)) & 0x03;
                int offset = shade * 3;
                colors.Add((Ppu.DmgShades[offset], Ppu.DmgShades[offset + 1], Ppu.DmgShades[offset + 2]));
            }

            return new DebugPaletteInfo(index, colors);
        }

        private static DebugPaletteInfo ColorPalette(int index, byte[] paletteRam, int palette)
        {
            var colors = new List<(byte r, byte g, byte b)>(4);

            for (int c = 0; c < 4; c++)
            {
                int entry = (palette * 8) + (c * 2);
                int rgb555 = paletteRam[entry] | (paletteRam[entry + 1] << 8);
                colors.Add((Expand(rgb555 & 0x1F), Expand((rgb555 >> 5) & 0x1F), Expand((rgb555 >> 10) & 0x1F)));
            }

            return new DebugPaletteInfo(index, colors);
        }

        private static byte Expand(int channel) => (byte)((channel << 3) | (channel >> 2));

        // Nothing to mute until Phase C exists - see Mercury_Gameplan.md §3.
        public void SetChannelMuted(int index, bool muted) { }

        public (short[] Samples, int SampleRate) GetAudioSamples() =>
            (Array.Empty<short>(), _core.AudioSampleRate);

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

                result.Add(new DisassembledInstruction(
                    pc,
                    bytes,
                    Sm83Disassembler.MnemonicOf(opcode, low),
                    Sm83Disassembler.FormatOperand(opcode, pc, low, high)));

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
            if (_core.Bus is not { Cgb: true }) return $"${tile:X2}";

            byte attributes = space.Read(address + Ppu.VramBankStride);
            string flips = $"{((attributes & 0x20) != 0 ? "X" : "-")}{((attributes & 0x40) != 0 ? "Y" : "-")}";

            return $"${tile:X2} p{attributes & 0x07} b{(attributes >> 3) & 0x01} {flips}{((attributes & 0x80) != 0 ? " pri" : "")}";
        }

        // 16 bytes per tile, but interleaved by row rather than split into two planes the way an NES tile is.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2)
            {
                throw new ArgumentException($"A Game Boy tile is always 2bpp; got {bpp}.", nameof(bpp));
            }

            var pixels = new byte[64];

            for (int row = 0; row < 8; row++)
            {
                byte low = space.Read(address + (row * 2));
                byte high = space.Read(address + (row * 2) + 1);

                for (int column = 0; column < 8; column++)
                {
                    int bit = 7 - column;
                    pixels[(row * 8) + column] =
                        (byte)(((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1));
                }
            }

            return pixels;
        }

        // Grey by pixel value: a tile has no palette of its own until a map entry picks one.
        public (byte[] Rgba, int Width, int Height) RenderTileSheet()
        {
            var bus = _core.Bus;
            if (bus is null) return (Array.Empty<byte>(), 0, 0);

            const int tilesPerRow = 16;
            int tileCount = bus.Vram.Length / 16;
            int rows = (tileCount + tilesPerRow - 1) / tilesPerRow;

            int width = tilesPerRow * 8;
            int height = rows * 8;
            var rgba = new byte[width * height * 4];

            for (int tile = 0; tile < tileCount; tile++)
            {
                int originX = (tile % tilesPerRow) * 8;
                int originY = (tile / tilesPerRow) * 8;

                for (int row = 0; row < 8; row++)
                {
                    byte low = bus.Vram[(tile * 16) + (row * 2)];
                    byte high = bus.Vram[(tile * 16) + (row * 2) + 1];

                    for (int column = 0; column < 8; column++)
                    {
                        int bit = 7 - column;
                        int value = ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1);
                        int shade = value * 3;
                        int offset = (((originY + row) * width) + originX + column) * 4;

                        rgba[offset] = Ppu.DmgShades[shade];
                        rgba[offset + 1] = Ppu.DmgShades[shade + 1];
                        rgba[offset + 2] = Ppu.DmgShades[shade + 2];
                        rgba[offset + 3] = 0xFF;
                    }
                }
            }

            return (rgba, width, height);
        }

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch()
        {
            var palettes = ReadPalettesLive();
            if (palettes.Count == 0) return (Array.Empty<byte>(), 0, 0);

            const int swatch = 16;
            const int columns = 4;

            int width = columns * swatch;
            int height = palettes.Count * swatch;
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
                            rgba[offset] = r;
                            rgba[offset + 1] = g;
                            rgba[offset + 2] = b;
                            rgba[offset + 3] = 0xFF;
                        }
                    }
                }
            }

            return (rgba, width, height);
        }

        // Only the ranges whose destination does not depend on live mapper or bank state.
        public PhysicalAddress? ResolvePhysical(int cpuAddress)
        {
            var bus = _core.Bus;
            if (bus is null) return null;

            if (cpuAddress < 0x4000) return new PhysicalAddress(MercuryCore.SpaceRom, cpuAddress);
            if (cpuAddress < 0x8000) return null;
            if (cpuAddress < 0xA000) return new PhysicalAddress(MercuryCore.SpaceVram, (bus.VramBank * MemoryBus.VramBankSize) + (cpuAddress - 0x8000));
            if (cpuAddress < 0xC000) return null;
            if (cpuAddress < 0xD000) return new PhysicalAddress(MercuryCore.SpaceWram, cpuAddress - 0xC000);
            if (cpuAddress < 0xE000) return new PhysicalAddress(MercuryCore.SpaceWram, (bus.WramBank * MemoryBus.WramBankSize) + (cpuAddress - 0xD000));
            if (cpuAddress < 0xFE00) return null;
            if (cpuAddress < 0xFEA0) return new PhysicalAddress(MercuryCore.SpaceOam, cpuAddress - 0xFE00);
            if (cpuAddress < 0xFF00) return new PhysicalAddress("PROHIBITED", cpuAddress - 0xFEA0, false);
            if (cpuAddress < 0xFF80) return new PhysicalAddress("IOREG", cpuAddress - 0xFF00, false);
            if (cpuAddress < 0xFFFF) return new PhysicalAddress(MercuryCore.SpaceHram, cpuAddress - 0xFF80);

            return new PhysicalAddress("IE", 0, false);
        }

        public string GetSummaryText()
        {
            var cart = _core.Cart;
            var cpu = _core.Cpu;
            var bus = _core.Bus;

            if (cart is null || cpu is null || bus is null) return "No ROM loaded.";

            var ppu = bus.Ppu;
            var text = new StringBuilder();

            text.AppendLine($"{_core.CoreName} (Mercury) - frame {_core.TotalFrames}, LY {ppu.Ly}, mode {(int)ppu.Mode}");
            text.AppendLine($"{cart.Mapper.Name}, {cart.RomBanks}x16K ROM, " +
                            $"{(cart.Ram.Length == 0 ? "no RAM" : $"{cart.Ram.Length / 1024}K RAM")}" +
                            $"{(cart.HasBattery ? ", battery" : "")}{(cart.HasTimer ? ", RTC" : "")}");
            text.AppendLine($"CPU  A={cpu.A:X2} F={cpu.F:X2} BC={cpu.BC:X4} DE={cpu.DE:X4} HL={cpu.HL:X4} " +
                            $"SP={cpu.SP:X4} PC={cpu.PC:X4}");
            text.AppendLine($"     IME={(cpu.Ime ? 1 : 0)} HALT={(cpu.Halted ? 1 : 0)} " +
                            $"IE={bus.InterruptEnable:X2} IF={bus.InterruptFlags:X2}");
            text.AppendLine($"PPU  LCDC={ppu.Lcdc:X2} STAT={ppu.ReadStat():X2} SCY={ppu.Scy:X2} SCX={ppu.Scx:X2} " +
                            $"WY={ppu.Wy:X2} WX={ppu.Wx:X2}");

            if (bus.Cgb)
            {
                text.AppendLine($"CGB  VBK={bus.VramBank} SVBK={bus.WramBank} " +
                                $"speed={(bus.DoubleSpeed ? "double" : "normal")} " +
                                $"HDMA={(bus.HdmaBlocksLeft == 0 ? "idle" : $"{bus.HdmaBlocksLeft} blocks")}");
            }

            text.Append("APU  not built - see Mercury_Gameplan.md phase C");

            return text.ToString();
        }
    }
}
