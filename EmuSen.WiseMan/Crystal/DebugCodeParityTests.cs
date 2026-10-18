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
    // The packs' debugger code in emusen-systems held to the C#: every opcode of the three disassemblers and generated VR4300 words, the NES's board names, and every view on generated memory, every tile and tilemap entry and the bytes each view reads included - see EmuSen_Backend.md §13.5.
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
                    IEnumerable<string> s => string.Join(",", s),
                    var other => other.ToString()!,
                };
            }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
        }

        private static void Same(List<string> differences, string what, Func<object?> native, Func<object?> managed)
        {
            string n = Shown(native), m = Shown(managed);
            if (n != m) differences.Add($"{what}:\n  library: {n}\n  C#:      {m}");
        }

        private static string Native(SystemCode.Instruction i, bool reference) => reference ? (i.Kind is { } k ? $"({k}, {i.Target})" : "null") : $"{i.Length} {i.Mnemonic} {i.Operands}";

        [Fact]
        public void Every_6502_and_sm83_opcode_disassembles_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var differences = new List<string>();
            byte[] operands = { 0x00, 0x01, 0x7F, 0x80, 0xFE, 0xFF, 0x34 };
            int[] addresses = { 0, 0x8000, 0xFFFE, -1, 0x7FFF_FFFF, int.MinValue };
            int[] references = { 0, 0x12, 0x1234, 0xFFFF, 0x10000, -5, 0x80 };
            for (int op = 0; op < 256; op++)
            {
                byte o = (byte)op;
                Same(differences, $"6502 {o:X2} length", () => SystemCode.Disassemble("6502", new[] { o }, 0, 0).Length, () => Nes6502Disassembler.LengthOfManaged(o));
                Same(differences, $"6502 {o:X2} mnemonic", () => SystemCode.Disassemble("6502", new[] { o }, 0, 0).Mnemonic, () => Nes6502Disassembler.MnemonicOfManaged(o));
                Same(differences, $"sm83 {o:X2} length", () => SystemCode.Disassemble("sm83", new[] { o }, 0, 0).Length, () => Sm83Disassembler.LengthOfManaged(o));
                foreach (int a in addresses)
                    foreach (byte lo in operands)
                        foreach (byte hi in new byte[] { 0x00, 0xC3, 0xFF })
                        {
                            Same(differences, $"6502 {o:X2} {lo:X2} {hi:X2} at {a}", () => SystemCode.Disassemble("6502", new[] { o, lo, hi }, a, 0).Operands, () => Nes6502Disassembler.FormatOperandManaged(o, a, lo, hi));
                            Same(differences, $"sm83 {o:X2} {lo:X2} {hi:X2} at {a}", () => SystemCode.Disassemble("sm83", new[] { o, lo, hi }, a, 0).Operands, () => Sm83Disassembler.FormatOperandManaged(o, a, lo, hi));
                        }
                foreach (int r in references)
                {
                    Same(differences, $"6502 {o:X2} refers {r}", () => Native(SystemCode.Disassemble("6502", new[] { o }, 0, r), true), () => Nes6502Disassembler.ClassifyStaticReferenceManaged(o, r)?.ToString() ?? "null");
                    foreach (int a in addresses)
                        Same(differences, $"sm83 {o:X2} refers {r} at {a}", () => Native(SystemCode.Disassemble("sm83", new[] { o }, a, r), true), () => Sm83Disassembler.ClassifyStaticReferenceManaged(o, r, a)?.ToString() ?? "null");
                }
                for (int suffix = 0; suffix < 256; suffix++)
                {
                    byte x = (byte)suffix;
                    Same(differences, $"sm83 {o:X2} {x:X2} mnemonic", () => SystemCode.Disassemble("sm83", new[] { o, x }, 0, 0).Mnemonic, () => Sm83Disassembler.MnemonicOfManaged(o, x));
                }
            }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }

        // Each primary opcode with every function and register field the decode reads, and random words besides, at addresses that wrap.
        [Fact]
        public void Every_vr4300_word_disassembles_alike()
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
            var differences = new List<string>();
            int n = 0;
            foreach (uint w in words)
            {
                uint a = addresses[n++ % addresses.Length];
                var m = Vr4300Disassembler.DecodeManaged(w, a);
                var i = SystemCode.Disassemble("vr4300", new[] { (byte)(w >> 24), (byte)(w >> 16), (byte)(w >> 8), (byte)w }, unchecked((int)a), 0);
                string managed = $"{m.Mnemonic} {m.Operands} {m.Kind} {(m.Kind is null ? 0 : m.Target)}", native = $"{i.Mnemonic} {i.Operands} {i.Kind} {(i.Kind is null ? 0 : unchecked((uint)i.Target))}";
                if (managed != native && differences.Count < 10) differences.Add($"{w:X8} at {a:X8}:\n  library: {native}\n  C#:      {managed}");
            }
            Assert.True(words.Count > 290_000, $"{words.Count} words");
            Assert.True(differences.Count == 0, string.Join("\n", differences));
        }

        // Each primary opcode with every function, element and register field the RSP's decode reads, and random words besides, at IMEM addresses that wrap - see EmuSen_Backend.md §18.7.
        [Fact]
        public void Every_rsp_word_disassembles_alike()
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
            var differences = new List<string>();
            int n = 0;
            foreach (uint w in words)
            {
                uint a = addresses[n++ % addresses.Length];
                var m = RspDisassembler.DecodeManaged(w, a);
                var i = SystemCode.Disassemble("rsp", new[] { (byte)(w >> 24), (byte)(w >> 16), (byte)(w >> 8), (byte)w }, unchecked((int)a), 0);
                string managed = $"{m.Mnemonic} {m.Operands} {m.Kind} {(m.Kind is null ? 0 : m.Target)}", native = $"{i.Mnemonic} {i.Operands} {i.Kind} {(i.Kind is null ? 0 : unchecked((uint)i.Target))}";
                if (managed != native && differences.Count < 10) differences.Add($"{w:X8} at {a:X8}:\n  library: {native}\n  C#:      {managed}");
            }
            Assert.True(words.Count > 340_000, $"{words.Count} words");
            Assert.True(differences.Count == 0, string.Join("\n", differences));
        }

        // The N64 pack's views against the C# moved from the MarsRT shim: each space's decode, the physical map over its boundaries, and the summary on generated registers - see EmuSen_Backend.md §18.7.
        [Fact]
        public void The_n64_views_answer_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(64);
            var differences = new List<string>();
            void Same(string what, object? native, object? managed)
            {
                if (!Equals(native?.ToString(), managed?.ToString()) && differences.Count < 10) differences.Add($"{what}:\n  library: {native}\n  C#:      {managed}");
            }
            string Text(MarsInstruction i) => $"{i.Mnemonic} {i.Operands} {i.Kind} {(i.Kind is null ? 0 : i.Target)}";
            string Place(PhysicalAddress p) => $"{p.Space} {p.Offset} {p.IsAddressable}";
            string[] spaces = { "RDRAM", "rdram", "DMEM", "IMEM", "PIFRAM", "ROM", "CPU", "VI", "nothing" };
            for (int i = 0; i < 4_000; i++)
            {
                uint word = (uint)r.Next() ^ (uint)r.Next() << 1, at = (uint)r.Next() ^ (uint)r.Next() << 1;
                if (i % 4 == 0) at &= 0xFFF;
                string space = spaces[i % spaces.Length];
                Same($"{space} {word:X8} at {at:X8}", Text(N64Views.DecodeIn(space, word, at)), Text(N64Views.DecodeInManaged(space, word, at)));
            }
            var addresses = new List<uint> { 0, 0x003F_FFFF, 0x0040_0000, 0x007F_FFFF, 0x0080_0000, 0x03EF_FFFF, 0x03F0_0000, 0x03FF_FFFF, 0x0400_0000, 0x0400_0FFF, 0x0400_1000, 0x0403_FFFF,
                0x0404_0000, 0x04FF_FFFF, 0x0500_0000, 0x07FF_FFFF, 0x0800_0000, 0x0FFF_FFFF, 0x1000_0000, 0x1FBF_FFFF, 0x1FC0_0000, 0x1FC0_07BF, 0x1FC0_07C0, 0x1FC0_07FF, 0x1FC0_0800, 0xFFFF_FFFF };
            for (int i = 0; i < 2_000; i++) addresses.Add((uint)r.Next() ^ (uint)r.Next() << 1);
            foreach (uint a in addresses)
                foreach (long rdram in new long[] { 0, 4 << 20, 8 << 20 })
                    Same($"physical {a:X8} with {rdram}", Place(N64Views.Physical(a, rdram)), Place(N64Views.PhysicalManaged(a, rdram)));
            for (int round = 0; round < 50; round++)
            {
                var vi = new[] { "CONTROL", "ORIGIN", "WIDTH", "V_INTR", "V_CURRENT", "BURST", "V_SYNC" }.Take(round % 10 == 0 ? 3 : 7)
                    .Select(n => new DebugRegisterValue(round % 7 == 0 ? n.ToLowerInvariant() : n, (ulong)r.NextInt64(), 32)).ToArray();
                var registers = new[] { new DebugRegisterValue("PC", (ulong)r.NextInt64(), 64), new DebugRegisterValue("sp", (ulong)r.NextInt64(), 64), new DebugRegisterValue("ra", (ulong)r.NextInt64(), 64) };
                Same($"summary {round}", N64Views.Summary(registers, vi), N64Views.SummaryManaged(registers, vi));
                Same($"summary without a VI {round}", N64Views.Summary(registers, null), N64Views.SummaryManaged(registers, null));
                Same($"summary without ra {round}", N64Views.Summary(registers[..2], vi) ?? "null", N64Views.SummaryManaged(registers[..2], vi) ?? "null");
            }
            Assert.True(differences.Count == 0, string.Join("\n", differences));
        }

        [Fact]
        public void Every_mapper_number_names_the_same_board()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            for (int mapper = -2; mapper < 4100; mapper++) Assert.Equal(NesSystem.BoardNamesManaged.GetValueOrDefault(mapper), SystemCode.BoardName(mapper));
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

        // A call on both halves over spaces of the same bytes, each half's reads logged; the answers and the reads compared, the reads as a set where the library may read a space whole.
        private static void Both(List<string> differences, string what, (string Name, byte[] Data, bool SideEffects)[] spaces, Func<ISystemDebugViews, IReadOnlyList<IDebugMemorySpace>, object?> call, ISystemDebugViews managed, string system, bool whole = false)
        {
            var nativeLog = new List<string>();
            var managedLog = new List<string>();
            var n = spaces.Select(s => (IDebugMemorySpace)new Logged(s.Name, s.Data, s.SideEffects, nativeLog)).ToArray();
            var m = spaces.Select(s => (IDebugMemorySpace)new Logged(s.Name, s.Data, s.SideEffects, managedLog)).ToArray();
            Same(differences, what, () => call(new NativeDebugViews(system, managed), n), () => call(managed, m));
            bool sameReads = whole ? nativeLog.Order().SequenceEqual(managedLog.Order()) : nativeLog.SequenceEqual(managedLog);
            if (!sameReads) differences.Add($"{what}: the reads differ ({nativeLog.Count} by the library, {managedLog.Count} by the C#)");
        }

        [Fact]
        public void Every_nes_view_answers_alike_and_reads_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1983);
            var differences = new List<string>();
            ISystemDebugViews views = NesDebugViews.Instance;
            foreach (int chrSize in new[] { 0, 16, 8190, 8192 })
                foreach (bool sideEffects in new[] { false, true })
                {
                    var spaces = new[] { ("PALETTE", Random(r, 32), false), ("CHR", Random(r, chrSize), sideEffects), ("CIRAM", Random(r, 2048), false) };
                    string label = $"chr {chrSize} {(sideEffects ? "with" : "without")} side effects";
                    Both(differences, $"palettes {label}", spaces, (v, s) => v.Palettes(s), views, "nes");
                    Both(differences, $"swatch {label}", spaces, (v, s) => v.PaletteSwatch(s), views, "nes");
                    Both(differences, $"tile sheet {label}", spaces, (v, s) => v.TileSheet(s), views, "nes", whole: !sideEffects);
                }
            var empty = new[] { ("palette", Array.Empty<byte>(), false), ("chr", Array.Empty<byte>(), false) };
            Both(differences, "palettes of empty spaces", empty, (v, s) => v.Palettes(s), views, "nes");
            Both(differences, "palettes named in lower case", new[] { ("palette", Random(r, 32), false) }, (v, s) => v.Palettes(s), views, "nes");
            var ciram = new[] { ("CIRAM", Random(r, 4096), false) };
            for (int address = -1; address < 0x1000; address++)
            {
                int a = address;
                Both(differences, $"tilemap entry {a}", ciram, (v, s) => v.DecodeTilemapEntry(s[0], a), views, "nes");
            }
            var chr = new[] { ("CHR", Random(r, 8192), false) };
            for (int address = 0; address < 8192; address += 16)
                foreach (int bpp in new[] { 2, 4 })
                {
                    int a = address, b = bpp;
                    Both(differences, $"tile {a} at {b}bpp", chr, (v, s) => v.DecodeTilePixels(s[0], a, b), views, "nes");
                }
            Both(differences, "stride", chr, (v, s) => v.TilemapEntryStride, views, "nes");
            Both(differences, "vectors", chr, (v, s) => v.InterruptVectors, views, "nes");
            Both(differences, "dma", chr, (v, s) => v.DmaChannels(s), views, "nes");
            Both(differences, "dma destination", chr, (v, s) => v.NameDmaDestination(0x18), views, "nes");
            Both(differences, "symbols", chr, (v, s) => v.SymbolNames, views, "nes");
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }

        [Fact]
        public void Every_snes_view_answers_alike_and_reads_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1990);
            var differences = new List<string>();
            ISystemDebugViews views = SnesDebugViews.Instance;
            for (int n = 0; n < 6; n++)
            {
                byte[] io = Random(r, n == 5 ? 0x3FF : 0x400);
                var spaces = new[] { ("VRAM", Random(r, n == 4 ? 0x8000 : 0x10000), n == 3), ("CGRAM", Random(r, 512), false), ("IO", io, false) };
                Both(differences, $"palettes {n}", spaces, (v, s) => v.Palettes(s), views, "snes");
                Both(differences, $"swatch {n}", spaces, (v, s) => v.PaletteSwatch(s), views, "snes");
                Both(differences, $"tile sheet {n}", spaces, (v, s) => v.TileSheet(s), views, "snes");
                Both(differences, $"dma {n}", spaces, (v, s) => v.DmaChannels(s), views, "snes");
            }
            Both(differences, "no spaces", Array.Empty<(string, byte[], bool)>(), (v, s) => v.Palettes(s), views, "snes");
            var vram = new[] { ("VRAM", Random(r, 0x10000), false) };
            for (int address = 0; address < 0x10000; address += 2)
            {
                int a = address;
                Both(differences, $"tilemap entry {a}", vram, (v, s) => v.DecodeTilemapEntry(s[0], a), views, "snes");
            }
            for (int address = 0; address < 0x10000; address += 32)
                foreach (int bpp in new[] { 2, 4, 8, 3 })
                {
                    int a = address, b = bpp;
                    Both(differences, $"tile {a} at {b}bpp", vram, (v, s) => v.DecodeTilePixels(s[0], a, b), views, "snes");
                }
            for (int register = 0; register < 256; register++)
            {
                byte g = (byte)register;
                Both(differences, $"dma destination {g:X2}", vram, (v, s) => v.NameDmaDestination(g), views, "snes");
            }
            Both(differences, "stride", vram, (v, s) => v.TilemapEntryStride, views, "snes");
            Both(differences, "vectors", vram, (v, s) => v.InterruptVectors, views, "snes");
            Both(differences, "symbols", vram, (v, s) => v.SymbolNames, views, "snes");
            string[] names = { "PB", "PC", "A", "X", "Y", "S", "D", "DB", "P", "E" };
            for (int n = 0; n < 400; n++)
            {
                var registers = names.Where(_ => n % 7 != 0 || r.Next(10) > 0).Select(x => new DebugRegisterValue(r.Next(4) == 0 ? x.ToLowerInvariant() : x, (ulong)r.NextInt64() * (ulong)r.Next(1, 3), 16)).ToArray();
                Both(differences, $"summary {n}", vram, (v, s) => v.Summary(registers), views, "snes");
                foreach (string symbol in new[] { "flag.c", "FLAG.N", "flag.m", "sp", "DBR", "pc", "" })
                    Both(differences, $"symbol {symbol} {n}", vram, (v, s) => v.Symbol(symbol, registers), views, "snes");
            }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }
    }
}
