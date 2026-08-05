using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cauldron;
using EmuSen.Cores.Nintendo.Moon.Memory;
using EmuSen.Cores.Nintendo.Moon.Processor;
using EmuSen.Cores.Nintendo.Moon.Video;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;

namespace EmuSen.Cores.Nintendo.Moon.Debug
{
    // A space backed by a name the core already knows how to read and write - see Moon_Debug.md §2.
    internal sealed class MoonDebugMemorySpace : IDebugMemorySpace
    {
        private readonly MoonCore _core;
        private readonly string _space;

        public string Name { get; }
        public int Size { get; }
        public bool IsWritable { get; }
        public bool HasSideEffects { get; }

        public MoonDebugMemorySpace(MoonCore core, string space, string name, bool isWritable, bool hasSideEffects)
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

    // The NES's IDebugTarget - the second implementation this interface has ever had. See Moon_Debug.md.
    public sealed class MoonDebugTarget : IDebugTarget, IWriteObserver
    {
        private readonly MoonCore _core;

        private readonly List<IDebugMemorySpace> _spaces = new();

        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _videoRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _apuRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _coprocessorRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites;
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes;
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels;
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad;

        private readonly bool[] _muted = new bool[Apu.Apu.ChannelCount];

        // Injectable only so a test can assert exact bar percentages - see Moon_Debug.md §3.2.
        private readonly Func<(double CpuApuMs, double PpuMs)> _frameTimings;

        public MoonDebugTarget(MoonCore core, Func<(double CpuApuMs, double PpuMs)>? frameTimings = null)
        {
            _core = core;
            _frameTimings = frameTimings ?? (() => (core.LastFrameCpuApuMs, core.LastFramePpuMs));

            BuildSpaces();

            _cpuRegisters = new(ReadCpuRegistersLive, ReadCpuRegistersLive());
            _videoRegisters = new(ReadVideoRegistersLive, ReadVideoRegistersLive());
            _apuRegisters = new(ReadApuRegistersLive, ReadApuRegistersLive());
            _coprocessorRegisters = new(() => Array.Empty<DebugRegisterValue>(), Array.Empty<DebugRegisterValue>());
            _sprites = new(ReadSpritesLive, ReadSpritesLive());
            _palettes = new(ReadPalettesLive, ReadPalettesLive());
            _audioChannels = new(ReadAudioChannelsLive, ReadAudioChannelsLive());
            _hardwareLoad = new(ReadHardwareLoadLive, ReadHardwareLoadLive());

            if (core.Bus != null) core.Bus.WriteObserver = this;
        }

        public string CoreName => "NES";

        public WatchRegistry Watches => _core.Watches;
        public FrameLogRegistry FrameLog => _core.FrameLog;
        public BreakpointRegistry Breakpoints => _core.Breakpoints;
        public CheatRegistry Cheats => _core.Cheats;
        public CoverageRegistry? Coverage => _core.Coverage;
        public LabelRegistry? Labels => _core.Labels;

        public long FrameCount => _core.TotalFrames;

        public int MaxSprites => 64;

        // One byte per nametable cell; the attribute byte is separate and coarser - see Moon_Debug.md §4.
        public int TilemapEntryStride => 1;

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => _cpuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => _videoRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => _apuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => _coprocessorRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _sprites;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _palettes;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _audioChannels;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _hardwareLoad;

        public IReadOnlyList<InterruptVector> InterruptVectors => new[]
        {
            new InterruptVector("NMI", Cpu.NmiVector),
            new InterruptVector("RESET", Cpu.ResetVector),
            new InterruptVector("IRQ/BRK", Cpu.IrqVector),
        };

        public IReadOnlyList<DebugCpu> DebugCpus => new[]
        {
            new DebugCpu("cpu", "2A03", _core.Breakpoints),
        };

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

        // Percent of one native frame's wall-clock budget, clamped so a frame running behind never exceeds "full" - see Moon_Debug.md §3.2.
        private IReadOnlyList<DebugLoadInfo> ReadHardwareLoadLive()
        {
            var (cpuApuMs, ppuMs) = _frameTimings();
            double frameBudgetMs = 1000.0 / _core.FrameRateHz;
            double ToPercent(double ms) => Math.Min(100.0, ms / frameBudgetMs * 100.0);

            return new[]
            {
                new DebugLoadInfo("CPU+APU", ToPercent(cpuApuMs), DebugLoadKind.EmulatorCost),
                new DebugLoadInfo("PPU", ToPercent(ppuMs), DebugLoadKind.EmulatorCost),
            };
        }

        public void OnWrite(string spaceName, int address, byte value) =>
            Watches.RecordWrite(spaceName, address, value, DescribeWriteSite);

        private string DescribeWriteSite() =>
            _core.Cpu is null ? "" : $"PC=${_core.Cpu.LastInstructionPC:X4}";

        private void BuildSpaces()
        {
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpaceRam, "RAM", true, false));
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpacePrgRom, "PRGROM", false, false));
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpacePrgRam, "PRGRAM", true, false));
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpaceChr, "CHR", true, false));
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpaceCiram, "CIRAM", true, false));
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpaceOam, "OAM", true, false));
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpacePalette, "PALETTE", true, false));

            // $2002 clears the vblank flag when read and $2007 advances the address, so this one bites.
            _spaces.Add(new MoonDebugMemorySpace(_core, MoonCore.SpaceCpuBus, "CPUBUS", true, true));
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
                new DebugRegisterValue("X", cpu.X, 8),
                new DebugRegisterValue("Y", cpu.Y, 8),
                new DebugRegisterValue("S", cpu.S, 8),
                new DebugRegisterValue("PC", cpu.PC, 16),
                new DebugRegisterValue("P", cpu.P, 8),
                new DebugRegisterValue("N", (ulong)(cpu.GetFlag(CpuFlags.N) ? 1 : 0), 1),
                new DebugRegisterValue("V", (ulong)(cpu.GetFlag(CpuFlags.V) ? 1 : 0), 1),
                new DebugRegisterValue("D", (ulong)(cpu.GetFlag(CpuFlags.D) ? 1 : 0), 1),
                new DebugRegisterValue("I", (ulong)(cpu.GetFlag(CpuFlags.I) ? 1 : 0), 1),
                new DebugRegisterValue("Z", (ulong)(cpu.GetFlag(CpuFlags.Z) ? 1 : 0), 1),
                new DebugRegisterValue("C", (ulong)(cpu.GetFlag(CpuFlags.C) ? 1 : 0), 1),
                new DebugRegisterValue("Cycles", (ulong)cpu.Cycles, 64),
            };
        }

        private IReadOnlyList<DebugRegisterValue> ReadVideoRegistersLive()
        {
            var ppu = _core.Ppu;
            if (ppu is null) return Array.Empty<DebugRegisterValue>();

            return new[]
            {
                new DebugRegisterValue("PPUCTRL", ppu.Control, 8),
                new DebugRegisterValue("PPUMASK", ppu.Mask, 8),
                new DebugRegisterValue("PPUSTATUS", ppu.Status, 8),
                new DebugRegisterValue("OAMADDR", ppu.OamAddress, 8),
                new DebugRegisterValue("v", ppu.V, 15),
                new DebugRegisterValue("t", ppu.T, 15),
                new DebugRegisterValue("x", ppu.FineX, 3),
                new DebugRegisterValue("w", (ulong)(ppu.WriteToggle ? 1 : 0), 1),
                new DebugRegisterValue("Scanline", (ulong)_core.CurrentScanline, 16),
                new DebugRegisterValue("VBlank", (ulong)(ppu.VBlankFlag ? 1 : 0), 1),
                new DebugRegisterValue("Sprite0Hit", (ulong)(ppu.Sprite0Hit ? 1 : 0), 1),
                new DebugRegisterValue("SpriteOverflow", (ulong)(ppu.SpriteOverflow ? 1 : 0), 1),
            };
        }

        private IReadOnlyList<DebugRegisterValue> ReadApuRegistersLive()
        {
            var apu = _core.Apu;
            if (apu is null) return Array.Empty<DebugRegisterValue>();

            var values = new List<DebugRegisterValue>();
            for (int i = 0; i < apu.Registers.Length; i++)
            {
                values.Add(new DebugRegisterValue($"${0x4000 + i:X4}", apu.Registers[i], 8));
            }

            values.Add(new DebugRegisterValue("FrameCounter", apu.FrameCounter, 8));
            values.Add(new DebugRegisterValue("FrameIRQ", (ulong)(apu.FrameIrqPending ? 1 : 0), 1));

            // The cartridge board's own registers ride along here - see Moon_Debug.md §3.1.
            if (_core.Cart?.Mapper is { } mapper)
            {
                values.Add(new DebugRegisterValue($"Board:{mapper.Name}", 0, 8));
                foreach (var (name, value, bits) in mapper.DebugState)
                {
                    values.Add(new DebugRegisterValue(name, value, bits));
                }
            }

            return values;
        }

        private IReadOnlyList<DebugSpriteInfo> ReadSpritesLive()
        {
            var ppu = _core.Ppu;
            if (ppu is null) return Array.Empty<DebugSpriteInfo>();

            int height = ppu.SpritesAre8x16 ? 16 : 8;
            var sprites = new List<DebugSpriteInfo>();

            for (int i = 0; i < 64; i++)
            {
                int y = ppu.Oam[i * 4];
                byte tile = ppu.Oam[(i * 4) + 1];
                byte attributes = ppu.Oam[(i * 4) + 2];
                int x = ppu.Oam[(i * 4) + 3];

                // A sprite parked below the visible area is the standard way to hide one.
                if (y >= Ppu.ScreenHeight) continue;

                sprites.Add(new DebugSpriteInfo(
                    i,
                    x,
                    y + 1,
                    8,
                    height,
                    tile,
                    attributes & 0x03,
                    (attributes & 0x20) != 0 ? 0 : 1,
                    (attributes & 0x40) != 0,
                    (attributes & 0x80) != 0));
            }

            return sprites;
        }

        private IReadOnlyList<DebugPaletteInfo> ReadPalettesLive()
        {
            var ppu = _core.Ppu;
            if (ppu is null) return Array.Empty<DebugPaletteInfo>();

            var palettes = new List<DebugPaletteInfo>();

            for (int p = 0; p < 8; p++)
            {
                var colors = new List<(byte r, byte g, byte b)>();
                for (int c = 0; c < 4; c++)
                {
                    int entry = (p * 4) + c;
                    byte nesColor = ppu.PaletteRam[Ppu.PaletteOffset((ushort)(0x3F00 + entry))];
                    int offset = (nesColor & 0x3F) * 3;
                    colors.Add((Ppu.NesPalette[offset], Ppu.NesPalette[offset + 1], Ppu.NesPalette[offset + 2]));
                }
                palettes.Add(new DebugPaletteInfo(p, colors));
            }

            return palettes;
        }

        private IReadOnlyList<DebugAudioChannelInfo> ReadAudioChannelsLive()
        {
            var apu = _core.Apu;
            if (apu is null) return Array.Empty<DebugAudioChannelInfo>();

            string[] names = { "Pulse 1", "Pulse 2", "Triangle", "Noise", "DMC" };
            var channels = new List<DebugAudioChannelInfo>();

            for (int i = 0; i < names.Length; i++)
            {
                channels.Add(new DebugAudioChannelInfo(
                    i,
                    names[i],
                    apu.LengthCounters[i] > 0,
                    0,
                    _muted[i],
                    "no synthesis - see Moon_APU.md"));
            }

            return channels;
        }

        public void SetChannelMuted(int index, bool muted)
        {
            if ((uint)index < (uint)_muted.Length) _muted[index] = muted;
        }

        // Nothing is synthesized yet, so there is never a buffer to snapshot.
        // A non-destructive peek, unlike ICore.DequeueAudioSamples - see EmuSen_Audio_Sync.md §7.
        public (short[] Samples, int SampleRate) GetAudioSamples() =>
            (_core.Apu?.Peek() ?? Array.Empty<short>(), _core.AudioSampleRate);

        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            var space = FindSpace(spaceName);
            if (space is null || space.Size == 0) return Array.Empty<DisassembledInstruction>();

            var result = new List<DisassembledInstruction>();
            int pc = address;

            for (int i = 0; i < count; i++)
            {
                byte opcode = space.Read(pc);
                int length = Nes6502Disassembler.LengthOf(opcode);

                var bytes = new byte[length];
                for (int b = 0; b < length; b++) bytes[b] = space.Read(pc + b);

                byte low = length > 1 ? bytes[1] : (byte)0;
                byte high = length > 2 ? bytes[2] : (byte)0;

                result.Add(new DisassembledInstruction(
                    pc,
                    bytes,
                    Nes6502Disassembler.MnemonicOf(opcode),
                    Nes6502Disassembler.FormatOperand(opcode, pc, low, high)));

                pc += length;
            }

            return result;
        }

        public (StaticReferenceKind Kind, int Target)? ClassifyStaticReference(DisassembledInstruction instr)
        {
            if (instr.Bytes.Count == 0) return null;

            byte opcode = instr.Bytes[0];
            int length = Nes6502Disassembler.LengthOf(opcode);
            if (instr.Bytes.Count < length) return null;

            int operand = length switch
            {
                2 => instr.Bytes[1],
                3 => instr.Bytes[1] | (instr.Bytes[2] << 8),
                _ => -1,
            };

            return operand < 0 ? null : Nes6502Disassembler.ClassifyStaticReference(opcode, operand);
        }

        // Offset 0x3C0 and up in a nametable page is the attribute table, not tiles.
        public string DecodeTilemapEntry(IDebugMemorySpace space, int address)
        {
            int page = (address / 0x400) & 0x03;
            int offset = address % 0x400;

            if (offset >= 0x3C0) return $"attr ${space.Read(address):X2}";

            byte tile = space.Read(address);

            int tileX = offset % 32;
            int tileY = offset / 32;
            byte attribute = space.Read((page * 0x400) + 0x3C0 + ((tileY / 4) * 8) + (tileX / 4));
            int quadrant = ((tileY & 0x02) << 1) | (tileX & 0x02);
            int palette = (attribute >> quadrant) & 0x03;

            return $"${tile:X2} p{palette}";
        }

        // 16 bytes per tile: eight rows of the low bitplane, then eight of the high one.
        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp)
        {
            if (bpp != 2)
            {
                throw new ArgumentException($"An NES tile is always 2bpp; got {bpp}.", nameof(bpp));
            }

            var pixels = new byte[64];

            for (int row = 0; row < 8; row++)
            {
                byte low = space.Read(address + row);
                byte high = space.Read(address + row + 8);

                for (int column = 0; column < 8; column++)
                {
                    int bit = 7 - column;
                    pixels[(row * 8) + column] =
                        (byte)(((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1));
                }
            }

            return pixels;
        }

        // Grayscale by pixel value, since a tile has no palette of its own until a nametable picks one.
        public (byte[] Rgba, int Width, int Height) RenderTileSheet()
        {
            var cart = _core.Cart;
            if (cart is null || cart.Chr.Length == 0) return (Array.Empty<byte>(), 0, 0);

            const int tilesPerRow = 16;
            int tileCount = cart.Chr.Length / 16;
            int rows = (tileCount + tilesPerRow - 1) / tilesPerRow;

            int width = tilesPerRow * 8;
            int height = rows * 8;
            var rgba = new byte[width * height * 4];
            byte[] shades = { 0, 85, 170, 255 };

            for (int tile = 0; tile < tileCount; tile++)
            {
                int originX = (tile % tilesPerRow) * 8;
                int originY = (tile / tilesPerRow) * 8;

                for (int row = 0; row < 8; row++)
                {
                    byte low = cart.Chr[(tile * 16) + row];
                    byte high = cart.Chr[(tile * 16) + row + 8];

                    for (int column = 0; column < 8; column++)
                    {
                        int bit = 7 - column;
                        int value = ((low >> bit) & 0x01) | (((high >> bit) & 0x01) << 1);
                        int offset = (((originY + row) * width) + originX + column) * 4;

                        rgba[offset] = shades[value];
                        rgba[offset + 1] = shades[value];
                        rgba[offset + 2] = shades[value];
                        rgba[offset + 3] = 0xFF;
                    }
                }
            }

            return (rgba, width, height);
        }

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch()
        {
            var ppu = _core.Ppu;
            if (ppu is null) return (Array.Empty<byte>(), 0, 0);

            const int swatch = 16;
            const int columns = 16;
            const int rows = 2;

            int width = columns * swatch;
            int height = rows * swatch;
            var rgba = new byte[width * height * 4];

            for (int entry = 0; entry < 32; entry++)
            {
                byte nesColor = ppu.PaletteRam[Ppu.PaletteOffset((ushort)(0x3F00 + entry))];
                int paletteOffset = (nesColor & 0x3F) * 3;

                int originX = (entry % columns) * swatch;
                int originY = (entry / columns) * swatch;

                for (int y = 0; y < swatch; y++)
                {
                    for (int x = 0; x < swatch; x++)
                    {
                        int offset = (((originY + y) * width) + originX + x) * 4;
                        rgba[offset] = Ppu.NesPalette[paletteOffset];
                        rgba[offset + 1] = Ppu.NesPalette[paletteOffset + 1];
                        rgba[offset + 2] = Ppu.NesPalette[paletteOffset + 2];
                        rgba[offset + 3] = 0xFF;
                    }
                }
            }

            return (rgba, width, height);
        }

        // Only the ranges whose destination is fixed; a banked $8000+ depends on live mapper state.
        public PhysicalAddress? ResolvePhysical(int cpuAddress)
        {
            if (cpuAddress < 0x2000) return new PhysicalAddress(MoonCore.SpaceRam, cpuAddress & 0x07FF);
            if (cpuAddress < 0x4000) return new PhysicalAddress("PPUREG", cpuAddress & 0x07, false);
            if (cpuAddress < 0x4020) return new PhysicalAddress("APUREG", cpuAddress - 0x4000, false);
            if (cpuAddress < 0x8000) return new PhysicalAddress(MoonCore.SpacePrgRam, cpuAddress & 0x1FFF);
            return null;
        }

        public string GetSummaryText()
        {
            var cart = _core.Cart;
            var cpu = _core.Cpu;
            var ppu = _core.Ppu;

            if (cart is null || cpu is null || ppu is null) return "No ROM loaded.";

            var text = new StringBuilder();

            text.AppendLine($"NES (Moon) - frame {_core.TotalFrames}, scanline {_core.CurrentScanline}");
            text.AppendLine($"Mapper {cart.MapperNumber} ({cart.Mapper.Name}), {cart.PrgBanks}x16K PRG, " +
                            $"{(cart.ChrIsRam ? "8K CHR RAM" : $"{cart.ChrBanks}x8K CHR ROM")}, " +
                            $"mirroring {cart.Mapper.Mirroring}{(cart.HasBattery ? ", battery" : "")}");
            text.AppendLine($"CPU  A={cpu.A:X2} X={cpu.X:X2} Y={cpu.Y:X2} S={cpu.S:X2} PC={cpu.PC:X4} P={cpu.P:X2}");
            text.AppendLine($"PPU  CTRL={ppu.Control:X2} MASK={ppu.Mask:X2} STATUS={ppu.Status:X2} " +
                            $"v={ppu.V:X4} t={ppu.T:X4} x={ppu.FineX}");
            text.AppendLine($"     rendering={(ppu.RenderingEnabled ? "on" : "off")} " +
                            $"vblank={(ppu.VBlankFlag ? 1 : 0)} sprite0={(ppu.Sprite0Hit ? 1 : 0)}");
            text.Append("APU  no synthesis modelled - registers and the frame IRQ only");

            return text.ToString();
        }
    }
}
