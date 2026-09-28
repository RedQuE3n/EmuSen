using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Nintendo.Moon;
using EmuSen.Cores.Nintendo.Moon.Input;
using EmuSen.Cores.Nintendo.MoonRT;
using EmuSen.Cores.Nintendo.Moon.Validation;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Cores
{
    // MoonRT's machine against the C# Moon's, state, sound and picture compared frame by frame and instruction by instruction - see Moon_Native.md §3.3.
    public class MoonRtMachineTests
    {
        private readonly ITestOutputHelper _output;

        public MoonRtMachineTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<int, int> Boards => MoonRtStateTests.Boards;

        [Theory]
        [MemberData(nameof(Boards))]
        public void A_busy_program_runs_identically_on_every_board(int mapper, int chrBanks)
        {
            var pair = new MoonRtPair(MoonRtStateTests.Rom(mapper, chrBanks, (0, MoonRtStateTests.Busy)), skipRendering: false);
            pair.Run(600, null);
            _output.WriteLine($"mapper {mapper}, {chrBanks} CHR banks: {pair.Summary}");
        }

        // NMI with OAM DMA and a scroll, 8x16 sprites, all five channels with a looping DMC, the board's banks written every frame and its IRQ armed.
        [Theory]
        [MemberData(nameof(Boards))]
        public void Interrupts_dma_sprites_sound_and_banking_run_identically(int mapper, int chrBanks)
        {
            var pair = new MoonRtPair(BoardPrograms.Build(mapper, chrBanks, cycleIrq: false), skipRendering: false);
            pair.Run(600, Pads);
            Assert.True(pair.Csharp.Bus!.Ram[1] > 0, "the NMI handler never ran");
            _output.WriteLine($"mapper {mapper}, {chrBanks} CHR banks: {pair.Summary}, {pair.Csharp.Bus!.Ram[2]} IRQs (mod 256)");
        }

        // RAMBO-1 with its counter on the CPU clock, the mode MMC3 has no counterpart for.
        [Fact]
        public void Rambo1s_cycle_counted_irq_runs_identically()
        {
            var pair = new MoonRtPair(BoardPrograms.Build(64, 8, cycleIrq: true), skipRendering: false);
            pair.Run(600, Pads);
            Assert.True(pair.Csharp.Bus!.Ram[2] > 0, "the IRQ never fired");
            _output.WriteLine(pair.Summary);
        }

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public void Pixel_writes_skipped_or_not_leave_the_same_machine(bool skip)
        {
            var pair = new MoonRtPair(BoardPrograms.Build(4, 8, cycleIrq: false), skipRendering: skip);
            pair.Run(300, Pads);
        }

        // The program above instruction by instruction, so where each IRQ, NMI and DMA lands inside a frame is in every comparison.
        [Theory]
        [InlineData(4)]
        [InlineData(9)]
        [InlineData(64)]
        [InlineData(69)]
        public void The_board_program_steps_identically_instruction_by_instruction(int mapper)
        {
            var pair = new MoonRtPair(BoardPrograms.Build(mapper, 8, cycleIrq: false), skipRendering: false);
            for (int i = 0; i < 60000; i++) Assert.True(pair.Step() >= 0);
            _output.WriteLine($"mapper {mapper}: 60000 instructions identical");
        }

        // The RESET button, twice, mid-program: what it keeps and what it clears must agree field by field.
        [Fact]
        public void A_reset_mid_program_leaves_identical_machines()
        {
            var pair = new MoonRtPair(BoardPrograms.Build(1, 0, cycleIrq: false), skipRendering: false);
            pair.Run(100, Pads);
            pair.Reset();
            pair.Run(100, Pads);
            pair.Reset();
            pair.Run(100, Pads);
        }

        // Both pads, a new pattern every few frames.
        public static (uint, uint)? Pads(int frame) => frame % 7 == 0 ? ((uint)(frame * 37) & 0xFF, (uint)(frame * 91) & 0xFF) : null;
    }

    // Random programs on MoonRT and the C# Moon, compared instruction by instruction - see Moon_Native.md §3.5; a class of its own so it runs beside the others.
    public class MoonRtRandomProgramTests
    {
        private readonly ITestOutputHelper _output;

        public MoonRtRandomProgramTests(ITestOutputHelper output) => _output = output;

        public static TheoryData<int, int> Boards => MoonRtStateTests.Boards;

        // Random bytes as code on every board, the state compared after every instruction; past the first four seeds the JAMs are NOPs.
        [Theory]
        [MemberData(nameof(Boards))]
        public void Random_programs_step_identically_instruction_by_instruction(int mapper, int chrBanks)
        {
            int steps = 0, jammed = 0;
            for (int seed = 0; seed < 12; seed++)
            {
                var noise = new Random(seed * 131 + mapper * 7 + chrBanks);
                byte[] rom = MoonRtStateTests.Rom(mapper, chrBanks);
                noise.NextBytes(rom.AsSpan(16, rom.Length - 16));
                if (seed >= 4)
                    for (int i = 16; i < 16 + 8 * SyntheticNesRom.PrgBankSize; i++)
                        if ((rom[i] & 0x0F) == 0x02 && (rom[i] >> 4) is not (8 or 0xA or 0xC or 0xE)) rom[i] = 0xEA;
                var pair = new MoonRtPair(rom, skipRendering: false);
                for (int i = 0; i < 4000; i++, steps++)
                {
                    if (pair.Step() < 0) break;
                    if (pair.Csharp.Cpu!.Jammed) { jammed++; break; }
                }
            }
            _output.WriteLine($"mapper {mapper}, {chrBanks} CHR banks: {steps} instructions identical, {jammed} of 12 programs jammed in both");
        }
    }

    // Real cartridges on both engines, frame by frame - see Moon_Native.md §3.3.
    public class MoonRtGameTests
    {
        private readonly ITestOutputHelper _output;

        public MoonRtGameTests(ITestOutputHelper output) => _output = output;

        [Fact]
        public void Real_games_run_identically_from_boot_and_from_a_transferred_state()
        {
            string? folder = Environment.GetEnvironmentVariable(MoonRtStateTests.RomsVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{MoonRtStateTests.RomsVariable} unset, not run");
                return;
            }
            foreach (string path in Directory.GetFiles(folder, "*.nes").Order(StringComparer.Ordinal))
            {
                byte[] rom = File.ReadAllBytes(path);
                var pair = new MoonRtPair(rom, skipRendering: false);
                pair.Run(3000, Script);
                _output.WriteLine($"{Path.GetFileName(path)} from boot: {pair.Summary}");

                var later = new MoonRtPair(rom, skipRendering: false, transferAt: pair.Csharp);
                later.Run(600, f => Script(f + 3000));
                _output.WriteLine($"{Path.GetFileName(path)} from its frame-3000 state: {later.Summary}");
            }
        }

        // The bench's Start and A presses on pad 1.
        private static (uint, uint)? Script(int frame) => (frame % 90) switch
        {
            0 => (1u << 3, 0u),
            5 => (0u, 0u),
            45 => (1u << 0, 0u),
            50 => (0u, 0u),
            _ => null,
        };
    }

    // The test ROM corpus on both engines, ROM by ROM, with the runner's reset protocol - see Moon_Native.md §3.4.
    public class MoonRtCorpusTests
    {
        public const string CorpusVariable = "EMUSEN_MOONRT_CORPUS";

        private readonly ITestOutputHelper _output;

        public MoonRtCorpusTests(ITestOutputHelper output) => _output = output;

        [Fact]
        public void The_test_rom_corpus_runs_identically_rom_by_rom()
        {
            string? folder = Environment.GetEnvironmentVariable(CorpusVariable);
            if (folder is null || !Directory.Exists(folder))
            {
                _output.WriteLine($"{CorpusVariable} unset, not run");
                return;
            }
            var files = Directory.GetFiles(folder, "*.nes", SearchOption.AllDirectories).Order(StringComparer.Ordinal).ToList();
            int frames = 0, run = 0, verdicts = 0, passed = 0, unsupported = 0;
            foreach (string path in files)
            {
                byte[] rom = File.ReadAllBytes(path);
                MoonRtPair pair;
                try
                {
                    pair = new MoonRtPair(rom, skipRendering: true, stateEvery: 30, soundAndPicture: false);
                }
                catch (MoonRtPair.BothRefused)
                {
                    unsupported++;
                    continue;
                }
                run++;
                var (status, frame, resets) = RunProtocol(pair, 2400, ref frames);
                pair.CompareState("the end");
                string text = NesTestRomRunner.ReadText(pair.Csharp);
                Assert.Equal(text, pair.RustText());
                if (status >= 0) verdicts++;
                if (status == 0) passed++;
                _output.WriteLine($"{Path.GetRelativePath(folder, path)}: status {status} at frame {frame}, {resets} resets");
            }
            _output.WriteLine($"{run} ROMs run ({unsupported} refused by both), {frames} frames identical; {verdicts} verdicts, {passed} passed");
        }

        // NesTestRomRunner.RunLoaded's loop, deciding from C#'s PRG RAM and resetting both engines.
        private static (int Status, int Frame, int Resets) RunProtocol(MoonRtPair pair, int budget, ref int frames)
        {
            bool signatureSeen = false, runningSeen = false;
            int resetIn = 0, resets = 0;
            for (int frame = 1; frame <= budget; frame++, frames++)
            {
                if (!pair.Frame(null, frame)) return (-2, frame, resets);
                if (resetIn > 0)
                {
                    if (--resetIn > 0) continue;
                    pair.Reset();
                    resets++;
                    runningSeen = false;
                    continue;
                }
                MoonCore core = pair.Csharp;
                if (!signatureSeen)
                {
                    if (core.ReadSpace(MoonCore.SpacePrgRam, 0x6001) != 0xDE || core.ReadSpace(MoonCore.SpacePrgRam, 0x6002) != 0xB0 || core.ReadSpace(MoonCore.SpacePrgRam, 0x6003) != 0x61) continue;
                    signatureSeen = true;
                }
                byte status = core.ReadSpace(MoonCore.SpacePrgRam, 0x6000);
                if (status == 0x80) { runningSeen = true; continue; }
                if (status == 0x81)
                {
                    if (!runningSeen) continue;
                    if (resets >= 4) break;
                    resetIn = 12;
                    continue;
                }
                if (!runningSeen) continue;
                return (status, frame, resets);
            }
            return (-1, budget, resets);
        }
    }

    // Hand-assembled programs that drive each board's registers and IRQ, the code at $C000 in every bank a board might map there.
    public static class BoardPrograms
    {
        private const ushort Nmi = 0xC100, Irq = 0xC180, Palette = 0xC200;

        public static byte[] Build(int mapper, int chrBanks, bool cycleIrq)
        {
            var main = new Asm(0xC000);
            main.Op(0x78).Op(0xD8).Ldx(0xFF).Op(0x9A);
            main.LdaImm(0x00).Sta(0x2000).Sta(0x2001);
            for (int w = 0; w < 2; w++) main.Bit(0x2002).Bpl(-3);
            main.LdaImm(0x3F).Sta(0x2006).LdaImm(0x00).Sta(0x2006).Ldx(0x00);
            main.LdaX(Palette).Sta(0x2007).Op(0xE8).Cpx(0x20).Bne(-9);
            main.Ldx(0x00).Op(0x8A).StaX(0x0200).Op(0xE8).Bne(-5);
            BoardInit(main, mapper, cycleIrq);
            main.LdaImm(0xB8).Sta(0x2000).LdaImm(0x1E).Sta(0x2001);
            foreach (var (address, value) in new (ushort, byte)[]
            {
                (0x4015, 0x0F), (0x4000, 0xBF), (0x4001, 0x8A), (0x4003, 0x40), (0x4004, 0x7F), (0x4006, 0x33), (0x4007, 0x41),
                (0x4008, 0xFF), (0x400A, 0x80), (0x400B, 0x40), (0x400C, 0x3F), (0x400E, 0x05), (0x400F, 0x40),
                (0x4010, 0x4F), (0x4012, 0x00), (0x4013, 0x10), (0x4015, 0x1F), (0x4017, 0x00),
            })
                main.LdaImm(value).Sta(address);
            main.Op(0x58);
            ushort loop = main.Here;
            main.Inc(0x00).Lda(0x0000).Sta(0x6000).Lda(0x4016).Sta(0x0003).Lda(0x4017).Sta(0x0004).Jmp(loop);

            var nmi = new Asm(Nmi);
            nmi.Op(0x48).LdaImm(0x01).Sta(0x4016).LdaImm(0x00).Sta(0x4016).LdaImm(0x02).Sta(0x4014).Inc(0x01).Lda(0x0001).Sta(0x2005).Sta(0x2005);
            BoardNmi(nmi, mapper);
            nmi.Op(0x68).Op(0x40);

            var irq = new Asm(Irq);
            irq.Op(0x48).Lda(0x4015);
            BoardAck(irq, mapper, cycleIrq);
            irq.Inc(0x02).Lda(0x0002).Sta(0x2005).Op(0x68).Op(0x40);

            Assert.True(main.Bytes.Length <= 0x100 && nmi.Bytes.Length <= 0x80 && irq.Bytes.Length <= 0x80, "the program's pieces overlap");
            byte[] rom = SyntheticNesRom.Build(prgBanks: 8, chrBanks: chrBanks, mapper: mapper, battery: true);
            int prg = 16;
            foreach (int bank in new[] { 0x0000, 0x4000, 0x1C000 })
            {
                main.Bytes.CopyTo(rom, prg + bank);
                nmi.Bytes.CopyTo(rom, prg + bank + 0x100);
                irq.Bytes.CopyTo(rom, prg + bank + 0x180);
                for (int i = 0; i < 0x20; i++) rom[prg + bank + 0x200 + i] = (byte)((i * 7 + 0x0F) & 0x3F);
            }
            foreach (int vectors in new[] { 0x3FFA, 0x7FFA, 0x1FFFA })
            {
                rom[prg + vectors] = (byte)Nmi; rom[prg + vectors + 1] = Nmi >> 8;
                rom[prg + vectors + 2] = 0x00; rom[prg + vectors + 3] = 0xC0;
                rom[prg + vectors + 4] = (byte)Irq; rom[prg + vectors + 5] = Irq >> 8;
            }
            return rom;
        }

        // The IRQ each board can raise, armed; FME-7's $6000 window opened as RAM.
        private static void BoardInit(Asm a, int mapper, bool cycleIrq)
        {
            switch (mapper)
            {
                case 4:
                    a.LdaImm(0x10).Sta(0xC000).Sta(0xC001).Sta(0xE001);
                    break;
                case 64:
                    a.LdaImm(0x10).Sta(0xC000).LdaImm(cycleIrq ? (byte)0x01 : (byte)0x00).Sta(0xC001).Sta(0xE001);
                    break;
                case 65:
                    a.LdaImm(0x10).Sta(0x9005).LdaImm(0x00).Sta(0x9006).Sta(0x9004).LdaImm(0x80).Sta(0x9003);
                    break;
                case 67:
                    a.LdaImm(0x10).Sta(0xC800).LdaImm(0x00).Sta(0xC800).LdaImm(0x10).Sta(0xD800);
                    break;
                case 69:
                    a.LdaImm(0x08).Sta(0x8000).LdaImm(0xC0).Sta(0xA000);
                    a.LdaImm(0x0E).Sta(0x8000).LdaImm(0x00).Sta(0xA000).LdaImm(0x0F).Sta(0x8000).LdaImm(0x20).Sta(0xA000);
                    a.LdaImm(0x0D).Sta(0x8000).LdaImm(0x81).Sta(0xA000);
                    break;
            }
        }

        // Each frame, the frame count written into the board's CHR and mirroring registers, never into the bank the code runs from.
        private static void BoardNmi(Asm a, int mapper)
        {
            a.Lda(0x0001);
            switch (mapper)
            {
                case 1:
                    for (int i = 0; i < 5; i++) { a.Sta(0xA000); if (i < 4) a.Op(0x4A); }
                    break;
                case 2: a.Sta(0x8000); break;
                case 3: a.Sta(0x8000); break;
                case 4: a.Sta(0xA000).LdaImm(0x02).Sta(0x8000).Lda(0x0001).Sta(0x8001).LdaImm(0x82).Sta(0x8000).Lda(0x0001).Sta(0x8001); break;
                case 7: a.AndImm(0x10).Sta(0x8000); break;
                case 9: a.Sta(0xB000).Sta(0xD000).Sta(0xF000); break;
                case 11: a.AndImm(0xF0).Sta(0x8000); break;
                case 64: a.Sta(0xA000).LdaImm(0x22).Sta(0x8000).Lda(0x0001).Sta(0x8001).LdaImm(0x08).Sta(0x8000).Lda(0x0001).Sta(0x8001); break;
                case 65: a.Sta(0xB000).Sta(0xB003).Sta(0x9001); break;
                case 66: a.AndImm(0x03).Sta(0x8000); break;
                case 67: a.Sta(0x8800).Sta(0xB800).Sta(0xE800); break;
                case 68: a.Sta(0x8000).Sta(0xC000).Sta(0xD000).Sta(0xE000); break;
                case 69: a.Sta(0x0005).LdaImm(0x00).Sta(0x8000).Lda(0x0005).Sta(0xA000).LdaImm(0x0C).Sta(0x8000).Lda(0x0005).Sta(0xA000); break;
                case 71: a.AndImm(0x10).Sta(0x9000); break;
                case 79: a.AndImm(0x07).Sta(0x4100); break;
            }
        }

        private static void BoardAck(Asm a, int mapper, bool cycleIrq)
        {
            switch (mapper)
            {
                case 4: a.Sta(0xE000).Sta(0xE001); break;
                case 64:
                    a.Sta(0xE000);
                    if (cycleIrq) a.LdaImm(0x01).Sta(0xC001);
                    a.Sta(0xE001);
                    break;
                case 65: a.Sta(0x9004).LdaImm(0x80).Sta(0x9003); break;
                case 67: a.LdaImm(0x10).Sta(0xC800).LdaImm(0x00).Sta(0xC800).LdaImm(0x10).Sta(0xD800); break;
                case 69: a.LdaImm(0x0D).Sta(0x8000).LdaImm(0x81).Sta(0xA000); break;
            }
        }
    }

    // Just enough of a 6502 assembler for BoardPrograms; branch offsets are given relative to the branch's own first byte.
    public sealed class Asm
    {
        private readonly List<byte> _bytes = new();
        private readonly ushort _origin;

        public Asm(ushort origin) => _origin = origin;

        public ushort Here => (ushort)(_origin + _bytes.Count);
        public byte[] Bytes => _bytes.ToArray();

        public Asm Op(params byte[] bytes) { _bytes.AddRange(bytes); return this; }
        private Asm Abs(byte opcode, ushort address) => Op(opcode, (byte)address, (byte)(address >> 8));
        public Asm LdaImm(byte v) => Op(0xA9, v);
        public Asm AndImm(byte v) => Op(0x29, v);
        public Asm Ldx(byte v) => Op(0xA2, v);
        public Asm Cpx(byte v) => Op(0xE0, v);
        public Asm Lda(ushort a) => Abs(0xAD, a);
        public Asm LdaX(ushort a) => Abs(0xBD, a);
        public Asm Sta(ushort a) => Abs(0x8D, a);
        public Asm StaX(ushort a) => Abs(0x9D, a);
        public Asm Bit(ushort a) => Abs(0x2C, a);
        public Asm Inc(byte zp) => Op(0xE6, zp);
        public Asm Jmp(ushort a) => Abs(0x4C, a);
        public Asm Bpl(int fromStart) => Op(0x10, (byte)(fromStart - 2));
        public Asm Bne(int fromStart) => Op(0xD0, (byte)(fromStart - 2));
    }

    internal sealed class MoonRtPair
    {
        public sealed class BothRefused : Exception
        {
            public BothRefused(string message) : base(message) { }
        }

        public readonly MoonCore Csharp;
        public readonly MoonMachine Rust;
        private readonly bool _skip;
        private readonly int _stateEvery;
        private readonly bool _soundAndPicture;
        private readonly byte[] _frame = new byte[MoonMachine.FrameBytes];
        private long _samples;
        private int _frames;
        private string? _layout;

        public string Summary => $"{_frames} frames identical in state{(_soundAndPicture ? ", sound and picture" : "")}, {_samples} samples, {Csharp.Cpu!.Cycles} CPU cycles";

        public MoonRtPair(byte[] rom, bool skipRendering, MoonCore? transferAt = null, int stateEvery = 1, bool soundAndPicture = true)
        {
            Assert.True(MoonMachine.Available, MoonNative.Report);
            _skip = skipRendering;
            _stateEvery = stateEvery;
            _soundAndPicture = soundAndPicture;
            Exception? csharp = null, rust = null;
            MoonCore? core = null;
            MoonMachine? machine = null;
            try { core = MoonRtStateTests.Load(rom); } catch (Exception e) { csharp = e; }
            try { machine = new MoonMachine(rom); } catch (Exception e) { rust = e; }
            if (csharp is not null || rust is not null)
            {
                machine?.Dispose();
                Assert.True(csharp?.GetType() == rust?.GetType(), $"C# {csharp?.GetType().Name ?? "loaded"}, Rust {rust?.GetType().Name ?? "loaded"}");
                throw new BothRefused(csharp!.Message);
            }
            Csharp = core!;
            Rust = machine!;
            Csharp.SkipRendering = skipRendering;
            Rust.SetOptions(skipRendering);
            if (transferAt is not null)
            {
                // Both fresh, so the fields no state carries start equal (Moon_Native.md §3.1).
                byte[] state = MoonRtStateTests.Save(transferAt);
                Csharp.LoadState(new MemoryStream(state));
                Rust.Load(state);
            }
            CompareState("load");
        }

        public void Run(int frames, Func<int, (uint, uint)?>? script)
        {
            for (int f = 0; f < frames; f++) Assert.True(Frame(script, f), $"frame {f}: both threw, which a frame run here does not expect");
        }

        public void Reset()
        {
            Csharp.Reset();
            Rust.Reset();
            CompareState("reset");
        }

        public string RustText()
        {
            var text = new System.Text.StringBuilder();
            Span<byte> one = stackalloc byte[1];
            for (int i = 0; i < 1024; i++)
            {
                Rust.ReadSpace(2, 0x6004 + i, one);
                if (one[0] == 0) break;
                text.Append(one[0] switch { >= 0x20 and < 0x7F => (char)one[0], 0x0A => '\n', _ => ' ' });
            }
            return text.ToString().Trim();
        }

        // False when both engines threw the same exception.
        public bool Frame(Func<int, (uint, uint)?>? script, int f)
        {
            if (script?.Invoke(f) is { } pads)
            {
                for (int b = 0; b < 8; b++)
                {
                    Csharp.SetButton(0, (NesButton)b, (pads.Item1 & (1u << b)) != 0);
                    Csharp.SetButton(1, (NesButton)b, (pads.Item2 & (1u << b)) != 0);
                }
                Rust.SetButtons(0, pads.Item1);
                Rust.SetButtons(1, pads.Item2);
            }

            Exception? csharp = Record(() => Csharp.RunFrame());
            Exception? rust = Record(() => Rust.RunFrame());
            Assert.True(csharp?.GetType() == rust?.GetType(), $"frame {_frames}: C# {csharp?.GetType().Name ?? "ran"}, Rust {rust?.GetType().Name ?? "ran"}");
            _frames++;
            if (csharp is not null) return false;

            if (_soundAndPicture)
            {
                short[] a = Csharp.DequeueAudioSamples(int.MaxValue), b = Rust.DrainAudio(int.MaxValue);
                int same = a.AsSpan().CommonPrefixLength(b);
                Assert.True(same == a.Length && a.Length == b.Length, $"frame {_frames}: {a.Length} C# samples, {b.Length} Rust, first difference at {same}");
                _samples += a.Length;
                if (!_skip)
                {
                    Rust.CopyFrame(_frame);
                    int pixel = Csharp.GetFrameBufferRgba().AsSpan().CommonPrefixLength(_frame);
                    Assert.True(pixel == _frame.Length, $"frame {_frames}: the pictures differ first at byte {pixel} (x {pixel / 4 % 256}, y {pixel / 1024})");
                }
            }
            if (_frames % _stateEvery == 0) CompareState($"frame {_frames}");
            return true;
        }

        // One instruction on each, as the frame loop runs one, compared; the cycles, or -1 when both threw the same exception.
        public int Step()
        {
            var bus = Csharp.Bus!;
            int cycles = 0;
            Exception? csharp = Record(() =>
            {
                cycles = bus.TakePendingDmaCycles();
                Csharp.Cpu!.SetNmiLine(Csharp.Ppu!.NmiOutput);
                cycles += Csharp.Cpu.Step();
                cycles += bus.TakeStolenCycles();
            });
            int rust = Rust.Step();
            Assert.True(csharp is null ? rust == cycles : rust < 0 && MoonMachine.Refusal(rust).GetType() == csharp.GetType(), $"C# {(csharp?.GetType().Name ?? $"{cycles} cycles")}, Rust {rust}");
            if (csharp is not null) return -1;
            CompareState($"pc ${Csharp.Cpu!.PC:X4}");
            return cycles;
        }

        private static Exception? Record(Action action)
        {
            try { action(); return null; }
            catch (Exception e) when (e is IndexOutOfRangeException or DivideByZeroException or ArgumentOutOfRangeException) { return e; }
        }

        public void CompareState(string when)
        {
            byte[] want = MoonRtStateTests.Save(Csharp), got = Rust.Save();
            int first = want.AsSpan().CommonPrefixLength(got);
            if (first == want.Length && want.Length == got.Length) return;
            _layout ??= Rust.Layout();
            string field = _layout.Split('\n').Where(l => l.Length > 0).Select(l => l.Split(' ')).LastOrDefault(p => int.Parse(p[0]) <= first) is { } line ? line[^1] : "?";
            Assert.Fail($"{when}: the states differ first at byte {first}, in {field}: C# {(first < want.Length ? want[first] : -1)}, Rust {(first < got.Length ? got[first] : -1)}");
        }
    }
}
