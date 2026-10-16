using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Mercury;
using EmuSen.Cores.Nintendo.MercuryRT;
using EmuSen.Galaxia.Input;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MercuryRT's machine against the C# Mercury's, state, sound and picture compared frame by frame and instruction by instruction - see Mercury_Native.md §3.3.
    public class MercuryRtMachineTests
    {
        public const string RomsVariable = MercuryRtStateTests.RomsVariable;

        // A directory of blargg and mooneye ROMs, searched recursively; absent, that case passes without running.
        public const string CorpusVariable = HardwareTestRomLibrary.Variable;

        private readonly ITestOutputHelper _output;

        public MercuryRtMachineTests(ITestOutputHelper output) => _output = output;

        internal static readonly byte[] Busy = MercuryPrograms.Busy;

        internal static byte[] InterruptsRom(byte kind, byte ramCode, byte cgb) => MercuryPrograms.InterruptsRom(kind, ramCode, cgb);

        internal static byte[] ColourRom() => MercuryPrograms.ColourRom();

        public static TheoryData<byte, byte, byte> Boards => MercuryRtStateTests.Boards;

        [Theory]
        [MemberData(nameof(Boards))]
        public void A_busy_program_runs_identically_on_every_board(byte kind, byte ramCode, byte cgb)
        {
            var pair = new MercuryRtPair(SyntheticGbRom.Build(romBanks: 4, cartridgeType: kind, ramSizeCode: ramCode, cgbFlag: cgb, patches: (0, Busy)), skipRendering: false);
            pair.Run(600, null);
            _output.WriteLine($"type ${kind:X2} cgb ${cgb:X2}: {pair.Summary}");
        }

        [Theory]
        [MemberData(nameof(Boards))]
        public void Interrupts_halt_dma_and_the_window_run_identically(byte kind, byte ramCode, byte cgb)
        {
            var pair = new MercuryRtPair(InterruptsRom(kind, ramCode, cgb), skipRendering: false);
            pair.Run(600, null);
            Assert.True(pair.Serial > 100, $"the vblank handler ran {pair.Serial} times");
            _output.WriteLine($"type ${kind:X2} cgb ${cgb:X2}: {pair.Summary}");
        }

        [Fact]
        public void Hdma_both_modes_and_double_speed_run_identically()
        {
            var pair = new MercuryRtPair(ColourRom(), skipRendering: false);
            pair.Run(600, null);
            Assert.True(pair.Csharp.Bus!.DoubleSpeed, "the program never reached double speed");
            _output.WriteLine(pair.Summary);
        }

        // The pictures are compared with rendering skipped too; only the colour program draws anything a skip that rendered would change (§8.7.4).
        [Theory]
        [InlineData(false, false)]
        [InlineData(true, false)]
        [InlineData(true, true)]
        public void Rendering_skipped_or_not_leaves_the_same_machine(bool skip, bool colour)
        {
            var pair = new MercuryRtPair(colour ? ColourRom() : InterruptsRom(0x03, 0x02, 0x80), skipRendering: skip);
            pair.Run(300, null);
        }

        // An illegal opcode in a plain frame stops both engines with C#'s message, the opcode and pc rebuilt from the frame's detail word (§8.7.1).
        [Fact]
        public void An_illegal_opcode_in_a_plain_frame_stops_both_engines_alike()
        {
            var pair = new MercuryRtPair(SyntheticGbRom.Build(patches: (0, new byte[] { 0x00, 0x00, 0xD3 })), skipRendering: false);
            Assert.False(pair.Frame(null, 0), "the frame ran past $D3");
        }

        // Each channel muted alone on both engines, the sound compared every frame; the busy program keys pulse 1 (§8.7.4).
        [Fact]
        public void Each_channel_mutes_the_same_channel_on_both_engines()
        {
            for (int channel = 0; channel < 4; channel++)
            {
                var pair = new MercuryRtPair(SyntheticGbRom.Build(patches: (0, Busy)), skipRendering: false);
                pair.Csharp.Bus!.Apu.SetChannelMuted(channel, true);
                pair.Rust.SetMutes(1u << channel);
                pair.Run(30, null);
            }
        }

        // HALT with IME clear and an interrupt pending, the HALT bug's one case, then EI's one-instruction delay into a pending interrupt - mutant M3 survived without it (§8.4).
        [Fact]
        public void The_halt_bug_and_the_ei_delay_step_identically()
        {
            byte[] rom = MercuryPrograms.HaltBugRom();
            var pair = new MercuryRtPair(rom, skipRendering: false);
            for (int i = 0; i < 40; i++) Assert.True(pair.Step());
            Assert.Equal(2, pair.Csharp.Bus!.Wram[0]);
            Assert.Equal(1, pair.Csharp.Bus!.Wram[1]);
        }

        // The window's line counter is back to 0 at every frame's end, so only a comparison inside the frame sees it counted with rendering skipped - mutant M12 (§8.4).
        [Fact]
        public void The_window_counts_its_lines_with_rendering_skipped_instruction_by_instruction()
        {
            var pair = new MercuryRtPair(InterruptsRom(0x03, 0x02, 0x00), skipRendering: true);
            int windowLines = 0;
            for (int i = 0; i < 20000; i++)
            {
                Assert.True(pair.Step());
                windowLines = Math.Max(windowLines, pair.Csharp.Bus!.Ppu.WindowLine);
            }
            Assert.True(windowLines > 50, $"the window drew {windowLines} lines");
        }

        // An OAM DMA started a T-cycle at a time across line 0's composite, so one of its bytes lands on the dot the sprites are read - mutants O2 (§12.4) and M7 (§8.4).
        [Fact]
        public void An_oam_dma_byte_landing_on_the_composite_dot_draws_identically()
        {
            const int composite = EmuSen.Cores.Nintendo.Mercury.Video.Ppu.OamScanCycles + EmuSen.Cores.Nintendo.Mercury.Video.Ppu.BaseDrawingCycles;
            var drawn = new HashSet<string>();
            for (int start = composite - 40; start <= composite; start++)
            {
                var pictures = new List<byte[]>();
                foreach (MercuryEngine engine in Enum.GetValues<MercuryEngine>())
                {
                    using MercuryRig core = MercuryRig.Create(engine, SyntheticGbRom.Build());
                    for (int row = 0; row < 16; row++) core.Bus.Vram[16 + row] = 0xFF;
                    for (int slot = 0; slot < 4; slot++) core.Bus.Oam[slot * 4 + 0] = 16;
                    for (int i = 0; i < 16; i++) core.Bus.Wram[i] = (byte)(i % 4 == 0 ? 16 : i % 4 == 1 ? 8 + 32 * (i / 4) + 16 : i % 4 == 2 ? 1 : 0);
                    core.Write(0xFF48, 0xE4);
                    core.Write(0xFF40, 0x93);
                    core.Tick(start);
                    core.Write(0xFF46, 0xC0);
                    core.Tick(EmuSen.Cores.Nintendo.Mercury.Video.Ppu.CyclesPerScanline);
                    pictures.Add(core.GetFrameBufferRgba()[..(160 * 4)]);
                }
                int same = pictures[0].AsSpan().CommonPrefixLength(pictures[1]);
                Assert.True(same == pictures[0].Length, $"a DMA started at dot {start}: line 0 differs first at x {same / 4}");
                drawn.Add(Convert.ToHexString(pictures[0]));
            }
            Assert.True(drawn.Count > 2, $"only {drawn.Count} distinct lines: no DMA byte landed while the line was composited");
        }

        // On a colour console, map attributes with the priority bit over a solid tile, and a black sprite the background must hide - mutant M14 (§8.4).
        [Fact]
        public void The_colour_background_priority_bit_hides_a_sprite_identically()
        {
            byte[] rom = MercuryPrograms.ColourPriorityRom();
            var pair = new MercuryRtPair(rom, skipRendering: false);
            pair.Run(30, null);
            byte[] picture = pair.Csharp.GetFrameBufferRgba();
            int sprite = (16 * 160 + 24) * 4;
            Assert.True(pair.Csharp.Bus!.Oam[1] == 0x20 && picture[sprite] == 0xFF, $"the sprite's pixel is {picture[sprite]:X2}, so the priority bit was not exercised");
        }

        // A sweep whose first step fits in 11 bits and whose check of the next does not, so only the second check silences the channel - mutant M15 (§8.4).
        [Fact]
        public void The_sweeps_second_overflow_check_silences_identically()
        {
            byte[] rom = MercuryPrograms.SweepOverflowRom();
            var pair = new MercuryRtPair(rom, skipRendering: false);
            pair.Run(10, null);
            var pulse = pair.Csharp.Bus!.Apu.Pulse1;
            Assert.True(pulse.Frequency == 1950 && !pulse.Enabled, $"frequency {pulse.Frequency}, enabled {pulse.Enabled}: the second check was not what silenced it");
        }

        // MBC1 and MBC3 turn a written bank 0 into bank 1; MBC5 does not, and MBC2's register sits behind address bit 8 - mutant M18 (§8.4).
        [Theory]
        [InlineData(0x01, 0x42)]
        [InlineData(0x0F, 0x42)]
        [InlineData(0x19, 0x00)]
        [InlineData(0x05, 0x42)]
        public void A_written_bank_zero_selects_what_each_board_selects_identically(byte kind, byte expected)
        {
            byte[] rom = MercuryPrograms.BankZeroRom(kind);
            var pair = new MercuryRtPair(rom, skipRendering: false);
            pair.Run(2, null);
            Assert.Equal(expected, pair.Csharp.Bus!.Wram[0]);
        }

        // MBC3 copies its clock only on a 0 then a 1, so a lone 1 must leave the latched seconds as they were - mutant M19 (§8.4).
        [Fact]
        public void The_mbc3_clock_latches_only_on_zero_then_one_identically()
        {
            byte[] rom = MercuryPrograms.Mbc3LatchRom();
            var pair = new MercuryRtPair(rom, skipRendering: false);
            pair.Run(2, null);
            Assert.Equal(0, pair.Csharp.Bus!.Wram[0]);
            Assert.Equal(5, pair.Csharp.Bus!.Wram[1]);
        }

        // An OAM DMA from WRAM watched instruction by instruction, so where each byte lands is in every comparison - mutant M7 survived frame-level runs (§8.4).
        [Fact]
        public void An_oam_dma_steps_identically_byte_by_byte()
        {
            byte[] rom = MercuryPrograms.OamDmaRom();
            var pair = new MercuryRtPair(rom, skipRendering: false);
            for (int i = 0; i < 3000; i++) Assert.True(pair.Step());
            Assert.Equal(0xA0, pair.Csharp.Bus!.Oam[0]);
        }
    }

    // Random programs on MercuryRT and the C# Mercury, compared instruction by instruction - see Mercury_Native.md §8.2; a class of its own so it runs beside the others.
    public class MercuryRtRandomProgramTests
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtRandomProgramTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<byte, byte, byte> Boards => MercuryRtStateTests.Boards;

        // Random bytes as code on every board, the state compared after every instruction; an illegal opcode must stop both.
        [Theory]
        [MemberData(nameof(Boards))]
        public void Random_programs_step_identically_instruction_by_instruction(byte kind, byte ramCode, byte cgb)
        {
            int steps = 0, illegal = 0;
            for (int seed = 0; seed < 16; seed++)
            {
                byte[] rom = MercuryPrograms.RandomRom(kind, ramCode, cgb, seed);
                var pair = new MercuryRtPair(rom, skipRendering: false);
                for (int i = 0; i < 5000; i++, steps++)
                {
                    if (!pair.Step()) { illegal++; break; }
                }
            }
            _output.WriteLine($"type ${kind:X2} cgb ${cgb:X2}: {steps} instructions identical, {illegal} of 16 programs stopped on an illegal opcode in both");
        }

    }

    // Real cartridges on both engines, frame by frame - see Mercury_Native.md §8.2.
    public class MercuryRtGameTests
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtGameTests(ITestOutputHelper output) => _output = output;

        public const string RomsVariable = MercuryRtStateTests.RomsVariable;

        [Fact]
        public void Real_games_run_identically_from_boot_and_from_a_transferred_state()
        {
            string? folder = Environment.GetEnvironmentVariable(RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{RomsVariable} unset, not run");
                return;
            }
            foreach (string path in Directory.GetFiles(folder, "*.gb*").Order(StringComparer.Ordinal))
            {
                byte[] rom = File.ReadAllBytes(path);
                var pair = new MercuryRtPair(rom, skipRendering: false);
                pair.Run(3000, Script);
                _output.WriteLine($"{Path.GetFileName(path)} from boot: {pair.Summary}");

                var later = new MercuryRtPair(rom, skipRendering: false, transferAt: pair.Csharp);
                later.Run(600, f => Script(f + 3000));
                _output.WriteLine($"{Path.GetFileName(path)} from its frame-3000 state: {later.Summary}");
            }
        }

        private static uint? Script(int frame) => (frame % 90) switch
        {
            >= 0 and < 5 => 1u << 7,
            >= 45 and < 50 => 1u << 4,
            _ => 0u,
        };

    }

    // The hardware corpus on both engines, ROM by ROM - see Mercury_Native.md §8.2.
    public class MercuryRtCorpusTests
    {
        private readonly ITestOutputHelper _output;

        public MercuryRtCorpusTests(ITestOutputHelper output) => _output = output;

        public const string CorpusVariable = MercuryRtMachineTests.CorpusVariable;

        // Every corpus ROM to its verdict or 3600 frames; serial every frame, state every 30 frames and at the end.
        [Fact]
        public void The_hardware_corpus_runs_identically_rom_by_rom()
        {
            string? folder = Environment.GetEnvironmentVariable(CorpusVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{CorpusVariable} unset, not run");
                return;
            }
            var files = Directory.GetFiles(folder, "*.gb*", SearchOption.AllDirectories).Where(f => f.EndsWith(".gb") || f.EndsWith(".gbc")).Order(StringComparer.Ordinal).ToList();
            int frames = 0;
            foreach (string path in files)
            {
                var pair = new MercuryRtPair(File.ReadAllBytes(path), skipRendering: true, stateEvery: 30, soundAndPicture: false);
                for (int f = 0; f < 3600; f++, frames++)
                {
                    if (!pair.Frame(null, f)) break;
                    if (HardwareTestRomLibrary.ReadVerdict(pair.Csharp.Bus!.SerialLog) != TestRomVerdict.NoVerdict) break;
                    if (pair.Csharp.Cart!.Ram is { Length: >= 4 } ram && ram[1] == 0xDE && ram[2] == 0xB0 && ram[3] == 0x61 && ram[0] != 0x80) break;
                }
                pair.CompareState("the end");
            }
            _output.WriteLine($"{files.Count} ROMs, {frames} frames identical in serial every frame and in state every 30 frames and at the end");
        }

    }

    internal sealed class MercuryRtPair
    {
        private static readonly PadButton[] Order = { PadButton.Right, PadButton.Left, PadButton.Up, PadButton.Down, PadButton.A, PadButton.B, PadButton.Select, PadButton.Start };

        public readonly MercuryCore Csharp;
        public readonly MercuryMachine Rust;
        private readonly bool _skip;
        private readonly int _stateEvery;
        private readonly bool _soundAndPicture;
        private readonly byte[] _frame = new byte[MercuryMachine.FrameBytes];
        private long _samples;
        private int _frames;
        private string? _layout;

        public int Serial => Csharp.Bus!.SerialLog.Count;

        public string Summary => $"{_frames} frames identical in state{(_soundAndPicture ? ", sound and picture" : "")}, {_samples} samples, {Serial} serial bytes, {Csharp.Cpu!.Cycles} CPU cycles";

        public MercuryRtPair(byte[] rom, bool skipRendering, MercuryCore? transferAt = null, int stateEvery = 1, bool soundAndPicture = true, byte[]? state = null, GbModel model = GbModel.Auto)
        {
            Assert.True(MercuryMachine.Available, MercuryNative.Report);
            CoreOptions.BatteryRamDisabled = true;
            _skip = skipRendering;
            _stateEvery = stateEvery;
            _soundAndPicture = soundAndPicture;
            Csharp = Load(rom, model);
            Csharp.SkipRendering = skipRendering;
            Rust = new MercuryMachine(rom, model);
            Rust.SetOptions(skipRendering);
            if (transferAt is not null)
            {
                using var stream = new MemoryStream();
                transferAt.SaveState(stream);
                state = stream.ToArray();
            }
            if (state is not null)
            {
                // Both fresh, so the fields no state carries start equal (§3.1).
                Csharp.LoadState(new MemoryStream(state));
                Rust.Load(state);
            }
            CompareState("load");
        }

        private static MercuryCore Load(byte[] rom, GbModel model)
        {
            string path = SyntheticGbRom.WriteTemp(rom);
            try
            {
                var core = new MercuryCore { Model = model };
                core.LoadRom(path);
                return core;
            }
            finally
            {
                File.Delete(path);
            }
        }

        public void Run(int frames, Func<int, uint?>? script)
        {
            for (int f = 0; f < frames; f++) Assert.True(Frame(script, f), $"frame {f}: both stopped on an illegal opcode, which a frame run here does not expect");
        }

        // False when both engines stopped on the same illegal opcode.
        public bool Frame(Func<int, uint?>? script, int f)
        {
            if (script?.Invoke(f) is { } mask)
            {
                for (int b = 0; b < Order.Length; b++) Csharp.SetButton(0, Order[b], (mask & (1u << b)) != 0);
                Rust.SetButtons(mask);
            }

            Exception? csharp = Record(() => Csharp.RunFrame());
            Exception? rust = Record(() => Rust.RunFrame());
            Assert.True(csharp?.Message == rust?.Message, $"frame {_frames}: C# {csharp?.Message ?? "ran"}, Rust {rust?.Message ?? "ran"}");
            _frames++;

            Assert.True(Csharp.Bus!.SerialLog.SequenceEqual(Rust.SerialLog()), $"frame {_frames}: the serial logs differ");
            if (_soundAndPicture)
            {
                short[] a = Csharp.DequeueAudioSamples(int.MaxValue), b = Rust.DrainAudio(int.MaxValue);
                int same = a.AsSpan().CommonPrefixLength(b);
                Assert.True(same == a.Length && a.Length == b.Length, $"frame {_frames}: {a.Length} C# samples, {b.Length} Rust, first difference at {same}");
                _samples += a.Length;
                Rust.CopyFrame(_frame);
                int pixel = Csharp.GetFrameBufferRgba().AsSpan().CommonPrefixLength(_frame);
                Assert.True(pixel == _frame.Length, $"frame {_frames}: the pictures differ first at byte {pixel} (x {pixel / 4 % 160}, y {pixel / 640}){(_skip ? ", rendering skipped" : "")}");
            }
            if (_frames % _stateEvery == 0 || csharp is not null) CompareState($"frame {_frames}");
            return csharp is null;
        }

        // One instruction on each, compared; false when both stopped on the same illegal opcode.
        public bool Step()
        {
            MemoryBusStep(out bool illegal);
            int rust = Rust.Step();
            Assert.True(illegal == (rust == -20), $"C# {(illegal ? "threw" : "stepped")}, Rust returned {rust}");
            if (!illegal) CompareState($"pc ${Csharp.Cpu!.PC:X4}");
            return !illegal;
        }

        private void MemoryBusStep(out bool illegal)
        {
            var bus = Csharp.Bus!;
            illegal = false;
            try
            {
                Csharp.Cpu!.Step(bus.InterruptEnable, bus.InterruptFlags, out int serviced);
                if (serviced >= 0) bus.InterruptFlags &= (byte)~(1 << serviced);
                int stall = bus.TakePendingStall();
                if (stall > 0) bus.Tick(stall);
            }
            catch (NotSupportedException)
            {
                illegal = true;
            }
        }

        private static Exception? Record(Action action)
        {
            try { action(); return null; }
            catch (NotSupportedException e) { return e; }
        }

        public void CompareState(string when)
        {
            using var stream = new MemoryStream();
            Csharp.SaveState(stream);
            byte[] want = stream.ToArray(), got = Rust.Save();
            int first = want.AsSpan().CommonPrefixLength(got);
            if (first == want.Length && want.Length == got.Length) return;
            _layout ??= Rust.Layout();
            string field = _layout.Split('\n').Where(l => l.Length > 0).Select(l => l.Split(' ')).LastOrDefault(p => int.Parse(p[0]) <= first) is { } line ? line[^1] : "?";
            Assert.Fail($"{when}: the states differ first at byte {first}, in {field}: C# {(first < want.Length ? want[first] : -1)}, Rust {(first < got.Length ? got[first] : -1)}");
        }
    }
}
