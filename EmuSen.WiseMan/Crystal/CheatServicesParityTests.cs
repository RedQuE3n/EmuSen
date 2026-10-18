using System.IO.Compression;
using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using EmuSen.Crystal.Cheats;
using EmuSen.Crystal.Services;
using EmuSen.Crystal.Systems;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // emusen-cheats held to the C# cheat files and database: every .cht text of the cheat tests and generated ones read with every console's codecs, cheats written, generated trees indexed, searched and pruned, and generated archives installed - see EmuSen_Backend.md §15.3.
    [Collection(TestCollections.ProcessGlobals)]
    public class CheatServicesParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCheatParity_" + Guid.NewGuid().ToString("N"));

        public CheatServicesParityTests()
        {
            Assert.True(ServiceLibrary.Ready, ServiceLibrary.Report);
            Assert.True(SystemTable.Ready, SystemTable.Report);
            Directory.CreateDirectory(_root);
        }

        public void Dispose()
        {
            foreach (string d in Directory.EnumerateDirectories(_root, "*", SearchOption.AllDirectories)) try { File.SetUnixFileMode(d, (UnixFileMode)0x1C0); } catch (IOException) { }
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void Same(string managed, string native)
        {
            if (managed != native) Assert.Fail($"C#:\n{managed}\nRust:\n{native}");
        }

        private static string Shown(Func<ChtParseResult> parse)
        {
            try
            {
                ChtParseResult r = parse();
                return $"skipped {r.Skipped}\n" + string.Join("\n", r.Cheats.Select(c => $"{c.Description}|{c.Enabled}|{c.Kind}|" + string.Join(";", c.Writes.Select(w =>
                    $"{w.Space} {w.Address} {w.Value} {w.Width} {w.Type} {w.BitPosition?.ToString() ?? "-"} {w.BigEndian} {w.RepeatCount} {w.RepeatAddAddress} {w.RepeatAddValue}"))));
            }
            catch (Exception e) { return "threw " + e.GetType().Name; }
        }

        // Every .cht text the cheat tests hold, as written in their sources.
        private static IEnumerable<string> TestTexts()
        {
            string root = ConfigRoot.Managed.Directory;
            foreach (string file in Directory.EnumerateFiles(Path.Combine(root, "EmuSen.WiseMan"), "*.cs", SearchOption.AllDirectories))
            {
                string source = File.ReadAllText(file);
                if (!source.Contains("cheats = ", StringComparison.Ordinal)) continue;
                foreach (Match m in Regex.Matches(source, "\"\"\"\\s*\\n(.*?)\\n\\s*\"\"\"", RegexOptions.Singleline))
                    if (m.Groups[1].Value.Contains("cheats", StringComparison.Ordinal)) yield return string.Join("\n", m.Groups[1].Value.Split('\n').Select(l => l.TrimStart()));
                foreach (Match m in Regex.Matches(source, "\"((?:[^\"\\\\\\n]|\\\\.)*cheats = (?:[^\"\\\\\\n]|\\\\.)*)\""))
                    yield return Regex.Unescape(m.Groups[1].Value);
            }
        }

        private static readonly string[] Keys = ["desc", "enable", "code", "address", "cheat_type", "memory_search_size", "address_bit_position", "value", "big_endian", "repeat_count", "repeat_add_to_address", "repeat_add_to_value", "handler", "unknown"];
        private static readonly string[] Numbers = ["0", "1", "2", "3", "4", "5", "-1", "65535", "4294967295", "9999999999999", " 7 ", "+3", "0x10", "1.5", "", "8\0", "8 \0", "\08", "1e3", "２"];

        // A .cht text of the format's every corner: entries in any order and case, codes of the system's codecs, explicit writes, comments, quotes, CRLF, counts past and short of the entries.
        private static string GeneratedText(Random r, IReadOnlyList<string> codes)
        {
            var lines = new List<string>();
            int entries = r.Next(0, 9);
            string count = r.Next(10) switch { 0 => "2147483647", 1 => "-2", 2 => "x", 3 => (entries + 3).ToString(), 4 => "\"" + entries + "\"", _ => entries.ToString() };
            if (r.Next(12) != 0) lines.Add((r.Next(4) == 0 ? "CHEATS" : "cheats") + " = " + count);
            for (int i = 0; i < entries + r.Next(2); i++)
            {
                string n = r.Next(15) == 0 ? "0" + i : i.ToString();
                foreach (string key in Keys.OrderBy(_ => r.Next()).Take(r.Next(1, 7)))
                {
                    string value = key switch
                    {
                        "desc" => r.Next(5) == 0 ? "\"\"" : "\"" + new[] { "Lives", "Max \"HP\"", "é ✓", "", " spaced ", "a=b" }[r.Next(6)] + "\"",
                        "enable" or "big_endian" => new[] { "true", "TRUE", "false", "yes", "" }[r.Next(5)],
                        "code" => "\"" + string.Join(r.Next(3) == 0 ? " + " : "+", Enumerable.Range(0, r.Next(1, 4)).Select(_ => codes[r.Next(codes.Count)])) + "\"",
                        _ => Numbers[r.Next(Numbers.Length)],
                    };
                    string name = $"cheat{n}_{key}";
                    lines.Add((r.Next(6) == 0 ? name.ToUpperInvariant() : name) + new[] { " = ", "=", " =", "\t=\t" }[r.Next(4)] + value);
                }
                if (r.Next(5) == 0) lines.Add(new[] { "# comment = 1", "// note = 2", "=orphan", "no equals here", "" }[r.Next(5)]);
            }
            return string.Join(r.Next(5) == 0 ? "\r\n" : "\n", lines.OrderBy(_ => r.Next(4) == 0 ? r.Next() : 0));
        }

        [Fact]
        public void Every_cht_text_reads_alike_with_every_consoles_codecs()
        {
            var managed = SystemPacks.ManagedCodecs.SelectMany(c => new[] { c.Auto, c.Explicit }).OfType<(string Id, Func<ICheatCodeCodec> Managed)>().DistinctBy(c => c.Id).ToDictionary(c => c.Id, c => c.Managed);
            NativeCheatCodec? Codec(string? id) => id is null ? null : new NativeCheatCodec(id, managed[id]());
            var pairs = SystemCode.CodecPairs.Select(p => (Auto: Codec(p.Auto), Explicit: Codec(p.Explicit))).Append((null, null)).ToArray();
            var corpus = CheatCodecParityTests.TestCodes().Concat(CheatCodecParityTests.GeneratedCodes().Take(1500)).ToArray();
            var texts = TestTexts().Distinct().ToList();
            var r = new Random(20261017);
            for (int i = 0; i < 1500; i++) texts.Add(GeneratedText(r, corpus));
            int read = 0;
            foreach (string text in texts)
                foreach (var (auto, patch) in pairs)
                {
                    string space = auto?.SpaceName ?? CheatImport.DefaultSpaceName;
                    string what = $"{auto?.Id ?? "none"}/{patch?.Id ?? "none"}:\n{text}\n";
                    Same(what + Shown(() => ChtFile.Managed.Parse(text, auto, space, patch)), what + Shown(() => CheatsNative.Parse(text, auto, space, patch)));
                    read++;
                }
            Assert.True(texts.Count > 1520 && read > 15000, $"{texts.Count} texts, {read} reads");
        }

        [Fact]
        public void Written_cheats_are_the_same_text()
        {
            var r = new Random(1717);
            for (int i = 0; i < 2000; i++)
            {
                var cheats = Enumerable.Range(0, r.Next(0, 5)).Select(_ => new ChtCheat
                {
                    Description = new[] { "A", "Max \"HP\"", "é ✓", "", "line\nbreak" }[r.Next(5)],
                    Enabled = r.Next(2) == 0,
                    Kind = r.Next(3) == 0 ? CheatKind.RomPatch : CheatKind.RamPoke,
                    Writes = Enumerable.Range(0, r.Next(0, 4)).Select(_ => new CheatWrite
                    {
                        Space = "CpuBus", Address = r.Next(5) == 0 ? -r.Next(1 << 20) : r.Next(1 << 24), Value = (uint)r.Next() * (uint)r.Next(3), Width = new[] { 0, 1, 2, 3, 4, 8 }[r.Next(6)],
                        Type = (CheatWriteType)r.Next(7), BitPosition = r.Next(3) == 0 ? r.Next(8) : null, BigEndian = r.Next(2) == 0, RepeatCount = r.Next(-1, 4),
                        RepeatAddAddress = r.Next(-3, 300), RepeatAddValue = (uint)r.Next(3),
                    }).ToArray(),
                }).ToList();
                Same(ChtFile.Managed.Write(cheats), CheatsNative.Write(cheats));
            }
        }

        // A tree of every shape the index meets: folders of systems with games in them, names that differ only in case, a folder's name repeated deeper down, links, hidden files and files of other kinds.
        private static void Plant(Random r, string tree)
        {
            var p = new Random(r.Next());
            string[] systems = ["Nintendo - SNES", "nintendo - snes", "Sega - Mega Drive", "Sony - PlayStation", "Ünïcode", "sub"];
            string[] games = ["Super Mario World (USA)", "super mario world (usa)", "Mario Paint", "Sonic.sfc", "Zelda.zip", ".hidden", "Final Fantasy VII (Disc 1)", "Ma", "MARIO", "ſonic", "Mario.zip"];
            Directory.CreateDirectory(tree);
            int files = p.Next(0, 25);
            for (int i = 0; i < files; i++)
            {
                string folder = p.Next(6) switch { 0 => tree, 1 => Path.Combine(tree, systems[p.Next(systems.Length)], "sub"), _ => Path.Combine(tree, systems[p.Next(systems.Length)]) };
                Directory.CreateDirectory(folder);
                string ext = p.Next(10) switch { 0 => ".CHT", 1 => ".txt", 2 => ".chtx", _ => ".cht" };
                File.WriteAllBytes(Path.Combine(folder, games[p.Next(games.Length)] + ext), new byte[p.Next(0, 300)]);
            }
            if (p.Next(4) == 0)
            {
                Directory.CreateSymbolicLink(Path.Combine(tree, "Linked"), Directory.CreateDirectory(Path.Combine(tree + "-outside", "Linked")).FullName);
                File.WriteAllBytes(Path.Combine(tree + "-outside", "Linked", "Outside Game.cht"), new byte[7]);
            }
            if (p.Next(5) == 0) File.CreateSymbolicLink(Path.Combine(tree, "Dangling.cht"), Path.Combine(tree, "nowhere"));
            if (p.Next(3) == 0)
            {
                string folder = Directory.CreateDirectory(Path.Combine(tree, systems[p.Next(systems.Length)])).FullName;
                File.WriteAllBytes(Path.Combine(Directory.CreateDirectory(tree + "-outside").FullName, "big.bin"), new byte[p.Next(1000, 5000)]);
                File.CreateSymbolicLink(Path.Combine(folder, "Linked Game.cht"), Path.Combine(tree + "-outside", "big.bin"));
            }
            if (p.Next(12) == 0) { string closed = Directory.CreateDirectory(Path.Combine(tree, "Closed")).FullName; File.WriteAllBytes(Path.Combine(closed, "x.cht"), [1]); File.SetUnixFileMode(closed, UnixFileMode.None); }
        }

        private static string Entries(IEnumerable<CheatDatabaseEntry> entries, string tree) => string.Join("\n", entries.Select(e => $"{e.System}|{e.Game}|{e.Path.Replace(tree, "<tree>")}"));

        [Fact]
        public void Generated_trees_are_indexed_searched_and_pruned_alike()
        {
            string[] needles = ["mario", "MARIO", "mario paint.sfc", "super mario world (usa).zip", "  sonic  ", "sonic", "Ma", "", " ", "zelda", "ſonic", "nothing"];
            string[][] keeps = [[], ["Nintendo - SNES"], ["nintendo - SNES", "Sega - Mega Drive"], ["Absent"], ["Nintendo - SNES", "Sega - Mega Drive", "Sony - PlayStation", "Ünïcode", "sub", "Linked", "Closed", "Database"]];
            int searched = 0, applied = 0;
            for (int seed = 1; seed <= 80; seed++)
            {
                string m = Path.Combine(_root, $"m{seed}", "Database"), n = Path.Combine(_root, $"n{seed}", "Database");
                Plant(new Random(seed), m);
                Plant(new Random(seed), n);
                CheatDatabase dm = CheatDatabase.Managed(m), dn = CheatDatabase.Library(n);
                Assert.True(dn.Native);
                string what = $"seed {seed}\n";
                Same(what + Entries(dm.All(), m), what + Entries(dn.All(), n));
                Same(what + string.Join(";", dm.Systems()), what + string.Join(";", dn.Systems()));
                foreach (var (system, _) in dm.Systems().Append(("SUB", 0)).Append(("absent", 0)))
                    foreach (string? filter in new[] { null, "", " mario ", "MARIO", "x" })
                        Same(what + Entries(dm.Games(system, filter), m), what + Entries(dn.Games(system, filter), n));
                foreach (string needle in needles)
                    foreach (int limit in new[] { 20, 1, 0, -1 })
                    {
                        Same(what + needle + Entries(dm.Find(needle, limit), m), what + needle + Entries(dn.Find(needle, limit), n));
                        searched++;
                    }
                string[] keep = keeps[seed % keeps.Length];
                CheatPrunePlan pm = CheatDatabasePruner.ManagedPlan(dm, keep), pn = CheatDatabasePruner.Plan(dn, keep);
                string Plan(CheatPrunePlan p, string tree) => $"{p.CanApply} [{p.Reason?.Replace(tree, "<tree>")}] keep {string.Join(",", p.Keeping)} remove " + string.Join(";", p.Removing.Select(s => $"{s.System} {s.Path.Replace(tree, "<tree>")} {s.Files} {s.Bytes}"));
                Same(what + Plan(pm, m), what + Plan(pn, n));
                var (removedM, failedM) = CheatDatabasePruner.ManagedApply(dm, pm);
                var (removedN, failedN) = pn.CanApply ? CheatsNative.Apply(n, pn) : (0, Array.Empty<string>());
                string Failed(IReadOnlyList<string> failed) => string.Join(";", failed.Select(f => f[..(f.IndexOf(" (", StringComparison.Ordinal) + 1)]));
                Same(what + $"{removedM} {Failed(failedM)}\n" + Tree(Path.GetDirectoryName(m)!, times: false), what + $"{removedN} {Failed(failedN)}\n" + Tree(Path.GetDirectoryName(n)!, times: false));
                applied += removedM;
            }
            Assert.True(searched > 3000 && applied > 20, $"{searched} searches, {applied} folders pruned");
        }

        private static string Tree(string tree, bool times = true) => string.Join("\n", Directory.EnumerateFileSystemEntries(tree, "*", new EnumerationOptions { RecurseSubdirectories = true, IgnoreInaccessible = true }).Order(StringComparer.Ordinal).Select(p =>
        {
            string name = Path.GetRelativePath(tree, p);
            var info = new FileInfo(p);
            if (info.LinkTarget is not null) return name + " -> link";
            if (Directory.Exists(p)) return name + "/";
            // A file left empty by an entry that failed was truncated when each half reached it, so its time is not compared.
            try { return $"{name} {Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(p)))[..12]} {Convert.ToString((int)File.GetUnixFileMode(p), 8)} {(info.Length == 0 || !times ? "" : File.GetLastWriteTimeUtc(p).ToString("o"))}"; }
            catch (UnauthorizedAccessException) { return name + " unreadable"; }
        }));

        private static readonly DateTime[] Times = [new(2026, 3, 8, 2, 30, 0), new(2026, 11, 1, 1, 30, 0), new(2026, 7, 1, 12, 0, 2), new(1980, 1, 1, 0, 0, 0), new(2107, 12, 31, 23, 59, 58), new(2001, 2, 3, 4, 5, 6)];

        // An archive of every entry the install meets, written as .NET writes one, then damaged where asked.
        private static byte[] Archive(Random r)
        {
            string[] names = ["cht/SNES/Game.cht", "cht/SNES/game.CHT", "cht/PS/Other.cht", "../escaped.cht", "/absolute.cht", "cht/../inside.cht", "cht\\back\\slash.cht", "cht/dir/", "README.md", "cht/SNES/notes.txt",
                ".cht", "cht/é ✓/Ünïcode.cht", "cht/SNES/Game.cht", "cht/x/../../../out.cht", "cht/a:b.cht", "cht/sp ace/ end .cht", ""];
            var stream = new MemoryStream();
            using (var zip = new ZipArchive(stream, ZipArchiveMode.Create, leaveOpen: true))
                for (int i = 0, n = r.Next(1, 9); i < n; i++)
                {
                    string name = names[r.Next(names.Length)];
                    if (name.Length == 0) name = "cht/SNES/Fill" + i + ".cht";
                    ZipArchiveEntry e = zip.CreateEntry(name, r.Next(3) == 0 ? CompressionLevel.NoCompression : CompressionLevel.Optimal);
                    e.LastWriteTime = new DateTimeOffset(Times[r.Next(Times.Length)], TimeSpan.Zero);
                    e.ExternalAttributes = r.Next(4) switch { 0 => 0, 1 => unchecked((int)0x81ED0000u), 2 => unchecked((int)0x81000000u), _ => unchecked((int)0x81A40000u) };
                    if (!name.EndsWith('/')) using (Stream s = e.Open()) s.Write(Encoding.UTF8.GetBytes(new string((char)('a' + r.Next(26)), r.Next(0, 3000))));
                }
            byte[] bytes = stream.ToArray();
            if (r.Next(6) == 0)
            {
                int local = bytes.AsSpan().IndexOf("PK\u0003\u0004"u8);
                if (r.Next(2) == 0) bytes[local] = 0;
                else
                {
                    int data = local + 30 + BitConverter.ToUInt16(bytes, local + 26) + BitConverter.ToUInt16(bytes, local + 28);
                    if (BitConverter.ToUInt16(bytes, local + 8) == 8 && BitConverter.ToUInt32(bytes, local + 18) > 2) { bytes[data] = 0xFF; bytes[data + 1] = 0xFF; }
                }
            }
            if (r.Next(25) == 0) bytes = bytes[..(bytes.Length / 2)];
            return bytes;
        }

        private static string Installed(Func<CheatDatabaseInstallResult> install)
        {
            try { CheatDatabaseInstallResult x = install(); return $"{x.Installed} installed, {x.Skipped} skipped"; }
            catch (Exception e) { return "threw " + e.GetType().Name; }
        }

        [Fact]
        public void Generated_archives_install_the_same_files_with_the_same_modes_and_times()
        {
            var r = new Random(4242);
            int installed = 0;
            for (int i = 0; i < 300; i++)
            {
                byte[] zip = Archive(r);
                string over = r.Next(4) == 0 ? "/" : "";
                string m = Path.Combine(_root, "zm" + i, "Database"), n = Path.Combine(_root, "zn" + i, "Database");
                bool before = r.Next(3) == 0;
                foreach (string tree in new[] { m, n })
                    if (before)
                    {
                        Directory.CreateDirectory(Path.Combine(tree, "cht", "SNES"));
                        File.WriteAllText(Path.Combine(tree, "cht", "SNES", "Game.cht"), "old");
                        File.SetUnixFileMode(Path.Combine(tree, "cht", "SNES", "Game.cht"), (UnixFileMode)0x1B6);
                        File.SetLastWriteTimeUtc(Path.Combine(tree, "cht", "SNES", "Game.cht"), new DateTime(2020, 1, 1, 0, 0, 0, DateTimeKind.Utc));
                    }
                string shownM = Installed(() => CheatDatabaseInstaller.ManagedInstall(new MemoryStream(zip), m + over));
                string shownN = Installed(() => CheatsNative.Install(zip, n + over) ?? CheatDatabaseInstaller.ManagedInstall(new MemoryStream(zip), n + over));
                string what = $"archive {i}\n";
                Same(what + shownM + "\n" + Tree(Path.GetDirectoryName(m)!), what + shownN + "\n" + Tree(Path.GetDirectoryName(n)!));
                installed += shownM.StartsWith("threw", StringComparison.Ordinal) ? 0 : int.Parse(shownM.Split(' ')[0]);
            }
            Assert.True(installed > 400, $"{installed} files installed");
        }
    }
}
