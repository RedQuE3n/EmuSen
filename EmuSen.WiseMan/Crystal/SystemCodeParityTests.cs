using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using EmuSen.WiseMan.Bindings.Systems;
using EmuSen.WiseMan.Bindings.Systems.GameBoy;
using EmuSen.WiseMan.Bindings.Systems.Nes;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // The recognisers, routing, header readers and OpenVGDB rules in emusen-systems held on generated discs, cartridges and files to the C#'s answers as recorded before it was deleted, the bytes each recogniser reads included - see EmuSen_Backend.md §13.3, §40.1.
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
            discs.Add(("ps2 cd iso", SyntheticPs2Disc.Cd().Iso));
            discs.Add(("ps2 cd raw", SyntheticPs2Disc.Cd().DataTrack));
            discs.Add(("ps2 cd licensed", SyntheticPs2Disc.Cd(licence: true).Iso));
            discs.Add(("ps2 dvd", SyntheticPs2Disc.Dvd().Iso));
            discs.Add(("ps2 no boot2", SyntheticPs2Disc.Dvd("BOOT = cdrom:\\X_123.45;1\r\n").Iso));
            discs.Add(("sega cd", SyntheticMdRom.Disc()));
            discs.Add(("sega cd europe", SyntheticMdRom.Disc("E", 0x20000)));
            discs.Add(("genesis", SyntheticMdRom.Cartridge()));
            discs.Add(("32x", SyntheticMdRom.Cartridge("SEGA 32X")));
            discs.Add(("32x at the name's end", SyntheticMdRom.Cartridge("SEGA SATURN 32X")));
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
                foreach (int cut in new[] { 0, 12, 15, 16, 30, 0x40, 0x1FF, 0x200, 0x8005, 0x8006, 0x8010, 8223, 8224, 9463, 9464, 12 * 1024, 20 * 2048, 21 * 2048 + 10, 40 * 1024, 300 * 2048, 301 * 2048 + 10 })
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
        public void Every_recogniser_answers_as_recorded_and_reads_the_same_bytes()
        {
            var golden = new ParityGolden("Recognisers");
            foreach (var (label, image) in Images())
                foreach (SystemEntry entry in SystemPacks.All.Select(p => p.Entry).Where(e => e.Recognise is not null))
                {
                    Probe probe = new(image);
                    golden.Add($"{entry.Id} on {label}", Shown(() => entry.Recognise!(probe)) + " reads " + string.Join(",", probe.Asked));
                }
            golden.Check(1000);
        }

        [Fact]
        public void Routing_decides_as_recorded_among_any_candidates_claims_and_folders()
        {
            var r = new Random(84);
            var entries = SystemPacks.All.Select(p => p.Entry).ToArray();
            string?[] folders = { null, "", "roms", "psx", "PSX", "segacd", "SegaCD", "genesis", "nes", "n64", "gbc", "Ps×" };
            var golden = new ParityGolden("Routing");
            for (int i = 0; i < 4000; i++)
            {
                var candidates = entries.OrderBy(_ => r.Next()).Take(r.Next(0, 5)).ToArray();
                var claims = Enumerable.Range(0, candidates.Length).Where(_ => r.Next(3) == 0)
                    .Select(at => (at, new Recognition(candidates[at].Id, r.Next(2) == 0 ? null : $"SLUS-{r.Next(100000):D5}", r.Next(3) switch { 0 => null, 1 => "Japan", _ => "Europe" }))).ToArray();
                string? folder = folders[r.Next(folders.Length)];
                string file = r.Next(4) == 0 ? "Café Game (Disc 1).cue" : $"game{i}.iso";
                golden.Add($"{file} in {folder ?? "null"} among {string.Join(",", candidates.Select(c => c.Id))} claimed by {string.Join(",", claims.Select(c => $"{c.Item1}:{c.Item2.SystemId}:{c.Item2.Serial}:{c.Item2.Region}"))}", Show(SystemCode.Route(file, folder, candidates, claims)));
            }
            golden.Check(4000);
        }

        // A 32X cartridge named .bin goes to the 32X by its header and a Genesis cartridge to the Genesis, wherever it sits - see EmuSen_Backend.md §32.
        [Fact]
        public void A_32x_cartridge_named_bin_routes_to_the_32x()
        {
            var candidates = SystemPacks.All.Select(p => p.Entry).Where(e => e.Extensions.Contains(".bin")).ToArray();
            Assert.Equal(new[] { "md", "32x", "psx", "ps2" }, candidates.Select(c => c.Id));
            foreach (var (system, expected) in new[] { ("SEGA 32X", "32x"), ("SEGA GENESIS", "md"), ("SEGA MEGA DRIVE", "md") })
            {
                byte[] image = SyntheticMdRom.Cartridge(system);
                var claims = Enumerable.Range(0, candidates.Length).Select(at => (at, candidates[at].Recognise?.Invoke(new Probe(image)))).Where(c => c.Item2 is not null).Select(c => (c.at, c.Item2!)).ToArray();
                foreach (string? folder in new[] { null, "genesis", "sega32x", "roms" })
                    Assert.Equal(expected, SystemCode.Route("game.bin", folder, candidates, claims).System?.Id);
            }
        }

        private static string Show(Routed routed) => $"{routed.System?.Id ?? "-"} | {(routed.Recognition is { } c ? $"{c.SystemId} {c.Serial} {c.Region}" : "-")} | {routed.Reason ?? "-"}";

        [Fact]
        public void Every_openvgdb_rule_gives_the_recorded_bytes()
        {
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
            var golden = new ParityGolden("OpenVgdb");
            foreach (SystemEntry entry in SystemPacks.All.Select(p => p.Entry))
                foreach (byte[] file in files)
                    golden.Add($"{entry.Id} on {file.Length} bytes {Convert.ToHexString(file.AsSpan(0, Math.Min(4, file.Length)))}", Shown(() => entry.OpenVgdbBytes(file)));
            golden.Check(100);
        }

        [Fact]
        public void Every_header_reads_as_recorded()
        {
            var r = new Random(1989);
            var golden = new ParityGolden("Headers");
            for (int i = 0; i < 6000; i++)
            {
                byte[] head = new byte[16];
                r.NextBytes(head);
                if (r.Next(10) > 0) Encoding.Latin1.GetBytes("NES\u001A").CopyTo(head, 0);
                if (r.Next(2) == 0) head.AsSpan(12, 4).Clear();
                int length = r.Next(5) switch { 0 => r.Next(0, 40), 1 => 16 + head[4] * 0x4000, 2 => 16 + 512 + head[4] * 0x4000 - 1, _ => r.Next(16, 0x100000) };
                byte[] image = new byte[length];
                head.AsSpan(0, Math.Min(16, length)).CopyTo(image);
                golden.Add($"nes {Convert.ToHexString(head)} {length}", Shown(() => NesHeader.Read(image)));
            }
            for (int type = 0; type < 256; type++)
                foreach (int length in new[] { 0x14F, 0x150, 0x8000, 0x8001, 0x200000 })
                {
                    byte[] image = new byte[length];
                    r.NextBytes(image.AsSpan(0, Math.Min(length, 0x150)));
                    if (length > 0x147) image[0x147] = (byte)type;
                    if (length > 0x149) image[0x149] = (byte)r.Next(8);
                    golden.Add($"gb type {type:X2} {length}", Shown(() => GbCartridge.Read(image)));
                }
            golden.Check(7000);
        }
    }
}
