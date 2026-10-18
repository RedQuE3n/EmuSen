using EmuSen.Common.Firmware;
using EmuSen.Crystal.Services;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Common
{
    // emusen-firmware held to the C# store on generated trees: the folders' order and its reports, every image found whole, split or by content, the overview's facts, and installs - see EmuSen_Backend.md §15.1 and EmuSen_Firmware.md §7.
    [Collection(TestCollections.ProcessGlobals)]
    public class FirmwareParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenFirmwareParity_" + Guid.NewGuid().ToString("N"));
        private readonly string? _config = ConfigStore.OverrideDirectory, _data = DataStore.OverrideDirectory;
        private readonly Action<string>? _sink = ConfigDiagnostics.Sink;
        private readonly List<string> _said = new();

        public FirmwareParityTests()
        {
            Assert.True(ServiceLibrary.Ready, ServiceLibrary.Report);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            FirmwareLibrary.ResetDirectory();
            Directory.CreateDirectory(DataStore.Firmware);
            ConfigDiagnostics.Sink = _said.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = _sink;
            ConfigStore.OverrideDirectory = _config;
            DataStore.OverrideDirectory = _data;
            foreach (string path in Directory.Exists(_root) ? Directory.EnumerateFileSystemEntries(_root, "*", SearchOption.AllDirectories) : Array.Empty<string>())
                try { if (!File.GetAttributes(path).HasFlag(FileAttributes.ReparsePoint)) File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite | UnixFileMode.UserExecute); } catch (IOException) { }
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static void Same(string managed, string native)
        {
            if (managed != native) Assert.Fail($"C#:   {managed}\nRust: {native}");
        }

        private static string Shown(FirmwareFound? f) => f is null ? "none" : $"{f.Folder} [{string.Join(",", f.Names)}] {string.Join(",", f.Parts.Select(p => Convert.ToHexString(System.Security.Cryptography.SHA256.HashData(p))[..16] + ":" + p.Length))}{(f.ByContent ? " by content" : "")}";

        private static string Shown((FirmwareLocated? Found, string? Wrong) l) => l.Found is null ? $"none, wrong in {l.Wrong ?? "none"}" : $"{l.Found.Folder} [{string.Join(",", l.Found.Names)}] {l.Found.Whole}{(l.Found.ByContent ? " by content" : "")}";

        private static string Md5(byte[] bytes) => Convert.ToHexStringLower(System.Security.Cryptography.MD5.HashData(bytes));

        // A value of every kind §7.7 names, in an order that leaves each half's memory of what it said in the same state.
        private IEnumerable<string?> Settings()
        {
            string file = Path.Combine(_root, "a-file");
            File.WriteAllBytes(file, [1]);
            string chosen = Directory.CreateDirectory(Path.Combine(_root, "BIOS")).FullName;
            return new string?[]
            {
                null, "", "  ", "\t", chosen, chosen + "/", " " + chosen + " ", chosen + "/./x/..", "relative", "./BIOS", "~someone/x", "relative",
                "~", "~/", "~/EmuSenParityNoSuchFolder", "~/EmuSenParityNoSuchFolder", file, file, Path.Combine(_root, "absent", "x"), chosen,
                DataStore.Firmware, DataStore.Firmware + "/", DataStore.Firmware + "//", Path.Combine(DataStore.Firmware, "..", "Firmware"), "/", "relative", null,
            };
        }

        [Fact]
        public void The_folders_and_what_is_said_of_each_setting_match()
        {
            var settings = Settings().ToArray();
            string home = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            var managed = new List<string>();
            foreach (string? value in settings) managed.Add($"{value}: {string.Join(" | ", FirmwareLibrary.Managed.Folders(new AppSettings { FirmwareDirectory = value }))}");
            var saidManaged = _said.ToList();
            _said.Clear();
            var native = new List<string>();
            foreach (string? value in settings) native.Add($"{value}: {string.Join(" | ", FirmwareNative.Folders(value, home, DataStore.Firmware)!)}");
            Assert.Equal(managed, native);
            Same(string.Join("\n", saidManaged), string.Join("\n", _said));
            Assert.True(saidManaged.Count >= 4, string.Join("\n", saidManaged));
        }

        // One generated tree: three folders, files of the right size, the wrong one, a folder or a link where a file is asked for, and files that will not open.
        private (FirmwareRequest[] Requests, string[] Folders) Tree(int seed)
        {
            var random = new Random(seed);
            string top = Path.Combine(_root, "t" + seed);
            string[] folders = new[] { "a", "b", "c" }.Select(n => Directory.CreateDirectory(Path.Combine(top, n)).FullName).ToArray();
            string[] pool = ["x.rom", "x.bin", "X.ROM", "y.rom", "lo.rom", "hi.rom", "mid.rom", "z part.rom", "é.rom"];
            var requests = new List<FirmwareRequest>();
            for (int i = 0; i < 6; i++)
            {
                int size = random.Next(5) switch { 0 => 0, 1 => -1, 2 => 4096, _ => random.Next(1, 64) };
                string[] names = pool.OrderBy(_ => random.Next()).ToArray();
                var parts = Enumerable.Range(0, random.Next(3)).Select(f => (IReadOnlyList<string>)names.Skip(2 + f).Take(random.Next(1, 4)).ToArray()).ToArray();
                requests.Add(new FirmwareRequest("TESTCORE", "CHIP" + i, names[0], size, "a made-up chip") { AlternateNames = names.Skip(1).Take(random.Next(3)).ToArray(), Parts = parts });
            }
            foreach (string folder in folders)
                foreach (string name in pool)
                {
                    string path = Path.Combine(folder, name);
                    int kind = random.Next(10);
                    int want = requests[random.Next(requests.Count)].Size;
                    int length = kind switch { 0 or 1 => Math.Max(0, want), 2 => Math.Max(0, want) + 1, 3 => random.Next(0, 40), 4 => Math.Max(0, want / 2), _ => -1 };
                    if (length >= 0) File.WriteAllBytes(path, Enumerable.Range(0, length).Select(_ => (byte)random.Next(256)).ToArray());
                    else if (kind == 5) Directory.CreateDirectory(path);
                    else if (kind == 6) File.CreateSymbolicLink(path, Path.Combine(top, "nowhere"));
                    else if (kind == 7 && File.Exists(Path.Combine(folders[0], pool[0]))) File.CreateSymbolicLink(path, Path.Combine(folders[0], pool[0]));
                    else if (kind == 8) { File.WriteAllBytes(path, new byte[Math.Max(0, want)]); File.SetUnixFileMode(path, UnixFileMode.None); }
                }
            // Checksums of files the tree holds, of every size, among made-up ones, for some requests: what step 3 of EmuSen_Firmware.md §7.4 reads.
            string[] digests = folders.SelectMany(f => Directory.EnumerateFiles(f)).Where(p => new FileInfo(p).LinkTarget is null && File.GetUnixFileMode(p) != UnixFileMode.None)
                .Select(p => Md5(File.ReadAllBytes(p))).Concat(["0123456789abcdef0123456789abcdef"]).ToArray();
            for (int i = 0; i < requests.Count; i++)
                if (random.Next(3) != 0) requests[i] = requests[i] with { Md5 = digests.OrderBy(_ => random.Next()).Take(random.Next(1, 6)).ToArray() };
            return (requests.ToArray(), random.Next(3) switch { 0 => folders, 1 => [folders[2], folders[0]], _ => [folders[1]] });
        }

        [Fact]
        public void Every_image_of_generated_trees_is_found_and_located_alike()
        {
            int found = 0, split = 0, wrong = 0, content = 0;
            for (int seed = 1; seed <= 150; seed++)
            {
                var (requests, folders) = Tree(seed);
                foreach (FirmwareRequest r in requests)
                {
                    FirmwareFound? m = FirmwareLibrary.Managed.Find(r, folders);
                    string what = $"seed {seed}: {r.FileName} {r.Size} [{string.Join(",", r.AlternateNames)}] [{string.Join(";", r.Parts.Select(p => string.Join(",", p)))}] [{string.Join(",", r.Md5.Select(d => d[..6]))}] in {string.Join(",", folders.Select(Path.GetFileName))}: ";
                    Same(what + Shown(m), what + Shown(FirmwareNative.Find(r, folders)));
                    var located = FirmwareLibrary.Managed.Locate(r, folders);
                    Same(what + Shown(located), what + Shown(FirmwareNative.Locate(r, folders)));
                    found += m is null ? 0 : 1;
                    split += m is { Whole: false } ? 1 : 0;
                    wrong += located.WrongSizeIn is null ? 0 : 1;
                    content += m is { ByContent: true } ? 1 : 0;
                }
            }
            Assert.True(found > 100 && split > 10 && wrong > 50 && content > 20, $"found {found}, split {split}, wrong {wrong}, by content {content}");
        }

        // The checksum cases of EmuSen_Firmware.md §7.7 on stand-ins of random bytes whose MD5 the test declares: each half answers as the rule says.
        [Fact]
        public void A_file_is_recognised_by_its_content_alike_in_both_halves()
        {
            var random = new Random(7);
            byte[] Bytes(int n) { var b = new byte[n]; random.NextBytes(b); return b; }
            string Folder(string name) => Directory.CreateDirectory(Path.Combine(_root, "content", name)).FullName;
            string chosen = Folder("chosen"), store = Folder("store"), elsewhere = Folder("elsewhere"), absent = Path.Combine(_root, "content", "absent");
            byte[] image = Bytes(64), other = Bytes(64), second = Bytes(64), shortOne = Bytes(63);
            var r = new FirmwareRequest("TESTCORE", "WIDGET", "widget.rom", 64, "a made-up chip") { AlternateNames = ["w.bin"], Md5 = [Md5(image), Md5(shortOne)] };
            void Expect(string what, IReadOnlyList<string> folders, string? folder, string? name, bool byContent, byte[]? bytes = null)
            {
                FirmwareFound? m = FirmwareLibrary.Managed.Find(r, folders), n = FirmwareNative.Find(r, folders);
                Same(what + ": " + Shown(m), what + ": " + Shown(n));
                Same(what + ": " + Shown(FirmwareLibrary.Managed.Locate(r, folders)), what + ": " + Shown(FirmwareNative.Locate(r, folders)));
                Assert.Equal((what, folder, name, byContent), (what, m?.Folder, m?.Names[0], m?.ByContent ?? false));
                if (bytes is not null) Assert.Equal(bytes, m!.Parts[0]);
            }
            File.WriteAllBytes(Path.Combine(chosen, "other content.bin"), other);
            File.WriteAllBytes(Path.Combine(chosen, "short.bin"), shortOne);
            Expect("another content of the size, and a listed one of another size", [chosen, absent], null, null, false);
            File.WriteAllBytes(Path.Combine(chosen, "PSX - PLAYER COPY.BIN"), image);
            Expect("a renamed file", [absent, chosen], chosen, "PSX - PLAYER COPY.BIN", true, image);
            File.WriteAllBytes(Path.Combine(store, "widget.rom"), second);
            Expect("the chosen folder by content before the store by name", [chosen, store], chosen, "PSX - PLAYER COPY.BIN", true, image);
            File.WriteAllBytes(Path.Combine(chosen, "w.bin"), second);
            Expect("a name before content in one folder", [chosen, store], chosen, "w.bin", false, second);
            File.Delete(Path.Combine(chosen, "w.bin"));
            File.Delete(Path.Combine(chosen, "PSX - PLAYER COPY.BIN"));
            File.WriteAllBytes(Path.Combine(elsewhere, "target"), image);
            File.CreateSymbolicLink(Path.Combine(chosen, "linked.bin"), Path.Combine(elsewhere, "target"));
            File.CreateSymbolicLink(Path.Combine(chosen, "dangling.bin"), Path.Combine(elsewhere, "nothing"));
            Expect("a link, by its target's size and content", [chosen], chosen, "linked.bin", true, image);
            Directory.CreateDirectory(Path.Combine(chosen, "inner"));
            File.Move(Path.Combine(elsewhere, "target"), Path.Combine(chosen, "inner", "deep.bin"));
            Expect("a subfolder is not searched, nor a missing folder", [chosen, absent], null, null, false);
            Expect("the store by name, all else failing", [chosen, store], store, "widget.rom", false, second);
            string order = Folder("order");
            File.WriteAllBytes(Path.Combine(order, "b.bin"), image);
            File.WriteAllBytes(Path.Combine(order, "a.bin"), other);
            r = r with { Md5 = [Md5(image), Md5(other)] };
            Expect("the checksums in the core's order before the names'", [order], order, "b.bin", true, image);
            File.Copy(Path.Combine(order, "b.bin"), Path.Combine(order, "B.bin"));
            Expect("then the names in ordinal order", [order], order, "B.bin", true, image);
        }

        // A file rewritten in place under its name is hashed again by each half, whose memory of checksums is keyed by the modification time too - see EmuSen_Firmware.md §7.6.
        [Fact]
        public void A_file_rewritten_in_place_is_recognised_by_its_new_content()
        {
            string folder = Directory.CreateDirectory(Path.Combine(_root, "rewritten")).FullName, file = Path.Combine(folder, "copy.bin");
            byte[] first = Enumerable.Range(0, 64).Select(i => (byte)i).ToArray(), then = Enumerable.Range(0, 64).Select(i => (byte)(i * 7)).ToArray();
            var r = new FirmwareRequest("TESTCORE", "WIDGET", "widget.rom", 64, "a made-up chip") { Md5 = [Md5(then)] };
            var old = r with { Md5 = [Md5(first)] };
            File.WriteAllBytes(file, first);
            Assert.Equal((true, true), (FirmwareLibrary.Managed.Find(old, [folder]) is not null, FirmwareNative.Find(old, [folder]) is not null));
            File.WriteAllBytes(file, then);
            File.SetLastWriteTimeUtc(file, DateTime.UtcNow.AddMinutes(1));
            Assert.Equal((false, false), (FirmwareLibrary.Managed.Find(old, [folder]) is not null, FirmwareNative.Find(old, [folder]) is not null));
            Assert.Equal((true, true), (FirmwareLibrary.Managed.Find(r, [folder]) is { ByContent: true }, FirmwareNative.Find(r, [folder]) is { ByContent: true }));
        }

        [Fact]
        public void Installs_write_the_same_files_and_refuse_alike()
        {
            var r = new FirmwareRequest("TESTCORE", "WIDGET", "widget.rom", 64, "a made-up chip");
            string picks = Directory.CreateDirectory(Path.Combine(_root, "picks")).FullName;
            File.WriteAllBytes(Path.Combine(picks, "right"), Enumerable.Range(0, 64).Select(i => (byte)i).ToArray());
            File.WriteAllBytes(Path.Combine(picks, "short"), new byte[63]);
            File.WriteAllBytes(Path.Combine(picks, "locked"), new byte[64]);
            File.SetUnixFileMode(Path.Combine(picks, "locked"), UnixFileMode.None);
            string blocked = Path.Combine(_root, "blocked");
            File.WriteAllBytes(blocked, [1]);
            foreach (string source in new[] { "right", "short", "locked", "absent" })
                foreach (string store in new[] { "store", "deep/store", "blocked" })
                {
                    string managedStore = store == "blocked" ? blocked : Path.Combine(_root, "m-" + source, store), nativeStore = store == "blocked" ? blocked : Path.Combine(_root, "n-" + source, store);
                    FirmwareLibrary.Directory = managedStore;
                    bool m = FirmwareLibrary.Managed.Install(r, Path.Combine(picks, source));
                    bool n;
                    try { n = FirmwareNative.Install(r, Path.Combine(picks, source), nativeStore); }
                    catch (ServiceFault fault) when (fault.Status == ServiceFault.Io) { n = false; }
                    FirmwareLibrary.ResetDirectory();
                    Assert.Equal((source, store, m), (source, store, n));
                    string Contents(string s) => File.Exists(Path.Combine(s, "widget.rom")) ? Convert.ToHexString(File.ReadAllBytes(Path.Combine(s, "widget.rom"))) : "none";
                    Assert.Equal(Contents(managedStore), Contents(nativeStore));
                }
        }

        // The facade under the switch reaches the same answers as the C# through the public calls.
        [Fact]
        public void The_public_calls_answer_as_the_managed_store()
        {
            string chosen = Directory.CreateDirectory(Path.Combine(_root, "Chosen")).FullName;
            new AppSettings { FirmwareDirectory = chosen }.Save();
            var r = new FirmwareRequest("TESTCORE", "WIDGET", "widget.rom", 8, "a made-up chip") { Parts = [["w.lo", "w.hi"]] };
            File.WriteAllBytes(Path.Combine(chosen, "w.lo"), [1, 2, 3, 4]);
            File.WriteAllBytes(Path.Combine(chosen, "w.hi"), [5, 6, 7, 8]);
            Assert.Equal([chosen, DataStore.Firmware], FirmwareLibrary.Folders);
            Assert.Equal(Shown(FirmwareLibrary.Managed.Find(r, FirmwareLibrary.Folders)), Shown(FirmwareLibrary.Find(r)));
            Assert.Equal([1, 2, 3, 4], FirmwareLibrary.TryLoadParts(r)![0]);
            Assert.Null(FirmwareLibrary.TryLoad(r));
            Assert.True(FirmwareLibrary.IsInstalled(r));
            Assert.Empty(FirmwareLibrary.MissingFrom([r]));
        }
    }
}
