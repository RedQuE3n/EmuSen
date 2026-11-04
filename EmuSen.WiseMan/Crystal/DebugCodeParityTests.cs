using System;
using System.Collections.Generic;
using System.Linq;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.GameBoy;
using EmuSen.Crystal.Systems.N64;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.Crystal.Systems.Snes;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.WiseMan.Crystal
{
    // The packs' debugger code in emusen-systems held to the C#'s answers, recorded as goldens before it was deleted: every opcode of the three disassemblers and generated VR4300 words, the NES's board names, and every view on generated memory, every tile and tilemap entry and the bytes each view reads included - see EmuSen_Backend.md §13.5, §40.1.
    public class DebugCodeParityTests
    {
        private static string Shown(Func<object?> answer)
        {
            try
            {
                return answer() switch
                {
                    null => "null",
                    byte[] b => Convert.ToHexString(b),
                    (byte[] rgba, int w, int h) => $"{w}x{h} {Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(rgba))}",
                    IEnumerable<DebugPaletteInfo> p => string.Join(";", p.Select(x => $"{x.Index}:{string.Join(",", x.Colors)}")),
                    IEnumerable<DebugDmaChannel> d => string.Join(";", d.Select(c => $"{c.Index} {c.GeneralEnabled} {c.HdmaEnabled} {c.Control} {c.DestinationRegister} {c.SourceAddress} {c.TransferSize} {c.HdmaActive} {c.TableAddress} {c.LineCounter} {c.IndirectAddress}")),
                    IEnumerable<InterruptVector> v => string.Join(";", v.Select(x => $"{x.Name} {x.Address} {x.Width} {x.Mode}")),
                    IEnumerable<DebugSpriteInfo> sp => string.Join(";", sp.Select(x => $"{x.Index} {x.X} {x.Y} {x.Width} {x.Height} {x.TileIndex} {x.PaletteIndex} {x.Priority} {x.FlipX} {x.FlipY}")),
                    IEnumerable<DebugAudioChannelInfo> ch => string.Join(";", ch.Select(x => $"{x.Index} {x.Name} {x.Active} {x.Level} {x.Muted} {x.Info}")),
                    PhysicalAddress pa => $"{pa.Space} {pa.Offset} {pa.IsAddressable}",
                    IEnumerable<string> s => string.Join(",", s),
                    var other => other.ToString()!,
                };
            }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
        }

        [ThreadStatic] private static ParityGolden? _golden;

        // The group a case's answer is digested with, for a sweep; null to keep the answer as text.
        [ThreadStatic] private static Func<string, string?>? _group;

        private static void Same(string what, Func<object?> answer, string? reads = null)
        {
            string recorded = reads is null ? Shown(answer) : Shown(answer) + " reads " + reads;
            if (_group?.Invoke(what) is { } group) _golden!.AddChunked(group, recorded, 64);
            else _golden!.Add(what, recorded);
        }

        private static string Native(SystemCode.Instruction i, bool reference) => reference ? (i.Kind is { } k ? $"({k}, {i.Target})" : "null") : $"{i.Length} {i.Mnemonic} {i.Operands}";

        [Fact]
        public void Every_6502_and_sm83_opcode_disassembles_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            _golden = new ParityGolden("Disassemblers6502Sm83");
            _group = what => System.Text.RegularExpressions.Regex.Replace(what, @"\b[0-9A-F]{2}\b|-?\d+", "#");
            byte[] operands = { 0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF, 0x34 };
            int[] addresses = { 0, 0x8000, 0xFFFE, -1, 0x7FFF_FFFF, int.MinValue };
            int[] references = { 0, 0x12, 0x1234, 0xFFFF, 0x10000, -5, 0x80 };
            for (int op = 0; op < 256; op++)
            {
                byte o = (byte)op;
                Same($"6502 {o:X2} length", () => SystemCode.Disassemble("6502", new[] { o }, 0, 0).Length);
                Same($"6502 {o:X2} mnemonic", () => SystemCode.Disassemble("6502", new[] { o }, 0, 0).Mnemonic);
                Same($"sm83 {o:X2} length", () => SystemCode.Disassemble("sm83", new[] { o }, 0, 0).Length);
                foreach (int a in addresses)
                    foreach (byte lo in operands)
                        foreach (byte hi in new byte[] { 0x00, 0xC3, 0xFF })
                        {
                            Same($"6502 {o:X2} {lo:X2} {hi:X2} at {a}", () => SystemCode.Disassemble("6502", new[] { o, lo, hi }, a, 0).Operands);
                            Same($"sm83 {o:X2} {lo:X2} {hi:X2} at {a}", () => SystemCode.Disassemble("sm83", new[] { o, lo, hi }, a, 0).Operands);
                        }
                foreach (int r in references)
                {
                    Same($"6502 {o:X2} refers {r}", () => Native(SystemCode.Disassemble("6502", new[] { o }, 0, r), true));
                    foreach (int a in addresses)
                        Same($"sm83 {o:X2} refers {r} at {a}", () => Native(SystemCode.Disassemble("sm83", new[] { o }, a, r), true));
                }
                for (int suffix = 0; suffix < 256; suffix++)
                {
                    byte x = (byte)suffix;
                    Same($"sm83 {o:X2} {x:X2} mnemonic", () => SystemCode.Disassemble("sm83", new[] { o, x }, 0, 0).Mnemonic);
                }
            }
            _golden.Check(1000);
            (_golden, _group) = (null, null);
        }

        // Each primary opcode with every function and register field the decode reads, and random words besides, at addresses that wrap.
        [Fact]
        public void Every_vr4300_word_disassembles_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1996);
            var words = new List<uint> { 0, 0xFFFF_FFFF };
            for (uint op = 0; op < 64; op++)
                for (uint field = 0; field < 64; field++)
                    foreach (uint regs in new uint[] { 0, 0x001F_0000, 0x03E0_0000, 0x03FF_F800, 0x0001_0000, 0x0020_0000, 0x0008_0000, 0x0010_07C0 })
                    {
                        words.Add(op << 26 | regs | field);
                        words.Add(op << 26 | regs | field | (uint)r.Next(0x10000) & 0xFFFF);
                        words.Add(op << 26 | (field << 21) | regs >> 5 | (uint)r.Next(64));
                    }
            for (int i = 0; i < 200_000; i++) words.Add((uint)r.Next() ^ (uint)r.Next() << 1);
            uint[] addresses = { 0x8000_0400, 0xFFFF_FFFC, 0, 0xBFC0_07C0, 0x7FFF_FFF8 };
            var golden = new ParityGolden("Vr4300");
            int structured = words.Count - 200_000;
            int n = 0;
            foreach (uint w in words)
            {
                uint a = addresses[n++ % addresses.Length];
                var i = SystemCode.Disassemble("vr4300", new[] { (byte)(w >> 24), (byte)(w >> 16), (byte)(w >> 8), (byte)w }, unchecked((int)a), 0);
                string native = $"{i.Mnemonic} {i.Operands} {i.Kind} {(i.Kind is null ? 0 : unchecked((uint)i.Target))}";
                golden.AddChunked(n <= structured ? "words" : "random words", native, n <= structured ? 64 : 1000);
            }
            golden.Check(500);
            Assert.True(words.Count > 290_000, $"{words.Count} words");
        }

        // Each primary opcode with every function, element and register field the RSP's decode reads, and random words besides, at IMEM addresses that wrap - see EmuSen_Backend.md §18.7.
        [Fact]
        public void Every_rsp_word_disassembles_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1997);
            var words = new List<uint> { 0, 0xFFFF_FFFF };
            for (uint op = 0; op < 64; op++)
                for (uint field = 0; field < 64; field++)
                    foreach (uint regs in new uint[] { 0, 0x001F_0000, 0x03E0_0000, 0x03FF_F800, 0x0001_0000, 0x0020_0000, 0x0200_0000, 0x01E0_0000, 0x0000_F800, 0x0000_0780, 0x0000_07C0 })
                    {
                        words.Add(op << 26 | regs | field);
                        words.Add(op << 26 | regs | field | (uint)r.Next(0x10000) & 0xFFFF);
                        words.Add(op << 26 | (field << 21) | regs >> 5 | (uint)r.Next(64));
                        words.Add(op << 26 | (field << 11) | regs | (uint)r.Next(128));
                    }
            for (int i = 0; i < 200_000; i++) words.Add((uint)r.Next() ^ (uint)r.Next() << 1);
            uint[] addresses = { 0, 0xFFC, 0x800, 0x1004, 0xFFFF_FFFC };
            var golden = new ParityGolden("Rsp");
            int structured = words.Count - 200_000;
            int n = 0;
            foreach (uint w in words)
            {
                uint a = addresses[n++ % addresses.Length];
                var i = SystemCode.Disassemble("rsp", new[] { (byte)(w >> 24), (byte)(w >> 16), (byte)(w >> 8), (byte)w }, unchecked((int)a), 0);
                string native = $"{i.Mnemonic} {i.Operands} {i.Kind} {(i.Kind is null ? 0 : unchecked((uint)i.Target))}";
                golden.AddChunked(n <= structured ? "words" : "random words", native, n <= structured ? 64 : 1000);
            }
            golden.Check(500);
            Assert.True(words.Count > 340_000, $"{words.Count} words");
        }

        // The N64 pack's views against the C#'s answers: each space's decode, the physical map over its boundaries, and the summary on generated registers - see EmuSen_Backend.md §18.7.
        [Fact]
        public void The_n64_views_answer_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(64);
            var golden = new ParityGolden("N64Views");
            void Same(string what, object? native) => golden.Add(what, native?.ToString() ?? "null");
            string Text(MarsInstruction i) => $"{i.Mnemonic} {i.Operands} {i.Kind} {(i.Kind is null ? 0 : i.Target)}";
            string Place(PhysicalAddress p) => $"{p.Space} {p.Offset} {p.IsAddressable}";
            string[] spaces = { "RDRAM", "rdram", "DMEM", "IMEM", "PIFRAM", "ROM", "CPU", "VI", "nothing" };
            for (int i = 0; i < 4_000; i++)
            {
                uint word = (uint)r.Next() ^ (uint)r.Next() << 1, at = (uint)r.Next() ^ (uint)r.Next() << 1;
                if (i % 4 == 0) at &= 0xFFF;
                string space = spaces[i % spaces.Length];
                Same($"{space} {word:X8} at {at:X8}", Text(N64Views.DecodeIn(space, word, at)));
            }
            var addresses = new List<uint> { 0, 0x003F_FFFF, 0x0040_0000, 0x007F_FFFF, 0x0080_0000, 0x03EF_FFFF, 0x03F0_0000, 0x03FF_FFFF, 0x0400_0000, 0x0400_0FFF, 0x0400_1000, 0x0403_FFFF,
                0x0404_0000, 0x04FF_FFFF, 0x0500_0000, 0x07FF_FFFF, 0x0800_0000, 0x0FFF_FFFF, 0x1000_0000, 0x1FBF_FFFF, 0x1FC0_0000, 0x1FC0_07BF, 0x1FC0_07C0, 0x1FC0_07FF, 0x1FC0_0800, 0xFFFF_FFFF };
            for (int i = 0; i < 2_000; i++) addresses.Add((uint)r.Next() ^ (uint)r.Next() << 1);
            foreach (uint a in addresses)
                foreach (long rdram in new long[] { 0, 4 << 20, 8 << 20 })
                    Same($"physical {a:X8} with {rdram}", Place(N64Views.Physical(a, rdram)));
            for (int round = 0; round < 50; round++)
            {
                var vi = new[] { "CONTROL", "ORIGIN", "WIDTH", "V_INTR", "V_CURRENT", "BURST", "V_SYNC" }.Take(round % 10 == 0 ? 3 : 7)
                    .Select(n => new DebugRegisterValue(round % 7 == 0 ? n.ToLowerInvariant() : n, (ulong)r.NextInt64(), 32)).ToArray();
                var registers = new[] { new DebugRegisterValue("PC", (ulong)r.NextInt64(), 64), new DebugRegisterValue("sp", (ulong)r.NextInt64(), 64), new DebugRegisterValue("ra", (ulong)r.NextInt64(), 64) };
                Same($"summary {round}", N64Views.Summary(registers, vi));
                Same($"summary without a VI {round}", N64Views.Summary(registers, null));
                Same($"summary without ra {round}", N64Views.Summary(registers[..2], vi) ?? "null");
            }
            golden.Check(10000);
        }

        [Fact]
        public void Every_mapper_number_names_the_golden_board()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var golden = new ParityGolden("BoardNames");
            for (int mapper = -2; mapper < 4100; mapper++) golden.Add(mapper.ToString(), SystemCode.BoardName(mapper) ?? "null");
            golden.Check(4000);
        }

        // A space over bytes that records every read, in order, and wraps an address outside it.
        private sealed class Logged(string name, byte[] data, bool sideEffects, List<string> log) : IDebugMemorySpace
        {
            public string Name => name;
            public int Size => data.Length;
            public bool IsWritable => false;
            public bool HasSideEffects => sideEffects;
            public byte Read(int address)
            {
                log.Add($"{name}:{address}");
                return data.Length == 0 ? (byte)0 : data[((address % data.Length) + data.Length) % data.Length];
            }
            public void Write(int address, byte value) { }
        }

        private static byte[] Random(Random r, int length)
        {
            var b = new byte[length];
            r.NextBytes(b);
            return b;
        }

        // A call on the library's views over spaces whose every read is logged; the reads digested as a set where the library may read a space whole.
        private static void Both(string what, (string Name, byte[] Data, bool SideEffects)[] spaces, Func<ISystemDebugViews, IReadOnlyList<IDebugMemorySpace>, object?> call, string system, bool whole = false)
        {
            var log = new List<string>();
            var n = spaces.Select(s => (IDebugMemorySpace)new Logged(s.Name, s.Data, s.SideEffects, log)).ToArray();
            string answer = Shown(() => call(new NativeDebugViews(system), n));
            Same(what, () => answer, Reads(whole ? log.Order(StringComparer.Ordinal) : log));
        }

        [Fact]
        public void Every_nes_view_answers_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1983);
            _golden = new ParityGolden("NesViews");
            _group = what => what.StartsWith("tile", StringComparison.Ordinal) ? System.Text.RegularExpressions.Regex.Replace(what, @"-?\d+", "#") : null;
            foreach (int chrSize in new[] { 0, 16, 8190, 8192 })
                foreach (bool sideEffects in new[] { false, true })
                {
                    var spaces = new[] { ("PALETTE", Random(r, 32), false), ("CHR", Random(r, chrSize), sideEffects), ("CIRAM", Random(r, 2048), false) };
                    string label = $"chr {chrSize} {(sideEffects ? "with" : "without")} side effects";
                    Both($"palettes {label}", spaces, (v, s) => v.Palettes(s), "nes");
                    Both($"swatch {label}", spaces, (v, s) => v.PaletteSwatch(s), "nes");
                    Both($"tile sheet {label}", spaces, (v, s) => v.TileSheet(s), "nes", whole: !sideEffects);
                }
            var empty = new[] { ("palette", Array.Empty<byte>(), false), ("chr", Array.Empty<byte>(), false) };
            Both("palettes of empty spaces", empty, (v, s) => v.Palettes(s), "nes");
            Both("palettes named in lower case", new[] { ("palette", Random(r, 32), false) }, (v, s) => v.Palettes(s), "nes");
            var ciram = new[] { ("CIRAM", Random(r, 4096), false) };
            for (int address = -1; address < 0x1000; address++)
            {
                int a = address;
                Both($"tilemap entry {a}", ciram, (v, s) => v.DecodeTilemapEntry(s[0], a), "nes");
            }
            var chr = new[] { ("CHR", Random(r, 8192), false) };
            for (int address = 0; address < 8192; address += 16)
                foreach (int bpp in new[] { 2, 4 })
                {
                    int a = address, b = bpp;
                    Both($"tile {a} at {b}bpp", chr, (v, s) => v.DecodeTilePixels(s[0], a, b), "nes");
                }
            Both("stride", chr, (v, s) => v.TilemapEntryStride, "nes");
            Both("vectors", chr, (v, s) => v.InterruptVectors, "nes");
            Both("dma", chr, (v, s) => v.DmaChannels(s), "nes");
            Both("dma destination", chr, (v, s) => v.NameDmaDestination(0x18), "nes");
            Both("symbols", chr, (v, s) => v.SymbolNames, "nes");
            _golden.Check();
            (_golden, _group) = (null, null);
        }

        [Fact]
        public void Every_snes_view_answers_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1990);
            _golden = new ParityGolden("SnesViews");
            _group = what => what.StartsWith("tile", StringComparison.Ordinal) ? System.Text.RegularExpressions.Regex.Replace(what, @"-?\d+", "#") : null;
            for (int n = 0; n < 6; n++)
            {
                byte[] io = Random(r, n == 5 ? 0x3FF : 0x400);
                var spaces = new[] { ("VRAM", Random(r, n == 4 ? 0x8000 : 0x10000), n == 3), ("CGRAM", Random(r, 512), false), ("IO", io, false) };
                Both($"palettes {n}", spaces, (v, s) => v.Palettes(s), "snes");
                Both($"swatch {n}", spaces, (v, s) => v.PaletteSwatch(s), "snes");
                Both($"tile sheet {n}", spaces, (v, s) => v.TileSheet(s), "snes");
                Both($"dma {n}", spaces, (v, s) => v.DmaChannels(s), "snes");
            }
            Both("no spaces", Array.Empty<(string, byte[], bool)>(), (v, s) => v.Palettes(s), "snes");
            var vram = new[] { ("VRAM", Random(r, 0x10000), false) };
            for (int address = 0; address < 0x10000; address += 2)
            {
                int a = address;
                Both($"tilemap entry {a}", vram, (v, s) => v.DecodeTilemapEntry(s[0], a), "snes");
            }
            for (int address = 0; address < 0x10000; address += 32)
                foreach (int bpp in new[] { 2, 4, 8, 3 })
                {
                    int a = address, b = bpp;
                    Both($"tile {a} at {b}bpp", vram, (v, s) => v.DecodeTilePixels(s[0], a, b), "snes");
                }
            for (int register = 0; register < 256; register++)
            {
                byte g = (byte)register;
                Both($"dma destination {g:X2}", vram, (v, s) => v.NameDmaDestination(g), "snes");
            }
            Both("stride", vram, (v, s) => v.TilemapEntryStride, "snes");
            Both("vectors", vram, (v, s) => v.InterruptVectors, "snes");
            Both("symbols", vram, (v, s) => v.SymbolNames, "snes");
            string[] names = { "PB", "PC", "A", "X", "Y", "S", "D", "DB", "P", "E" };
            for (int n = 0; n < 400; n++)
            {
                var registers = names.Where(_ => n % 7 != 0 || r.Next(10) > 0).Select(x => new DebugRegisterValue(r.Next(4) == 0 ? x.ToLowerInvariant() : x, (ulong)r.NextInt64() * (ulong)r.Next(1, 3), 16)).ToArray();
                Both($"summary {n}", vram, (v, s) => v.Summary(registers), "snes");
                foreach (string symbol in new[] { "flag.c", "FLAG.N", "flag.m", "sp", "DBR", "pc", "" })
                    Both($"symbol {symbol} {n}", vram, (v, s) => v.Symbol(symbol, registers), "snes");
            }
            _golden.Check();
            (_golden, _group) = (null, null);
        }

        private static DebugUnit GbUnit(Random r, string name, string kind, params (string Name, int Bits)[] registers) =>
            new(0, name, kind, registers.Select(x => new DebugRegisterValue(x.Name, (ulong)r.NextInt64(0, 1L << Math.Min(x.Bits + 1, 40)), x.Bits)).ToArray());

        // Units as a Game Boy engine lists them, in colour mode or not, with HDMA, a channel or the APU left out at times, values past their widths at times.
        private static DebugUnit[] GbUnits(Random r, bool colour)
        {
            var ppu = new List<(string, int)> { ("LCDC", 8), ("STAT", 8), ("SCY", 8), ("SCX", 8), ("LY", 8), ("LYC", 8), ("BGP", 8), ("OBP0", 8), ("OBP1", 8), ("WY", 8), ("WX", 8), ("Mode", 2), ("Dot", 16), ("WindowLine", 8) };
            if (colour) ppu.AddRange(new[] { ("VBK", 1), ("SVBK", 3), ("BCPS", 8), ("OCPS", 8), ("KEY1", 8) });
            var units = new List<DebugUnit> { GbUnit(r, "PPU", "video", ppu.ToArray()) };
            if (r.Next(5) > 0) units.Add(GbUnit(r, "APU", "audio", Enumerable.Range(0xFF10, 0x17).Select(a => ($"${a:X4}", 8)).Concat(new[] { ("Power", 1), ("SeqStep", 3) }).ToArray()));
            units.Add(GbUnit(r, "Timer", "timer", ("DIV", 8), ("TIMA", 8), ("TMA", 8), ("TAC", 8)));
            if (r.Next(6) > 0) units.Add(GbUnit(r, "Pulse 1", "channel", ("Enabled", 1), ("Dac", 1), ("Duty", 2), ("Freq", 11), ("Vol", 4), ("Len", 8), ("SweepPeriod", 3), ("SweepShift", 3), ("SweepNegate", 1)));
            units.Add(GbUnit(r, "Pulse 2", "channel", ("Enabled", 1), ("Dac", 1), ("Duty", 2), ("Freq", 11), ("Vol", 4), ("Len", 8)));
            units.Add(GbUnit(r, r.Next(4) == 0 ? "wave" : "Wave", "channel", ("Enabled", 1), ("Dac", 1), ("Freq", 11), ("Vol", 2), ("Len", 9)));
            units.Add(GbUnit(r, "Noise", "channel", ("Enabled", 1), ("Dac", 1), ("Shift", 4), ("Div", 3), ("Short", 1), ("Vol", 4), ("Len", 8)));
            if (colour) units.Add(GbUnit(r, "HDMA", "dma", ("BlocksLeft", 8), ("HBlank", 1), ("Source", 16), ("Destination", 16)));
            return units.ToArray();
        }

        // A call on the library's views over spaces and units, the reads logged in order.
        private static void BothWithUnits(string what, (string Name, byte[] Data, bool SideEffects)[] spaces, DebugUnit[] units, string system,
            Func<ISystemDebugViews, IReadOnlyList<IDebugMemorySpace>, IReadOnlyList<DebugUnit>, object?> call)
        {
            var log = new List<string>();
            var n = spaces.Select(x => (IDebugMemorySpace)new Logged(x.Name, x.Data, x.SideEffects, log)).ToArray();
            string answer = Shown(() => call(new NativeDebugViews(system), n, units));
            Same(what, () => answer, Reads(log));
        }

        // Units as an engine of the two Sega consoles lists them, a unit left out at times and values past their widths at times - see Endou_Native.md §15.4.
        private static DebugUnit[] SmsUnits(Random r, bool sg1000)
        {
            var vdp = Enumerable.Range(0, 11).Select(i => ($"R{i}", 8)).Concat(new[] { ("Status", 8), ("LineFlag", 1), ("Address", 14), ("Code", 2), ("Second", 1), ("Buffer", 8),
                ("LineCounter", 8), ("VCounter", 8), ("HLatch", 8), ("Line", 9), ("Lines", 8), ("Mode4", 1), ("IRQ", 1), ("Variant", 3) }).ToArray();
            var units = new List<DebugUnit>();
            if (r.Next(8) > 0)
            {
                DebugUnit u = GbUnit(r, r.Next(5) == 0 ? "vdp" : "VDP", "video", vdp);
                var regs = u.Registers.ToArray();
                void Set(string n, ulong value) { int at = Array.FindIndex(regs, x => x.Name == n); regs[at] = new DebugRegisterValue(n, value, regs[at].BitWidth); }
                if (r.Next(4) > 0) Set("Lines", (ulong)new[] { 192, 224, 240 }[r.Next(3)]);
                if (r.Next(3) > 0) Set("Variant", sg1000 ? 4UL : (ulong)r.Next(1, 3));
                units.Add(u with { Registers = regs });
            }
            if (r.Next(6) > 0) units.Add(GbUnit(r, "PSG", "audio", ("Tone0", 10), ("Tone1", 10), ("Tone2", 10), ("Noise", 3), ("Volume0", 4), ("Volume1", 4), ("Volume2", 4), ("Volume3", 4), ("Latch", 3)));
            if (r.Next(6) > 0)
            {
                DebugUnit b = GbUnit(r, "Board", "cartridge", ("Mapper", 2), ("Control", 8), ("Bank0", 8), ("Bank1", 8), ("Bank2", 8), ("Ram", 16), ("Battery", 1));
                var regs = b.Registers.ToArray();
                if (r.Next(2) == 0) regs[0] = new DebugRegisterValue("Mapper", 1, 2);
                if (r.Next(2) == 0) regs[5] = new DebugRegisterValue("Ram", (ulong)new[] { 0, 0x2000, 0x8000 }[r.Next(3)], 16);
                units.Add(b with { Registers = regs });
            }
            if (!sg1000 && r.Next(4) > 0) units.Add(GbUnit(r, "IO", "io", ("MemoryControl", 8), ("IoControl", 8)));
            return units.ToArray();
        }

        // A call on the library's views of the Master System or the SG-1000 over spaces and units, the reads logged in order.
        private static void BothSms(string what, (string Name, byte[] Data, bool SideEffects)[] spaces, DebugUnit[] units, bool sg1000,
            Func<ISystemDebugViews, IReadOnlyList<IDebugMemorySpace>, IReadOnlyList<DebugUnit>, object?> call) => BothWithUnits(what, spaces, units, sg1000 ? "sg1000" : "sms", call);

        // Every view of the two Sega consoles on generated spaces and units, every tilemap entry and tile of a VRAM's first part, and the address map's edges - see Endou_Native.md §15.5.
        [Fact]
        public void Every_sms_view_answers_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1985);
            _golden = new ParityGolden("SmsViews");
            _group = what => what.StartsWith("tile", StringComparison.Ordinal) ? System.Text.RegularExpressions.Regex.Replace(what, @"-?\d+", "#") : null;
            for (int round = 0; round < 64; round++)
            {
                bool sg1000 = round % 2 == 1;
                byte[] vram = Random(r, round % 9 == 8 ? 0x2000 : 0x4000);
                for (int i = 0; i < vram.Length; i++) if (r.Next(40) == 0) vram[i] = 0xD0;
                var spaces = new List<(string, byte[], bool)> { ("RAM", Random(r, sg1000 ? 0x400 : 0x2000), false), ("VRAM", vram, round % 5 == 4), ("ROM", Random(r, 0x8000 << (round % 3)), false) };
                if (!sg1000 || round % 4 == 1) spaces.Add((round % 7 == 6 ? "cram" : "CRAM", Random(r, round % 11 == 10 ? 16 : 32), false));
                var all = spaces.ToArray();
                DebugUnit[] units = round % 13 == 12 ? Array.Empty<DebugUnit>() : SmsUnits(r, sg1000);
                string label = $"round {round} ({(sg1000 ? "sg1000" : "sms")})";
                BothSms($"palettes {label}", all, units, sg1000, (v, s, u) => v.Palettes(s, u));
                BothSms($"palettes without units {label}", all, units, sg1000, (v, s, u) => v.Palettes(s));
                BothSms($"swatch {label}", all, units, sg1000, (v, s, u) => v.PaletteSwatch(s, u));
                BothSms($"tile sheet {label}", all, units, sg1000, (v, s, u) => v.TileSheet(s));
                BothSms($"sprites {label}", all, units, sg1000, (v, s, u) => v.Sprites(s, u));
                uint mutes = (uint)r.Next(16);
                BothSms($"channels {label}", all, units, sg1000, (v, s, u) => v.AudioChannels(s, u, mutes));
                string[] names = { "AF", "BC", "DE", "HL", "AF'", "BC'", "DE'", "HL'", "IX", "IY", "SP", "PC", "I", "R", "WZ", "IM", "IFF1", "IFF2" };
                var registers = names.Where(x => x != "PC" || round % 9 != 7).Select(x => new DebugRegisterValue(x, (ulong)r.Next(0x20000), 16)).ToArray();
                BothSms($"summary {label}", all, units, sg1000, (v, s, u) => v.Summary(registers, s, u));
                BothSms($"summary without units {label}", all, units, sg1000, (v, s, u) => v.Summary(registers));
                foreach (DebugUnit unit in units) BothSms($"label {unit.Name} {label}", all, units, sg1000, (v, s, u) => v.UnitLabel(unit) ?? "null");
                foreach (int address in new[] { -1, 0, 0x7FFF, 0xBFFF, 0xC000, 0xC3FF, 0xC400, 0xDFFF, 0xE000, 0xFFFC, 0xFFFF, 0x10000 })
                    BothSms($"address {address:X} {label}", all, units, sg1000, (v, s, u) => v.ResolvePhysical(address, s, u));
            }
            var bank = new[] { ("VRAM", Random(r, 0x4000), false) };
            foreach (bool sg1000 in new[] { false, true })
            {
                for (int address = -1; address < 0x1000; address += 3)
                {
                    int a = address;
                    BothSms($"tilemap entry {a} {sg1000}", bank, Array.Empty<DebugUnit>(), sg1000, (v, s, u) => v.DecodeTilemapEntry(s[0], a));
                }
                for (int address = 0; address < 0x2000; address += 8)
                    foreach (int bpp in new[] { 1, 2, 4 })
                    {
                        int a = address, b = bpp;
                        BothSms($"tile {a} at {b}bpp {sg1000}", bank, Array.Empty<DebugUnit>(), sg1000, (v, s, u) => v.DecodeTilePixels(s[0], a, b));
                    }
                var none = Array.Empty<DebugUnit>();
                BothSms($"stride {sg1000}", bank, none, sg1000, (v, s, u) => v.TilemapEntryStride);
                BothSms($"vectors {sg1000}", bank, none, sg1000, (v, s, u) => v.InterruptVectors);
                BothSms($"max sprites {sg1000}", bank, none, sg1000, (v, s, u) => v.MaxSprites);
                BothSms($"dma {sg1000}", bank, none, sg1000, (v, s, u) => v.DmaChannels(s, u));
                BothSms($"symbols {sg1000}", bank, none, sg1000, (v, s, u) => v.SymbolNames);
            }
            _golden.Check();
            (_golden, _group) = (null, null);
        }

        // Every Game Boy view on generated spaces and units in each console mode, every tilemap entry and tile of a VRAM bank, and the address map from end to end - see EmuSen_Backend.md §17.10.
        [Fact]
        public void Every_game_boy_view_answers_as_the_golden_says()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1989);
            _golden = new ParityGolden("GameBoyViews");
            _group = what => what.StartsWith("tile", StringComparison.Ordinal) ? System.Text.RegularExpressions.Regex.Replace(what, @"-?\d+", "#") : null;
            for (int round = 0; round < 48; round++)
            {
                bool colour = round % 3 == 0, twoBanks = colour || round % 3 == 1;
                string system = round % 2 == 0 ? "gb" : "gbc";
                byte[] rom = Random(r, new[] { 0x14F, 0x150, 0x8000, 0x20000 }[round % 4]);
                if (rom.Length > 0x149) (rom[0x147], rom[0x149]) = ((byte)r.Next(256), (byte)r.Next(8));
                byte[] oam = Random(r, round % 11 == 5 ? 100 : 0xA0);
                for (int i = 0; i + 3 < oam.Length; i += 4) if (r.Next(3) == 0) oam[i] = (byte)(r.Next(2) == 0 ? 0 : 160);
                var spaces = new List<(string, byte[], bool)> { ("ROM", rom, false), ("VRAM", Random(r, twoBanks ? 0x4000 : 0x2000), false), ("OAM", oam, false), ("CPUBUS", Random(r, 16), true) };
                if (round % 7 != 6) spaces.AddRange(new[] { ("BGPAL", Random(r, 64), false), (round % 13 == 12 ? "objpal" : "OBJPAL", Random(r, 64), false) });
                var all = spaces.ToArray();
                DebugUnit[] units = round % 17 == 16 ? Array.Empty<DebugUnit>() : GbUnits(r, colour);
                string label = $"round {round} ({(colour ? "colour" : twoBanks ? "compatibility" : "dmg")})";
                BothWithUnits($"palettes {label}", all, units, system, (v, s, u) => v.Palettes(s, u));
                BothWithUnits($"palettes without units {label}", all, units, system, (v, s, u) => v.Palettes(s));
                BothWithUnits($"swatch {label}", all, units, system, (v, s, u) => v.PaletteSwatch(s, u));
                BothWithUnits($"tile sheet {label}", all, units, system, (v, s, u) => v.TileSheet(s));
                BothWithUnits($"sprites {label}", all, units, system, (v, s, u) => v.Sprites(s, u));
                uint mutes = (uint)r.Next(32);
                BothWithUnits($"channels {label}", all, units, system, (v, s, u) => v.AudioChannels(s, u, mutes));
                BothWithUnits($"dma {label}", all, units, system, (v, s, u) => v.DmaChannels(s, u));
                string[] names = { "A", "F", "BC", "DE", "HL", "SP", "PC", "Z", "N", "H", "C", "IME", "HALT", "IE", "IF", "Cycles" };
                var registers = names.Where(x => x != "PC" || round % 9 != 8).Select(x => new DebugRegisterValue(x, (ulong)r.Next(0x20000), 16)).ToArray();
                BothWithUnits($"summary {label}", all, units, system, (v, s, u) => v.Summary(registers, s, u));
                BothWithUnits($"summary without units {label}", all, units, system, (v, s, u) => v.Summary(registers));
                foreach (int address in new[] { -1, 0, 0x3FFF, 0x4000, 0x7FFF, 0x8000, 0x9FFF, 0xA000, 0xC000, 0xCFFF, 0xD000, 0xDFFF, 0xE000, 0xFE00, 0xFE9F, 0xFEA0, 0xFF00, 0xFF7F, 0xFF80, 0xFFFE, 0xFFFF, 0x10000 })
                    BothWithUnits($"address {address:X} {label}", all, units, system, (v, s, u) => v.ResolvePhysical(address, s, u));
            }
            var bank = new[] { ("VRAM", Random(r, 0x4000), false) };
            foreach (bool colour in new[] { false, true })
            {
                DebugUnit[] units = GbUnits(r, colour);
                for (int address = -1; address < 0x2000; address += 3)
                {
                    int a = address;
                    BothWithUnits($"tilemap entry {a} {(colour ? "colour" : "dmg")}", bank, units, "gb", (v, s, u) => v.DecodeTilemapEntry(s[0], a, s, u));
                    BothWithUnits($"tilemap entry without units {a}", bank, units, "gb", (v, s, u) => v.DecodeTilemapEntry(s[0], a));
                }
            }
            for (int address = 0; address < 0x4000; address += 16)
                foreach (int bpp in new[] { 2, 4 })
                {
                    int a = address, b = bpp;
                    BothWithUnits($"tile {a} at {b}bpp", bank, Array.Empty<DebugUnit>(), "gbc", (v, s, u) => v.DecodeTilePixels(s[0], a, b));
                }
            var none = Array.Empty<DebugUnit>();
            BothWithUnits("stride", bank, none, "gb", (v, s, u) => v.TilemapEntryStride);
            BothWithUnits("vectors", bank, none, "gb", (v, s, u) => v.InterruptVectors);
            BothWithUnits("max sprites", bank, none, "gb", (v, s, u) => v.MaxSprites);
            BothWithUnits("dma destination", bank, none, "gb", (v, s, u) => v.NameDmaDestination(0x51));
            BothWithUnits("symbols", bank, none, "gb", (v, s, u) => v.SymbolNames);
            _golden.Check();
            (_golden, _group) = (null, null);
        }

        // Units as an NES engine lists them, a unit left out at times and values past their widths at times - see EmuSen_Backend.md §23.2.
        private static DebugUnit[] NesUnits(Random r)
        {
            var units = new List<DebugUnit>();
            if (r.Next(8) > 0) units.Add(GbUnit(r, "PPU", "video", ("PPUCTRL", 8), ("PPUMASK", 8), ("PPUSTATUS", 8), ("v", 15), ("t", 15), ("x", 3), ("Scanline", 9)));
            if (r.Next(6) > 0) units.Add(GbUnit(r, "Board", "cartridge", ("Mapper", r.Next(3) == 0 ? 12 : 7)));
            if (r.Next(6) > 0) units.Add(GbUnit(r, "Cartridge", "cartridge", ("Mirroring", 2), ("Battery", 1), ("ChrRam", 1)));
            foreach (string pulse in new[] { "Pulse 1", "Pulse 2" })
                if (r.Next(7) > 0) units.Add(GbUnit(r, pulse, "channel", ("TimerPeriod", 11), ("SweepShift", 3), ("SweepNegate", 1), ("OnesComplement", 1), ("LengthCounter", 8), ("ConstantVolume", 1), ("Volume", 4), ("Decay", 4), ("Duty", 2), ("SweepEnabled", 1)));
            if (r.Next(6) > 0) units.Add(GbUnit(r, "Triangle", "channel", ("LengthCounter", 8), ("TimerPeriod", 11), ("ControlFlag", 1)));
            if (r.Next(6) > 0) units.Add(GbUnit(r, "Noise", "channel", ("LengthCounter", 8), ("ConstantVolume", 1), ("Volume", 4), ("Decay", 4), ("PeriodIndex", 4), ("ShortMode", 1)));
            if (r.Next(6) > 0) units.Add(GbUnit(r, "DMC", "channel", ("OutputLevel", 7), ("BytesRemaining", 12), ("RateIndex", 4), ("SampleAddress", 16), ("SampleLength", 12), ("Loop", 1)));
            return units.ToArray();
        }

        // The NES views that read the units, which the C# answered alone until 2026-11-04: the sprites, the channels, the address map, the boards' labels and the summary - see EmuSen_Backend.md §40.1.
        [Fact]
        public void Every_nes_unit_view_matches_the_golden()
        {
            var r = new Random(1985_10_18);
            var golden = new ParityGolden("NesUnitViews");
            ISystemDebugViews views = NesSystem.Pack.DebugViews!;
            for (int round = 0; round < 64; round++)
            {
                var log = new List<string>();
                byte[] oam = Random(r, round % 9 == 8 ? 200 : 256);
                for (int i = 0; i + 3 < oam.Length; i += 4) if (r.Next(3) == 0) oam[i] = (byte)(r.Next(2) == 0 ? 0 : 240 + r.Next(16));
                var spaces = new List<IDebugMemorySpace> { new Logged(round % 7 == 6 ? "oam" : "OAM", oam, false, log), new Logged("PRGROM", Random(r, 0x4000 << (round % 4)), false, log), new Logged("CHR", Random(r, round % 5 == 4 ? 0 : 0x2000 << (round % 3)), false, log) };
                DebugUnit[] units = round % 13 == 12 ? Array.Empty<DebugUnit>() : NesUnits(r);
                string[] names = { "A", "X", "Y", "S", "PC", "P" };
                var registers = names.Where(x => x != "PC" || round % 9 != 7).Select(x => new DebugRegisterValue(x, (ulong)r.Next(0x20000), 16)).ToArray();
                uint mutes = (uint)r.Next(32);
                string label = $"round {round}";
                golden.Add($"sprites {label}", () => views.Sprites(spaces, units), Show);
                golden.Add($"max sprites {label}", () => views.MaxSprites, Show);
                golden.Add($"channels {label}", () => views.AudioChannels(spaces, units, mutes), Show);
                golden.Add($"summary {label}", () => views.Summary(registers, spaces, units), Show);
                foreach (DebugUnit unit in units) golden.Add($"label {unit.Name} {label}", () => views.UnitLabel(unit), Show);
                foreach (int address in new[] { -1, 0, 0x7FF, 0x800, 0x1FFF, 0x2000, 0x2007, 0x2008, 0x3FFF, 0x4000, 0x401F, 0x4020, 0x5FFF, 0x6000, 0x7FFF, 0x8000, 0xFFFF, 0x10000 })
                    golden.Add($"address {address:X} {label}", () => views.ResolvePhysical(address, spaces, units), Show);
            }
            foreach (long mapper in new long[] { -1, 0, 1, 4, 5, 79, 80, 4095, 1L << 40 })
                golden.Add($"label of mapper {mapper}", () => views.UnitLabel(new DebugUnit(0, "Board", "cartridge", new[] { new DebugRegisterValue("Mapper", unchecked((ulong)mapper), 64) })), Show);
            golden.Add("label of a unit that is no cartridge", () => views.UnitLabel(new DebugUnit(0, "Board", "video", new[] { new DebugRegisterValue("Mapper", 1, 8) })), Show);
            golden.Check(1000);
        }

        private static string Show(object? answer) => Shown(() => answer);

        // A view's reads, in order, as a short digest.
        private static string Reads(IEnumerable<string> log) => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(System.Text.Encoding.UTF8.GetBytes(string.Join("\n", log))))[..16];
    }
}
