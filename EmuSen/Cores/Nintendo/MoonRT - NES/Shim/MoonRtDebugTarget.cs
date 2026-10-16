using System;
using System.Collections.Generic;
using System.Text;
using EmuSen.Cores.Native;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Cores.Nintendo.MoonRT
{
    // The NES's IDebugTarget over MoonRT: registers read from its state by its own listing, memory through its spaces, the views from the NES pack - see Moon_Debug.md, Moon_Native.md §9.4.
    public sealed class MoonRtDebugTarget : IDebugTarget
    {
        private readonly MoonRtCore _engine;
        private readonly List<IDebugMemorySpace> _spaces = new();
        private readonly PollingProvider<IReadOnlyList<DebugRegisterValue>> _cpuRegisters, _videoRegisters, _apuRegisters, _coprocessorRegisters;
        private readonly PollingProvider<IReadOnlyList<DebugSpriteInfo>> _sprites;
        private readonly PollingProvider<IReadOnlyList<DebugPaletteInfo>> _palettes;
        private readonly PollingProvider<IReadOnlyList<DebugAudioChannelInfo>> _audioChannels;
        private readonly PollingProvider<IReadOnlyList<DebugLoadInfo>> _hardwareLoad;
        private byte[]? _state;

        public MoonRtDebugTarget(MoonRtCore engine)
        {
            _engine = engine;
            foreach (var (name, writable, sideEffects) in new[] { ("RAM", true, false), ("PRGROM", false, false), ("PRGRAM", true, false), ("CHR", true, false), ("CIRAM", true, false), ("OAM", true, false), ("PALETTE", true, false), ("CPUBUS", true, true) })
                _spaces.Add(new DelegateDebugMemorySpace(name, () => engine.SpaceSize(name), a => engine.ReadSpace(name, a), writable ? (a, v) => engine.WriteSpace(name, a, v) : null, sideEffects));
            _cpuRegisters = Polled(ReadCpuRegisters);
            _videoRegisters = Polled(ReadVideoRegisters);
            _apuRegisters = Polled(ReadApuRegisters);
            _coprocessorRegisters = Polled(() => Array.Empty<DebugRegisterValue>());
            _sprites = new(ReadSprites, ReadSprites());
            _palettes = new(() => NesDebugViews.Instance.Palettes(_spaces), NesDebugViews.Instance.Palettes(_spaces));
            _audioChannels = new(ReadAudioChannels, ReadAudioChannels());
            _hardwareLoad = new(ReadHardwareLoad, ReadHardwareLoad());
        }

        private PollingProvider<IReadOnlyList<DebugRegisterValue>> Polled(Func<IReadOnlyList<DebugRegisterValue>> read) => new(read, read());

        public string CoreName => "NES";
        public WatchRegistry Watches => _engine.Watches;
        public FrameLogRegistry FrameLog => _engine.FrameLog;
        public BreakpointRegistry Breakpoints => _engine.Breakpoints;
        public CheatRegistry Cheats => _engine.Cheats;
        public CoverageRegistry? Coverage => _engine.Coverage;
        public LabelRegistry? Labels => _engine.Labels;
        public CallStackRegistry? CallStack => _engine.CallStack;
        public long FrameCount => _engine.TotalFrames;
        public int MaxSprites => 64;
        public int TilemapEntryStride => NesDebugViews.Instance.TilemapEntryStride;

        // The Apply button, which must not wait for a frame the pause is not running - see EmuSen_Cheats.md §6.
        public void ApplyCheats() => _engine.ApplyCheats();

        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CpuRegisters => _cpuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> VideoRegisters => _videoRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> ApuRegisters => _apuRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorRegisters => _coprocessorRegisters;
        public IRealtimeProvider<IReadOnlyList<DebugSpriteInfo>> Sprites => _sprites;
        public IRealtimeProvider<IReadOnlyList<DebugPaletteInfo>> Palettes => _palettes;
        public IRealtimeProvider<IReadOnlyList<DebugAudioChannelInfo>> AudioChannels => _audioChannels;
        public IRealtimeProvider<IReadOnlyList<DebugLoadInfo>> HardwareLoad => _hardwareLoad;

        public IReadOnlyList<InterruptVector> InterruptVectors => NesDebugViews.Instance.InterruptVectors;

        public IReadOnlyList<DebugCpu> DebugCpus => new[]
        {
            new DebugCpu("cpu", "2A03", _engine.Breakpoints)
            {
                Coverage = _engine.Coverage,
                CallStack = _engine.CallStack,
                CodeSpace = NesSystem.CpuBus,
                ProgramCounter = () => _engine.Pc,
            },
        };

        public void RefreshProviders()
        {
            _state = null;
            _cpuRegisters.Refresh();
            _videoRegisters.Refresh();
            _apuRegisters.Refresh();
            _coprocessorRegisters.Refresh();
            _sprites.Refresh();
            _palettes.Refresh();
            _audioChannels.Refresh();
            _hardwareLoad.Refresh();
        }

        // The machine's state and its listing, taken once per refresh; null with no game.
        private (StateLayout Layout, byte[] State)? Snapshot()
        {
            if (!_engine.IsRomLoaded || _engine.Layout is not { } layout) return null;
            _state ??= _engine.Machine.Save();
            return (layout, _state);
        }

        private IReadOnlyList<DebugRegisterValue> ReadCpuRegisters()
        {
            if (Snapshot() is not (var l, var s)) return Array.Empty<DebugRegisterValue>();
            long p = l.Int(s, "Cpu.P");
            var values = new List<DebugRegisterValue>
            {
                new("A", (ulong)l.Int(s, "Cpu.A"), 8), new("X", (ulong)l.Int(s, "Cpu.X"), 8), new("Y", (ulong)l.Int(s, "Cpu.Y"), 8),
                new("S", (ulong)l.Int(s, "Cpu.S"), 8), new("PC", (ulong)l.Int(s, "Cpu.PC"), 16), new("P", (ulong)p, 8),
            };
            foreach (var (flag, bit) in new[] { ("N", 7), ("V", 6), ("D", 3), ("I", 2), ("Z", 1), ("C", 0) }) values.Add(new(flag, (ulong)((p >> bit) & 1), 1));
            values.Add(new("Cycles", (ulong)l.Int(s, "Cpu.Cycles"), 64));
            return values;
        }

        private IReadOnlyList<DebugRegisterValue> ReadVideoRegisters()
        {
            if (Snapshot() is not (var l, var s)) return Array.Empty<DebugRegisterValue>();
            long status = l.Int(s, "Ppu.Status");
            return new DebugRegisterValue[]
            {
                new("PPUCTRL", (ulong)l.Int(s, "Ppu.Control"), 8), new("PPUMASK", (ulong)l.Int(s, "Ppu.Mask"), 8), new("PPUSTATUS", (ulong)status, 8),
                new("OAMADDR", (ulong)l.Int(s, "Ppu.OamAddress"), 8), new("v", (ulong)l.Int(s, "Ppu.V"), 15), new("t", (ulong)l.Int(s, "Ppu.T"), 15),
                new("x", (ulong)l.Int(s, "Ppu.FineX"), 3), new("w", (ulong)l.Int(s, "Ppu.WriteToggle"), 1), new("Scanline", (ulong)l.Int(s, "Ppu.Scanline"), 16),
                new("VBlank", (ulong)((status >> 7) & 1), 1), new("Sprite0Hit", (ulong)((status >> 6) & 1), 1), new("SpriteOverflow", (ulong)((status >> 5) & 1), 1),
            };
        }

        private IReadOnlyList<DebugRegisterValue> ReadApuRegisters()
        {
            if (Snapshot() is not (var l, var s)) return Array.Empty<DebugRegisterValue>();
            var values = new List<DebugRegisterValue>();
            for (int i = 0; i < l.Count("Apu.Registers"); i++) values.Add(new($"${0x4000 + i:X4}", (ulong)l.Int(s, "Apu.Registers", i), 8));
            values.Add(new("FrameCounter", (ulong)l.Int(s, "Apu.FrameCounter"), 8));
            values.Add(new("FrameIRQ", (ulong)l.Int(s, "Apu.FrameIrqPending"), 1));
            // The cartridge board's own registers ride along here - see Moon_Debug.md §3.1.
            if (_engine.Header is { } header && MoonBoards.For(header.Mapper) is { } board)
            {
                values.Add(new($"Board:{board.Name}", 0, 8));
                foreach (var (name, path, index, bits) in board.Registers) values.Add(new(name, (ulong)l.Int(s, path, index), bits));
            }
            return values;
        }

        private IReadOnlyList<DebugSpriteInfo> ReadSprites()
        {
            if (Snapshot() is not (var l, var s)) return Array.Empty<DebugSpriteInfo>();
            ReadOnlySpan<byte> oam = l.Bytes(s, "Ppu.Oam");
            int height = (l.Int(s, "Ppu.Control") & 0x20) != 0 ? 16 : 8;
            var sprites = new List<DebugSpriteInfo>();
            for (int i = 0; i < 64; i++)
            {
                int y = oam[i * 4], x = oam[i * 4 + 3];
                byte tile = oam[i * 4 + 1], attributes = oam[i * 4 + 2];
                // A sprite parked below the visible area is the standard way to hide one.
                if (y >= MoonRtCore.Height) continue;
                sprites.Add(new DebugSpriteInfo(i, x, y + 1, 8, height, tile, attributes & 0x03, (attributes & 0x20) != 0 ? 0 : 1, (attributes & 0x40) != 0, (attributes & 0x80) != 0));
            }
            return sprites;
        }

        // Level is a 0-100 rescale of whatever stands in for the channel's volume - see Moon_APU.md §5.
        private IReadOnlyList<DebugAudioChannelInfo> ReadAudioChannels()
        {
            if (Snapshot() is not (var l, var s)) return Array.Empty<DebugAudioChannelInfo>();
            long I(string path) => l.Int(s, path);
            bool F(string path) => l.Flag(s, path);
            int Envelope(string channel) => (int)(F($"Apu.{channel}.Envelope.ConstantVolume") ? I($"Apu.{channel}.Envelope.Volume") : I($"Apu.{channel}.Envelope._decay"));

            DebugAudioChannelInfo Pulse(int index, string name, string channel)
            {
                long period = I($"Apu.{channel}.TimerPeriod"), change = period >> (int)I($"Apu.{channel}.SweepShift");
                long target = F($"Apu.{channel}.SweepNegate") ? period - change - (F($"Apu.{channel}._onesComplement") ? 1 : 0) : period + change;
                bool muted = period < 8 || target > 0x7FF;
                long length = I($"Apu.{channel}.LengthCounter");
                int volume = Envelope(channel);
                return new(index, name, length > 0 && !muted, Scale(volume, 15), _engine.IsChannelMuted(index),
                    $"Duty={I($"Apu.{channel}.Duty")} Period={period} Vol={volume} Len={length} Sweep={F($"Apu.{channel}.SweepEnabled")}");
            }

            long triangleLength = I("Apu.Triangle.LengthCounter"), trianglePeriod = I("Apu.Triangle.TimerPeriod"), noiseLength = I("Apu.Noise.LengthCounter");
            int noiseVolume = Envelope("Noise"), dmcOut = (int)I("Apu.Dmc.OutputLevel");
            return new DebugAudioChannelInfo[]
            {
                Pulse(0, "Pulse 1", "Pulse1"),
                Pulse(1, "Pulse 2", "Pulse2"),
                new(2, "Triangle", triangleLength > 0, triangleLength > 0 && trianglePeriod >= 2 ? 100 : 0, _engine.IsChannelMuted(2),
                    $"Period={trianglePeriod} Len={triangleLength} Halt={F("Apu.Triangle.ControlFlag")}"),
                new(3, "Noise", noiseLength > 0, Scale(noiseVolume, 15), _engine.IsChannelMuted(3),
                    $"PeriodIndex={I("Apu.Noise.PeriodIndex")} Vol={noiseVolume} Len={noiseLength} Short={F("Apu.Noise.ShortMode")}"),
                new(4, "DMC", I("Apu.Dmc._bytesRemaining") > 0, Scale(dmcOut, 127), _engine.IsChannelMuted(4),
                    $"Rate={I("Apu.Dmc.RateIndex")} Out={dmcOut} Addr=0x{I("Apu.Dmc.SampleAddress"):X4} Len={I("Apu.Dmc.SampleLength")} Loop={F("Apu.Dmc.Loop")}"),
            };
        }

        private static int Scale(int value, int max) => (int)Math.Round(value / (double)max * 100.0);

        // The whole frame's time is shown as the CPU's, as the machine runs its parts as one - see Moon_Debug.md §3.2.
        private IReadOnlyList<DebugLoadInfo> ReadHardwareLoad()
        {
            double budget = 1000.0 / _engine.FrameRateHz;
            double Percent(double ms) => Math.Min(100.0, ms / budget * 100.0);
            double frame = _engine.IsRomLoaded ? _engine.LastFrameMilliseconds : 0;
            return new[] { new DebugLoadInfo("CPU+APU", Percent(frame), DebugLoadKind.EmulatorCost), new DebugLoadInfo("PPU", Percent(0), DebugLoadKind.EmulatorCost) };
        }

        public void SetChannelMuted(int index, bool muted) => _engine.SetChannelMuted(index, muted);

        // A non-destructive peek, unlike ICore.DequeueAudioSamples - see EmuSen_Audio_Sync.md §7.
        public (short[] Samples, int SampleRate) GetAudioSamples() =>
            _engine.IsRomLoaded ? (_engine.Machine.Core.PeekAudio(), _engine.AudioSampleRate) : (Array.Empty<short>(), _engine.AudioSampleRate);

        public IReadOnlyList<IDebugMemorySpace> GetMemorySpaces() => _spaces;

        public IReadOnlyList<DisassembledInstruction> Disassemble(string spaceName, int address, int count)
        {
            var space = _spaces.Find(s => string.Equals(s.Name, spaceName, StringComparison.OrdinalIgnoreCase));
            if (space is null || space.Size == 0) return Array.Empty<DisassembledInstruction>();
            var result = new List<DisassembledInstruction>();
            int pc = address;
            for (int i = 0; i < count; i++)
            {
                byte opcode = space.Read(pc);
                int length = Nes6502Disassembler.LengthOf(opcode);
                var bytes = new byte[length];
                for (int b = 0; b < length; b++) bytes[b] = space.Read(pc + b);
                result.Add(new DisassembledInstruction(pc, bytes, Nes6502Disassembler.MnemonicOf(opcode),
                    Nes6502Disassembler.FormatOperand(opcode, pc, length > 1 ? bytes[1] : (byte)0, length > 2 ? bytes[2] : (byte)0)));
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
            int operand = length switch { 2 => instr.Bytes[1], 3 => instr.Bytes[1] | (instr.Bytes[2] << 8), _ => -1 };
            return operand < 0 ? null : Nes6502Disassembler.ClassifyStaticReference(opcode, operand);
        }

        public string DecodeTilemapEntry(IDebugMemorySpace space, int address) => NesDebugViews.Instance.DecodeTilemapEntry(space, address);

        public byte[] DecodeTilePixels(IDebugMemorySpace space, int address, int bpp) => NesDebugViews.Instance.DecodeTilePixels(space, address, bpp);

        public (byte[] Rgba, int Width, int Height) RenderTileSheet() => NesDebugViews.Instance.TileSheet(_spaces);

        public (byte[] Rgba, int Width, int Height) RenderPaletteSwatch() => NesDebugViews.Instance.PaletteSwatch(_spaces);

        // Only the ranges whose destination is fixed; a banked $8000+ depends on the board's live state.
        public PhysicalAddress? ResolvePhysical(int cpuAddress)
        {
            if (cpuAddress < 0x2000) return new PhysicalAddress(NesSystem.Ram, cpuAddress & 0x07FF);
            if (cpuAddress < 0x4000) return new PhysicalAddress("PPUREG", cpuAddress & 0x07, false);
            if (cpuAddress < 0x4020) return new PhysicalAddress("APUREG", cpuAddress - 0x4000, false);
            if (cpuAddress < 0x8000) return new PhysicalAddress(NesSystem.PrgRam, cpuAddress & 0x1FFF);
            return null;
        }

        public string GetSummaryText()
        {
            if (Snapshot() is not (var l, var s) || _engine.Header is not { } header) return "No ROM loaded.";
            var board = MoonBoards.For(header.Mapper);
            long I(string path) => l.Int(s, path);
            var text = new StringBuilder();
            text.AppendLine($"NES (MoonRT) - frame {FrameCount}, scanline {I("Ppu.Scanline")}");
            text.AppendLine($"Mapper {header.Mapper} ({board?.Name ?? "?"}), {header.PrgBanks}x16K PRG, " +
                            $"{(header.ChrIsRam ? "8K CHR RAM" : $"{header.ChrBanks}x8K CHR ROM")}, " +
                            $"mirroring {board?.Mirroring(l, s, header.Mirroring) ?? header.Mirroring}{(header.HasBattery ? ", battery" : "")}");
            text.AppendLine($"CPU  A={I("Cpu.A"):X2} X={I("Cpu.X"):X2} Y={I("Cpu.Y"):X2} S={I("Cpu.S"):X2} PC={I("Cpu.PC"):X4} P={I("Cpu.P"):X2}");
            long mask = I("Ppu.Mask"), status = I("Ppu.Status");
            text.AppendLine($"PPU  CTRL={I("Ppu.Control"):X2} MASK={mask:X2} STATUS={status:X2} v={I("Ppu.V"):X4} t={I("Ppu.T"):X4} x={I("Ppu.FineX")}");
            text.AppendLine($"     rendering={((mask & 0x18) != 0 ? "on" : "off")} vblank={(status >> 7) & 1} sprite0={(status >> 6) & 1}");
            text.Append("APU  no synthesis modelled - registers and the frame IRQ only");
            return text.ToString();
        }
    }
}
