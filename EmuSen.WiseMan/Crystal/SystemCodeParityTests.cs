using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.GameBoy;
using EmuSen.Crystal.Systems.Nes;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The recognisers, routing, header readers and OpenVGDB rules in emusen-systems held to the C# on generated discs, cartridges and files, the bytes each recogniser reads included - see EmuSen_Backend.md §13.3.
    public class SystemCodeParityTests
    {
        // A probe over an image that records every length a recogniser asks for.
        private sealed class Probe(byte[] bytes) : IMediaProbe
        {
            public readonly List<int> Asked = new();
            public string Path => "image.bin";
            public string ImageName => "image.bin";
            public byte[] Read(int length)
            {
                Asked.Add(length);
                return bytes.Length <= length ? bytes : bytes[..length];
            }
        }

        private static string Shown(Func<object?> answer)
        {
            try
            {
                return answer() switch
                {
                    null => "null",
                    Recognition r => $"{r.SystemId} {r.Serial ?? "-"} {r.Region ?? "-"}",
                    byte[] b => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(b)) + $" {b.Length}",
                    var other => other.ToString()!,
                };
            }
            catch (Exception e) { return $"threw {e.GetType().Name}: {e.Message}"; }
        }

        [Fact]
        public void Every_system_has_code_where_its_literal_has()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            foreach (SystemEntry literal in SystemPacks.Literals)
            {
                SystemCode.SystemCodeFacts facts = SystemCode.Code.Single(c => c.Id == literal.Id);
                Assert.Equal(literal.Recognise is not null, facts.Recognises);
                Assert.Equal(literal.Id is "nes" or "gb" or "gbc", facts.Header);
            }
            Assert.Equal(SystemPacks.Literals.Select(l => l.Id), SystemCode.Code.Select(c => c.Id));
        }

        private static byte[] Cnf(byte[] iso, string config)
        {
            byte[] image = (byte[])iso.Clone();
            byte[] text = Encoding.ASCII.GetBytes(config);
            image.AsSpan(SyntheticPsxDisc.FirstFile * 2048, 2048).Clear();
            text.AsSpan(0, Math.Min(text.Length, 2048)).CopyTo(image.AsSpan(SyntheticPsxDisc.FirstFile * 2048));
            return image;
        }

        // Discs and cartridges of every kind the recognisers tell apart, cut at every length a recogniser's reads straddle, with noise written over parts.
        private static IEnumerable<(string Label, byte[] Image)> Images()
        {
            var r = new Random(1994);
            var discs = new List<(string, byte[])>();
            foreach (string market in new[] { "Americas", "Europe", "Japan" })
                foreach (int mode in new[] { 1, 2 })
                {
                    var disc = SyntheticPsxDisc.Build(market, mode);
                    discs.Add(($"psx iso {market} {mode}", disc.Iso));
                    discs.Add(($"psx raw {market} {mode}", disc.DataTrack));
                }
            byte[] iso = SyntheticPsxDisc.Build().Iso;
            foreach (string config in new[] { "BOOT=cdrom:\\SLUS_007.70;1", "  boot = cdrom:SCES_123.45;1\n", "BOOT2 = cdrom0:\\SLPS_000.01;1", "TCB = 4\nBOOT cdrom:\\X\nBOOT = cdrom:\\AB_CD.EF;1",
                "BOOT = cdrom:\\TOOLONGNAME_123.45;1", "BOOT = cdrom:\\NODASH.EXE;1", "BOOT = éééé-éé", "", "BOOT=" })
                discs.Add(($"psx cnf '{config}'", Cnf(iso, config)));
            discs.Add(("sega cd", SyntheticMdRom.Disc()));
            discs.Add(("sega cd europe", SyntheticMdRom.Disc("E", 0x20000)));
            discs.Add(("genesis", SyntheticMdRom.Cartridge()));
            byte[] raw = new byte[0x9000];
            new byte[] { 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x00 }.CopyTo(raw, 0);
            raw[15] = 1;
            Encoding.ASCII.GetBytes("SEGADISCSYSTEM").CopyTo(raw, 16);
            discs.Add(("raw sega cd mode 1", raw));
            byte[] raw2 = (byte[])raw.Clone();
            raw2[15] = 2;
            Encoding.ASCII.GetBytes("SEGADISCSYSTEM").CopyTo(raw2, 24);
            discs.Add(("raw sega cd mode 2", raw2));
            foreach (var (label, image) in discs)
            {
                yield return (label, image);
                foreach (int cut in new[] { 0, 12, 15, 16, 30, 0x40, 0x8005, 0x8006, 0x8010, 8223, 8224, 9463, 9464, 12 * 1024, 20 * 2048, 21 * 2048 + 10 })
                    if (cut < image.Length) yield return ($"{label} cut {cut}", image[..cut]);
                for (int n = 0; n < 6; n++)
                {
                    byte[] noisy = (byte[])image.Clone();
                    int at = r.Next(Math.Min(noisy.Length, 0xC000)), length = r.Next(1, 600);
                    r.NextBytes(noisy.AsSpan(at, Math.Min(length, noisy.Length - at)));
                    yield return ($"{label} noise {n}", noisy);
                }
            }
            for (int n = 0; n < 20; n++)
            {
                byte[] noise = new byte[r.Next(0, 0x30000)];
                r.NextBytes(noise);
                yield return ($"noise {n}", noise);
            }
        }

        [Fact]
        public void Every_recogniser_answers_alike_and_reads_the_same_bytes()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var differences = new List<string>();
            int count = 0;
            foreach (var (label, image) in Images())
                foreach (SystemEntry literal in SystemPacks.Literals.Where(l => l.Recognise is not null))
                {
                    count++;
                    Probe managed = new(image), native = new(image);
                    string m = Shown(() => literal.Recognise!(managed)), n = Shown(() => SystemCode.Recognise(literal.Id, native));
                    m += " reads " + string.Join(",", managed.Asked);
                    n += " reads " + string.Join(",", native.Asked);
                    if (m != n) differences.Add($"{literal.Id} on {label}:\n  library: {n}\n  C#:      {m}");
                }
            Assert.True(count > 1000, $"{count} cases");
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }

        [Fact]
        public void Routing_decides_alike_among_any_candidates_claims_and_folders()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(84);
            var entries = SystemPacks.All.Select(p => p.Entry).ToArray();
            string?[] folders = { null, "", "roms", "psx", "PSX", "segacd", "SegaCD", "genesis", "nes", "n64", "gbc", "Ps×" };
            var differences = new List<string>();
            for (int i = 0; i < 4000; i++)
            {
                var candidates = entries.OrderBy(_ => r.Next()).Take(r.Next(0, 5)).ToArray();
                var claims = Enumerable.Range(0, candidates.Length).Where(_ => r.Next(3) == 0)
                    .Select(at => (at, new Recognition(candidates[at].Id, r.Next(2) == 0 ? null : $"SLUS-{r.Next(100000):D5}", r.Next(3) switch { 0 => null, 1 => "Japan", _ => "Europe" }))).ToArray();
                string? folder = folders[r.Next(folders.Length)];
                string file = r.Next(4) == 0 ? "Café Game (Disc 1).cue" : $"game{i}.iso";
                string m = Show(SystemRouting.DecideManaged(file, () => folder, candidates, claims));
                string n = Show(SystemCode.Route(file, folder, candidates, claims)!);
                if (m != n) differences.Add($"{file} in {folder ?? "null"} among {string.Join(",", candidates.Select(c => c.Id))} claimed by {string.Join(",", claims.Select(c => c.Item1))}:\n  library: {n}\n  C#:      {m}");
            }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }

        private static string Show(Routed routed) => $"{routed.System?.Id ?? "-"} | {(routed.Recognition is { } c ? $"{c.SystemId} {c.Serial} {c.Region}" : "-")} | {routed.Reason ?? "-"}";

        [Fact]
        public void Every_openvgdb_rule_gives_the_same_bytes()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1983);
            var files = new List<byte[]> { Array.Empty<byte>(), new byte[] { 1 }, new byte[3], new byte[16], new byte[17], new byte[512], new byte[1536], new byte[2048] };
            foreach (int length in new[] { 4, 5, 6, 8, 16, 17, 513, 1024 + 512, 4096 + 512, 4099, 0x10000 })
                foreach (byte[] head in new[] { "NES\u001A"u8.ToArray(), "x"u8.ToArray(), new byte[] { 0x80, 0x37, 0x12, 0x40 }, new byte[] { 0x40, 0x12, 0x37, 0x80 }, new byte[] { 0x37, 0x80, 0x40, 0x12 } })
                {
                    byte[] file = new byte[length];
                    r.NextBytes(file);
                    head.AsSpan(0, Math.Min(head.Length, length)).CopyTo(file);
                    files.Add(file);
                }
            var differences = new List<string>();
            foreach (SystemEntry literal in SystemPacks.Literals)
                foreach (byte[] file in files)
                {
                    string m = Shown(() => literal.OpenVgdbBytes(file)), n = Shown(() => SystemCode.OpenVgdbBytes(literal.Id, file));
                    if (m != n) differences.Add($"{literal.Id} on {file.Length} bytes {Convert.ToHexString(file.AsSpan(0, Math.Min(4, file.Length)))}: library {n}, C# {m}");
                }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }

        [Fact]
        public void Every_header_reads_alike()
        {
            Assert.True(SystemTable.Ready, SystemTable.Report);
            var r = new Random(1989);
            var differences = new List<string>();
            for (int i = 0; i < 6000; i++)
            {
                byte[] head = new byte[16];
                r.NextBytes(head);
                if (r.Next(10) > 0) Encoding.Latin1.GetBytes("NES\u001A").CopyTo(head, 0);
                if (r.Next(2) == 0) head.AsSpan(12, 4).Clear();
                int length = r.Next(5) switch { 0 => r.Next(0, 40), 1 => 16 + head[4] * 0x4000, 2 => 16 + 512 + head[4] * 0x4000 - 1, _ => r.Next(16, 0x100000) };
                byte[] image = new byte[length];
                head.AsSpan(0, Math.Min(16, length)).CopyTo(image);
                string m = Shown(() => NesHeader.ReadManaged(image)), n = Shown(() => SystemCode.NesHeader(image));
                if (m != n) differences.Add($"nes {Convert.ToHexString(head)} {length}: library {n}, C# {m}");
            }
            for (int type = 0; type < 256; type++)
                foreach (int length in new[] { 0x14F, 0x150, 0x8000, 0x8001, 0x200000 })
                {
                    byte[] image = new byte[length];
                    r.NextBytes(image.AsSpan(0, Math.Min(length, 0x150)));
                    if (length > 0x147) image[0x147] = (byte)type;
                    if (length > 0x149) image[0x149] = (byte)r.Next(8);
                    string m = Shown(() => GbCartridge.ReadManaged(image)), n = Shown(() => SystemCode.GbHeader(image));
                    if (m != n) differences.Add($"gb type {type:X2} {length}: library {n}, C# {m}");
                }
            Assert.True(differences.Count == 0, $"{differences.Count} difference(s):\n" + string.Join("\n", differences.Take(10)));
        }
    }
}
