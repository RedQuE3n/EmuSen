using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Telemetry;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // VenusRT's views through the SNES system pack on one synthetic image at one frame - see EmuSen_CoreAPI.md §31.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusRtDebugViewTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenVenusRtView_" + Guid.NewGuid().ToString("N"));

        public VenusRtDebugViewTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(null);
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        // A program from $8005, after SyntheticRom's boot stub, with VRAM tiles at $8100, colours at $8200 and HDMA tables at $8300.
        private static byte[] Image(Action<List<byte>> program, int frames = 0)
        {
            var code = new List<byte>();
            program(code);
            byte[] tiles = Enumerable.Range(0, 64).Select(i => (byte)(i * 37 + 11)).ToArray();
            byte[] colours = Enumerable.Range(0, 64).Select(i => (byte)(i * 53 + 7)).ToArray();
            return SyntheticRom.Build((5, code.ToArray()), (0x100, tiles), (0x200, colours),
                (0x300, new byte[] { 0x7F, 0x11, 0x7F, 0x22, 0x7F, 0x33, 0x00 }), (0x310, new byte[] { 0x01, 0x44, 0x00 }));
        }

        private static void Op(List<byte> code, params int[] bytes) => code.AddRange(bytes.Select(b => (byte)b));

        private static void Sta8(List<byte> code, int value, int address)
        {
            Op(code, 0xA9, value);
            Op(code, 0x8D, address & 0xFF, address >> 8);
        }

        private static void Stx16(List<byte> code, int value, int address)
        {
            Op(code, 0xA2, value & 0xFF, value >> 8);
            Op(code, 0x8E, address & 0xFF, address >> 8);
        }

        // Two DMAs that fill VRAM and CGRAM, two HDMA channels to WH0 (one still running at the frame's end, one ended), a channel set but idle, then the registers to known values and a JSR into a one-instruction loop.
        private static void Program(List<byte> c)
        {
            Op(c, 0x18, 0xFB, 0xC2, 0x30, 0xE2, 0x20);
            Sta8(c, 0x80, 0x2100);
            Sta8(c, 0x80, 0x2115);
            Op(c, 0x9C, 0x16, 0x21, 0x9C, 0x17, 0x21);
            Sta8(c, 0x01, 0x4300);
            Sta8(c, 0x18, 0x4301);
            Stx16(c, 0x8100, 0x4302);
            Op(c, 0x9C, 0x04, 0x43);
            Stx16(c, 0x0040, 0x4305);
            Op(c, 0x9C, 0x21, 0x21, 0x9C, 0x10, 0x43);
            Sta8(c, 0x22, 0x4311);
            Stx16(c, 0x8200, 0x4312);
            Op(c, 0x9C, 0x14, 0x43);
            Stx16(c, 0x0040, 0x4315);
            Sta8(c, 0x03, 0x420B);
            foreach (var (channel, table) in new[] { (2, 0x8300), (3, 0x8310) })
            {
                Op(c, 0x9C, channel << 4, 0x43);
                Sta8(c, 0x26, 0x4301 + (channel << 4));
                Stx16(c, table, 0x4302 + (channel << 4));
                Op(c, 0x9C, 0x04 + (channel << 4), 0x43);
            }
            Sta8(c, 0xC3, 0x4350);
            Sta8(c, 0x18, 0x4351);
            Stx16(c, 0x1234, 0x4352);
            Sta8(c, 0x7E, 0x4354);
            Stx16(c, 0x5678, 0x4355);
            Sta8(c, 0x7F, 0x4357);
            Stx16(c, 0x9ABC, 0x4358);
            Sta8(c, 0x85, 0x435A);
            Sta8(c, 0x0C, 0x420C);
            Op(c, 0xC2, 0x30, 0xA9, 0x34, 0x12, 0xA2, 0x78, 0x56, 0xA0, 0xBC, 0x9A, 0xC2, 0xFF, 0xE2, 0xA5);
            int loop = 0x8005 + c.Count + 3;
            Op(c, 0x20, loop & 0xFF, loop >> 8, 0x80, 0xFE);
        }

        private string Write(string name, byte[] bytes)
        {
            string path = Path.Combine(_root, name);
            File.WriteAllBytes(path, bytes);
            return path;
        }

        // Built and run as the corpus and a frontend do, the target made after the frames; <before> runs on the target first, for what must be armed.
        private static (ICore Core, IDebugTarget Target) Run(string path, string? engine, int frames, Action<IDebugTarget>? before = null)
        {
            ICore core = CoreFactory.Create(path, headless: true, engine, options: TestCores.NoBattery);
            core.LoadRom(path);
            if (before is not null)
            {
                IDebugTarget early = CoreFactory.Bundle(core).DebugTarget;
                before(early);
                for (int i = 0; i < frames; i++) core.RunFrame();
                early.RefreshProviders();
                return (core, early);
            }
            for (int i = 0; i < frames; i++) core.RunFrame();
            return (core, CoreFactory.Bundle(core).DebugTarget);
        }

        private Game One(byte[] image, int frames = 3, Action<IDebugTarget>? before = null)
        {
            var (core, target) = Run(Write("game.sfc", image), null, frames, before);
            Assert.IsType<CoreEngine>(core);
            return new Game(core, target);
        }

        // VenusRT on one image, with its target.
        private sealed record Game(ICore Core, IDebugTarget Target) : IDisposable
        {
            public void Dispose() => (Core as IDisposable)?.Dispose();
        }

        private static string Shell(IDebugTarget target, string line) => DianaShInterpreter.CreateDefault(target).Submit(line).Output;

        [Fact]
        public void Palettes_are_the_packs_sixteen()
        {
            using var game = One(Image(Program));
            string all = Shell(game.Target, "pal");
            Assert.Equal(16, all.Split('\n').Length);
            Assert.NotEqual(Shell(game.Target, "pal 1"), Shell(game.Target, "pal 3"));
            Assert.Equal((192, 192), (game.Target.RenderPaletteSwatch().Width, game.Target.RenderPaletteSwatch().Height));
        }

        // The palettes come from each frame's copy in the snapshot ring and match the live decode, and a refresh allocates nothing - see EmuSen_Backend.md §14.5.
        [Fact]
        public void Palette_snapshots_match_the_live_decode_and_allocate_nothing()
        {
            using var game = One(Image(Program));
            for (int i = 0; i < 200; i++)
            {
                game.Core.RunFrame();
                game.Target.RefreshProviders();
            }
            long before = GC.GetAllocatedBytesForCurrentThread();
            for (int i = 0; i < 1000; i++) game.Target.RefreshProviders();
            Assert.Equal(0L, GC.GetAllocatedBytesForCurrentThread() - before);

            static string Text(IEnumerable<DebugPaletteInfo> palettes) => string.Join(";", palettes.Select(p => p.Index + ":" + string.Join(",", p.Colors)));
            var views = global::EmuSen.Crystal.Systems.SystemPacks.For(((CoreEngine)game.Core).Machine.Info.System)!.DebugViews!;
            string live = Text(views.Palettes(game.Target.GetMemorySpaces()));
            Assert.Equal(16, game.Target.Palettes.Current.Count);
            Assert.Equal(live, Text(game.Target.Palettes.Current));
        }

        [Fact]
        public void Tiles_tilemaps_and_the_tile_sheet_decode_vram()
        {
            using var game = One(Image(Program));
            Assert.Matches("[1-9A-F]", Shell(game.Target, "tile VRAM 0 4").Split('\n')[1]);
            Assert.Equal(2, game.Target.TilemapEntryStride);
            var (sheet, w, h) = game.Target.RenderTileSheet();
            Assert.Equal((256, 512), (w, h));
            Assert.Contains(sheet, b => b is not (0 or 255));
        }

        [Fact]
        public void Dma_channels_show_the_programs_transfers()
        {
            using var game = One(Image(Program));
            string table = Shell(game.Target, "dma");
            Assert.Contains("  2   on ", table);
            Assert.Contains("  3   arm ", table);
            Assert.Contains("$008140", table);
        }

        [Fact]
        public void Vectors_name_the_reset()
        {
            using var game = One(Image(Program));
            Assert.Contains("RESET        $00FFFC -> $008000", Shell(game.Target, "vectors"));
        }

        // P is A5h, N.MxdI.C: each flag symbol is its bit.
        [Fact]
        public void Flag_symbols_are_the_bits_of_P()
        {
            using var game = One(Image(Program));
            foreach (var (symbol, bit) in new[] { ("flag.c", 1), ("flag.z", 0), ("flag.i", 1), ("flag.d", 0), ("flag.x", 0), ("flag.m", 1), ("flag.v", 0), ("flag.n", 1) })
                Assert.StartsWith($"{bit} ", Shell(game.Target, $"eval {symbol}"));
        }

        // A condition naming a pack's symbol reads it from the registers at the hit, though a hit refreshes no view; P is A5h at the loop - see `man bp`.
        [Fact]
        public void A_condition_on_a_flag_symbol_reads_P_at_the_hit()
        {
            var code = new List<byte>();
            Program(code);
            int loop = 0x8005 + code.Count - 2;
            foreach (var (condition, halts) in new[] { ("flag.n==1", true), ("flag.n==0", false) })
            {
                using var game = One(Image(Program), before: t => Assert.Contains("added at", Shell(t, $"bp add {loop:X6} if {condition}")));
                Assert.Equal(halts, game.Core.IsHaltedAtBreakpoint);
                if (halts) Assert.Equal(loop, game.Core.HaltedAddress);
            }
        }

        // The pack's CPU line under VenusRT's heading.
        [Fact]
        public void The_summary_carries_the_CPU_line()
        {
            using var game = One(Image(Program));
            string[] lines = game.Target.GetSummaryText().Split('\n');
            Assert.StartsWith("VenusRT (Rust)", lines[0]);
            Assert.StartsWith("[STATE] CPU", lines[1]);
            Assert.Contains("A=1234 X=5678 Y=9ABC", lines[1]);
            Assert.Contains("P=A5 [NvMxdIzC]  E=0", lines[1]);
        }

        // A call stack VenusRT keeps on observed frames, so coverage is armed before they run.
        [Fact]
        public void Stack_depth_counts_the_JSR()
        {
            using var game = One(Image(Program), before: t => t.Coverage!.Arm());
            Assert.StartsWith("1 ", Shell(game.Target, "eval stackdepth"));
        }

        // The S-CPU starts the SA-1 at $8400 (CRV, then CCNT cleared), and the SA-1 sets its registers and loops.
        private static byte[] Sa1Image() => SyntheticRom.BuildSa1((0x7FD6, new byte[] { 0x35 }),
            (5, new byte[] { 0xA9, 0x00, 0x8D, 0x03, 0x22, 0xA9, 0x84, 0x8D, 0x04, 0x22, 0x9C, 0x00, 0x22, 0x80, 0xFE }),
            (0x400, new byte[] { 0x18, 0xFB, 0xC2, 0x30, 0xA9, 0x34, 0x12, 0xA2, 0x78, 0x56, 0xA0, 0xBC, 0x9A, 0xE2, 0xA5, 0x80, 0xFE }));

        // The combined view shows what `cop` names, as `regs sa1` shows it, with the values the SA-1's program set.
        [Fact]
        public void Regs_shows_the_SA1_as_the_coprocessor()
        {
            using var game = One(Sa1Image(), frames: 2);
            string regs = Shell(game.Target, "regs");
            Assert.Contains("Coprocessor registers:", regs);
            string combined = regs[(regs.IndexOf("Coprocessor registers:", StringComparison.Ordinal) + "Coprocessor registers:".Length)..].Trim();
            string alone = Shell(game.Target, "regs sa1");
            Assert.Equal(alone[(alone.IndexOf('\n') + 1)..].Trim(), combined);

            // S is the reset's: $01FF held, less the release's three stack cycles - see VenusRT_Disputes.md D-49.
            var native = game.Target.CoprocessorRegisters.Current.ToDictionary(r => r.Name, r => r.Value);
            Assert.Equal((0x1234ul, 0x5678ul, 0x9ABCul, 0x01FCul), (native["A"], native["X"], native["Y"], native["S"]));
        }

        // A write the game makes each frame is undone by the frame's end.
        [Fact]
        public void A_freeze_holds()
        {
            using var game = One(Image(c => Op(c, 0xE6, 0x10, 0x80, 0xFC)), frames: 0);
            Assert.StartsWith("Frozen #1", Shell(game.Target, "freeze add WRAM 10 AB"));
            for (int i = 0; i < 3; i++) game.Core.RunFrame();
            Assert.Contains("AB", Shell(game.Target, "mem WRAM 10 1"));
            Assert.Contains("(3 write(s) undone)", Shell(game.Target, "freeze list"));
        }

        // The engine's timing as the one phase a core without PHASES has, against the machine's own frame.
        [Fact]
        public void Hardware_load_shows_the_frame()
        {
            string path = Write("game.sfc", Image(Program));
            var (core, target) = Run(path, CoreCatalog.VenusRtEngine, 2);
            using (core as IDisposable)
            {
                core.RunFrame();
                target.RefreshProviders();
                var load = Assert.Single(target.HardwareLoad.Current);
                Assert.Equal(("frame", DebugLoadKind.EmulatorCost), (load.Name, load.Kind));
                Assert.InRange(load.Percent, 0.0001, 100.0);
            }
        }
    }
}
