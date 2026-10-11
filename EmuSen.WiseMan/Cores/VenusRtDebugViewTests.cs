using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Cauldron;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // VenusRT's views through the SNES system pack against C# Venus's on one synthetic image at one frame - see EmuSen_CoreAPI.md §31.
    [Collection(TestCollections.ProcessGlobals)]
    public class VenusRtDebugViewTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenVenusRtView_" + Guid.NewGuid().ToString("N"));
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public VenusRtDebugViewTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreOptions.BatteryRamDisabled = true;
            CoreDiscovery.UseDirectories(null);
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
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
            ICore core = CoreFactory.Create(path, headless: true, engine);
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

        private Pair Both(byte[] image, int frames = 3, Action<IDebugTarget>? before = null)
        {
            string path = Write("game.sfc", image);
            var (venus, venusView) = Run(path, CoreCatalog.VenusEngine, frames, before);
            var (rust, rustView) = Run(path, CoreCatalog.VenusRtEngine, frames, before);
            Assert.IsType<CoreEngine>(rust);
            return new Pair(venus, venusView, rust, rustView);
        }

        // C# Venus and VenusRT on one image, each with its target.
        private sealed record Pair(ICore VenusCore, IDebugTarget Venus, ICore RustCore, IDebugTarget Rust) : IDisposable
        {
            public void Deconstruct(out IDebugTarget venus, out IDebugTarget rust, out IDisposable owner) => (venus, rust, owner) = (Venus, Rust, this);

            public void RunFrames(int frames)
            {
                for (int i = 0; i < frames; i++)
                {
                    VenusCore.RunFrame();
                    RustCore.RunFrame();
                }
            }

            public void Dispose()
            {
                (VenusCore as IDisposable)?.Dispose();
                (RustCore as IDisposable)?.Dispose();
            }
        }

        private static string Shell(IDebugTarget target, string line) => DianaShInterpreter.CreateDefault(target).Submit(line).Output;

        [Fact]
        public void Palettes_match_Venus()
        {
            var (venus, rust, owner) = Both(Image(Program));
            using (owner)
            {
                string all = Shell(venus, "pal");
                Assert.Equal(16, all.Split('\n').Length);
                Assert.NotEqual(Shell(venus, "pal 1"), Shell(venus, "pal 3"));
                Assert.Equal(all, Shell(rust, "pal"));
                Assert.Equal(venus.RenderPaletteSwatch().Rgba, rust.RenderPaletteSwatch().Rgba);
            }
        }

        [Fact]
        public void Tiles_tilemaps_and_the_tile_sheet_match_Venus()
        {
            var (venus, rust, owner) = Both(Image(Program));
            using (owner)
            {
                foreach (string line in new[] { "tile VRAM 0 2", "tile VRAM 0 4", "tile VRAM 0 8", "tile VRAM 10 2", "tilemap VRAM 0 8 4" })
                    Assert.Equal(Shell(venus, line), Shell(rust, line));
                Assert.Matches("[1-9A-F]", Shell(rust, "tile VRAM 0 4").Split('\n')[1]);
                Assert.Equal(venus.TilemapEntryStride, rust.TilemapEntryStride);
                var (sheet, w, h) = venus.RenderTileSheet();
                var rustSheet = rust.RenderTileSheet();
                Assert.Equal((w, h), (rustSheet.Width, rustSheet.Height));
                Assert.Equal(sheet, rustSheet.Rgba);
            }
        }

        [Fact]
        public void Dma_channels_match_Venus()
        {
            var (venus, rust, owner) = Both(Image(Program));
            using (owner)
            {
                string table = Shell(venus, "dma");
                Assert.Contains("  2   on ", table);
                Assert.Contains("  3   arm ", table);
                Assert.Contains("$008140", table);
                Assert.Equal(table, Shell(rust, "dma"));
                Assert.Equal(venus.DmaChannels.Select(Fields), rust.DmaChannels.Select(Fields));
            }
        }

        // Every field but the indirect address, which C# Venus keeps apart from the byte count the two share on the hardware - see EmuSen_CoreAPI.md §31.2.
        private static string Fields(DebugDmaChannel c) =>
            $"{c.Index} {c.GeneralEnabled} {c.HdmaEnabled} {c.Control:X2} {c.DestinationRegister:X2} {c.SourceAddress:X6} {c.TransferSize} {c.HdmaActive} {c.TableAddress:X6} {c.LineCounter:X2}";

        [Fact]
        public void Vectors_match_Venus()
        {
            var (venus, rust, owner) = Both(Image(Program));
            using (owner)
            {
                string vectors = Shell(venus, "vectors");
                Assert.Contains("RESET        $00FFFC -> $008000", vectors);
                Assert.Equal(vectors, Shell(rust, "vectors"));
            }
        }

        [Fact]
        public void Flag_and_alias_symbols_match_Venus()
        {
            var (venus, rust, owner) = Both(Image(Program));
            using (owner)
            {
                Assert.Equal("[STATE] CPU", Shell(venus, "summary")[..11]);
                foreach (string symbol in new[] { "flag.c", "flag.z", "flag.i", "flag.d", "flag.x", "flag.m", "flag.v", "flag.n", "sp", "dp", "pbr", "dbr", "p" })
                    Assert.Equal(Shell(venus, $"eval {symbol}"), Shell(rust, $"eval {symbol}"));
                Assert.StartsWith("1 ", Shell(rust, "eval flag.n"));
                Assert.StartsWith("0 ", Shell(rust, "eval flag.x"));
            }
        }

        // The C# summary's CPU line is the pack's line under VenusRT's heading.
        [Fact]
        public void The_summary_carries_Venus_CPU_line()
        {
            var (venus, rust, owner) = Both(Image(Program));
            using (owner)
            {
                string[] rustLines = rust.GetSummaryText().Split('\n');
                Assert.StartsWith("VenusRT (Rust)", rustLines[0]);
                Assert.Equal(venus.GetSummaryText().Split('\n')[0], rustLines[1]);
                Assert.Contains("A=1234 X=5678 Y=9ABC", rustLines[1]);
                Assert.Contains("P=A5 [NvMxdIzC]  E=0", rustLines[1]);
            }
        }

        // A call stack VenusRT keeps on observed frames, so coverage is armed before they run.
        [Fact]
        public void Stack_depth_matches_Venus()
        {
            var (venus, rust, owner) = Both(Image(Program), before: t => t.Coverage!.Arm());
            using (owner)
            {
                Assert.StartsWith("1 ", Shell(venus, "eval stackdepth"));
                Assert.Equal(Shell(venus, "eval stackdepth"), Shell(rust, "eval stackdepth"));
            }
        }

        // The S-CPU starts the SA-1 at $8400 (CRV, then CCNT cleared), and the SA-1 sets its registers and loops.
        private static byte[] Sa1Image() => SyntheticRom.BuildSa1((0x7FD6, new byte[] { 0x35 }),
            (5, new byte[] { 0xA9, 0x00, 0x8D, 0x03, 0x22, 0xA9, 0x84, 0x8D, 0x04, 0x22, 0x9C, 0x00, 0x22, 0x80, 0xFE }),
            (0x400, new byte[] { 0x18, 0xFB, 0xC2, 0x30, 0xA9, 0x34, 0x12, 0xA2, 0x78, 0x56, 0xA0, 0xBC, 0x9A, 0xE2, 0xA5, 0x80, 0xFE }));

        // The combined view shows what `cop` names, as `regs sa1` shows it, and the 65816 registers C# Venus's SA-1 view shares hold the same values.
        [Fact]
        public void Regs_shows_the_SA1_as_the_coprocessor()
        {
            using var pair = Both(Sa1Image(), frames: 2);
            var (venus, rust, _) = pair;
            string regs = Shell(rust, "regs");
            Assert.Contains("Coprocessor registers:", regs);
            string combined = regs[(regs.IndexOf("Coprocessor registers:", StringComparison.Ordinal) + "Coprocessor registers:".Length)..].Trim();
            string alone = Shell(rust, "regs sa1");
            Assert.Equal(alone[(alone.IndexOf('\n') + 1)..].Trim(), combined);

            // S is the reset's: $01FF held, less the release's three stack cycles - see VenusRT_Disputes.md D-49.
            string[] shared = { "PB", "PC", "A", "X", "Y", "S", "P" };
            var csharp = venus.CoprocessorRegisters.Current.Where(r => shared.Contains(r.Name)).ToDictionary(r => r.Name, r => r.Value);
            var native = rust.CoprocessorRegisters.Current.Where(r => shared.Contains(r.Name)).ToDictionary(r => r.Name, r => r.Value);
            Assert.Equal(shared.Length, csharp.Count);
            Assert.Equal(shared.Select(n => (n, csharp[n])), shared.Select(n => (n, native[n])));
            Assert.Equal(0x1234ul, native["A"]);
            Assert.Equal(0x01FCul, native["S"]);
        }

        // A write the game makes each frame is undone by the frame's end on both, though VenusRT undoes it there and Venus at once.
        [Fact]
        public void A_freeze_holds_as_Venus_holds_it()
        {
            byte[] image = Image(c => Op(c, 0xE6, 0x10, 0x80, 0xFC));
            using var pair = Both(image, frames: 0);
            var (venus, rust, _) = pair;
            {
                foreach (var t in new[] { venus, rust }) Assert.StartsWith("Frozen #1", Shell(t, "freeze add WRAM 10 AB"));
                pair.RunFrames(3);
                Assert.Equal(Shell(venus, "mem WRAM 10 1"), Shell(rust, "mem WRAM 10 1"));
                Assert.Contains("AB", Shell(rust, "mem WRAM 10 1"));
                Assert.Contains("(3 write(s) undone)", Shell(rust, "freeze list"));
            }
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
