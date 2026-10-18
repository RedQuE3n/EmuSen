using System.Security.Cryptography;
using EmuSen.Crystal.SaveMedia;
using EmuSen.Crystal.Services;
using EmuSen.Crystal.Systems.Genesis;
using EmuSen.Crystal.Systems.N64;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Crystal
{
    // emusen-savemedia held to the C# card manager on generated trees of every medium's cards, free and held, each script of operations run on a copy of the tree by each half, outcomes and files compared - see EmuSen_Backend.md §14.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class SaveMediaManagerParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCardManagerParity_" + Guid.NewGuid().ToString("N"));
        private readonly Action<string>? _sink = ConfigDiagnostics.Sink;
        private readonly List<string> _said = new();
        private readonly ITestOutputHelper _output;

        private static readonly string[] Extensions = [".mcr", ".mpk", ".brm", ".crm"];

        public SaveMediaManagerParityTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(ServiceLibrary.Ready, ServiceLibrary.Report);
            Assert.True(PlatformLibrary.GalaxiaActive, "the lock's parity needs Galaxia's library");
            Directory.CreateDirectory(_root);
            ConfigDiagnostics.Sink = _said.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = _sink;
            foreach (string path in Directory.EnumerateFiles(_root, "*", SearchOption.AllDirectories)) try { File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite); } catch (IOException) { }
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static byte[] RandomImage(Random r, string extension) => extension switch
        {
            ".mcr" => SaveMediaParityTests.RandomCard(r, PsxCardFormat.Instance.Managed),
            ".mpk" => SaveMediaParityTests.RandomPak(r),
            ".brm" => SaveMediaParityTests.RandomBram(r, SegaCdBramFormat.BackupRam.Managed),
            _ => SaveMediaParityTests.RandomBram(r, SegaCdBramFormat.RamCartridge.Managed),
        };

        // One generated tree written twice, under C# and under Rust: cards of every medium, blank and empty files, a file that will not open, a folder named as a card, a file of no medium.
        private static void Plant(Random r, string tree)
        {
            var plan = new Random(r.Next());
            Directory.CreateDirectory(tree);
            foreach (string ext in Extensions)
                for (int i = 0; i < 3; i++)
                {
                    string path = Path.Combine(tree, $"{"ABC"[i]}{ext}");
                    switch (plan.Next(8))
                    {
                        case 0: break;
                        case 1: File.WriteAllBytes(path, []); break;
                        case 2: File.WriteAllBytes(path, new byte[plan.Next(1, 3) == 1 ? 64 : 131072]); break;
                        default: File.WriteAllBytes(path, RandomImage(plan, ext)); break;
                    }
                }
            File.WriteAllBytes(Path.Combine(tree, "Locked.mcr"), PsxCardFormat.Instance.Managed.Format()!);
            File.SetUnixFileMode(Path.Combine(tree, "Locked.mcr"), UnixFileMode.None);
            Directory.CreateDirectory(Path.Combine(tree, "Folder.mpk"));
            File.WriteAllText(Path.Combine(tree, "notes.txt"), "not a card");
        }

        private static string Shown(Func<MediaOutcome> op)
        {
            try { MediaOutcome o = op(); return $"{o.Done}: {o.Words}"; }
            catch (Exception e) { return "threw " + e.GetType().Name; }
        }

        private static string Shown(Func<MediumCard?> open)
        {
            try
            {
                MediumCard? c = open();
                if (c is null) return "no medium";
                MediumListing? l = c.Listing;
                return $"{c.Medium.Id} {c.Exists} {c.Length} " + (l is null ? "unlisted" : $"{l.Formatted} {l.FreeUnits} [{l.Problem}] " + string.Join("|", l.Saves.Select(s => $"{s.Id}/{s.Title}/{s.Units}/{s.Game}/{s.Damaged}")));
            }
            catch (Exception e) { return "threw " + e.GetType().Name; }
        }

        private static string Tree(string tree) => string.Join("\n", Directory.EnumerateFileSystemEntries(tree, "*", SearchOption.AllDirectories).Order(StringComparer.Ordinal).Select(p =>
        {
            string name = Path.GetRelativePath(tree, p);
            if (Directory.Exists(p)) return name + "/";
            try { return name + " " + Convert.ToHexString(SHA256.HashData(File.ReadAllBytes(p)))[..16]; }
            catch (UnauthorizedAccessException) { return name + " unreadable"; }
        }));

        // A card held while an operation runs, the same way in both trees: by a game of this program, shared or not, or by another program's lock.
        private static IDisposable? Hold(int kind, string card) => kind switch
        {
            0 => MediumLock.TryTake(card, "Game X"),
            1 => MediumLock.TryTake(card, "Game Y", shared: true),
            2 => Directory.Exists(Path.GetDirectoryName(card)) ? new FileStream(card + MediumLock.Suffix, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None, 1, FileOptions.DeleteOnClose) : null,
            _ => null,
        };

        [Fact]
        public void Generated_scripts_on_cards_of_every_medium_free_and_held_do_the_same()
        {
            int done = 0, refused = 0, held = 0, ops = 0;
            for (int seed = 1; seed <= 60; seed++)
            {
                string managed = Path.Combine(_root, $"m{seed}"), native = Path.Combine(_root, $"n{seed}");
                Plant(new Random(seed), managed);
                Plant(new Random(seed), native);
                var r = new Random(seed * 7919);
                for (int step = 0; step < 30; step++)
                {
                    string ext = Extensions[r.Next(Extensions.Length)];
                    string Name() => r.Next(14) switch { 0 => "notes.txt", 1 => "Locked.mcr", 2 => "Folder.mpk", 3 => "New" + ext, 4 => Path.Combine("Deep", "D" + ext), 5 => "A" + Extensions[r.Next(4)], _ => $"{"ABC"[r.Next(3)]}{ext}" };
                    string a = Name(), b = Name(), c = Name();
                    var listing = SaveMediaManager.Managed.Open(Path.Combine(managed, a))?.Listing;
                    string save = r.Next(6) switch { 0 => "1", 1 => "2", 2 => "99", 3 => "NOSUCH", _ => listing is { Saves.Count: > 0 } ? listing.Saves[r.Next(listing.Saves.Count)].Id : "1" };
                    int op = r.Next(8), hold = r.Next(6);
                    string heldName = r.Next(2) == 0 ? a : b;
                    string M(string n) => Path.Combine(managed, n);
                    string N(string n) => Path.Combine(native, n);
                    string shownManaged, shownNative;
                    using (Hold(hold, M(heldName)))
                        shownManaged = Run(op, M, a, b, c, save, managedHalf: true);
                    using (Hold(hold, N(heldName)))
                        shownNative = Run(op, N, a, b, c, save, managedHalf: false);
                    string what = $"seed {seed} step {step}: op {op} {a} {b} {c} '{save}' hold {hold} on {heldName}\n";
                    Same(what + shownManaged, what + shownNative);
                    Same(what + Tree(managed), what + Tree(native));
                    ops++;
                    done += shownManaged.StartsWith("True", StringComparison.Ordinal) ? 1 : 0;
                    refused += shownManaged.StartsWith("False", StringComparison.Ordinal) ? 1 : 0;
                    held += shownManaged.Contains("in use by", StringComparison.Ordinal) ? 1 : 0;
                }
            }
            _output.WriteLine($"{ops} operations: {done} done, {refused} refused, {held} held");
            Assert.True(done > 150 && refused > 300 && held > 40, $"{ops} operations: {done} done, {refused} refused, {held} held");
        }

        private static string Run(int op, Func<string, string> at, string a, string b, string c, string save, bool managedHalf) => op switch
        {
            0 => Shown(() => managedHalf ? SaveMediaManager.Managed.Open(at(a)) : SaveMediaNative.Open(at(a))),
            1 => Shown(() => managedHalf ? SaveMediaManager.Managed.CopySave(at(a), save, at(b)) : SaveMediaNative.CopySave(at(a), save, at(b))),
            2 => Shown(() => managedHalf ? SaveMediaManager.Managed.MoveSave(at(a), save, at(b)) : SaveMediaNative.MoveSave(at(a), save, at(b))),
            3 => Shown(() => managedHalf ? SaveMediaManager.Managed.DeleteSave(at(a), save) : SaveMediaNative.DeleteSave(at(a), save)),
            4 => Shown(() => managedHalf ? SaveMediaManager.Managed.Import(at(a), [at(b), at(c), at(a)]) : SaveMediaNative.Import(at(a), [at(b), at(c), at(a)])),
            5 => Shown(() => managedHalf ? SaveMediaManager.Managed.New(at(a)) : SaveMediaNative.New(at(a))),
            6 => Shown(() => managedHalf ? SaveMediaManager.Managed.Duplicate(at(a), at(b)) : SaveMediaNative.Duplicate(at(a), at(b))),
            _ => Shown(() => managedHalf ? SaveMediaManager.Managed.Rename(at(a), at(b)) : SaveMediaNative.Rename(at(a), at(b))),
        };

        private static void Same(string managed, string native)
        {
            if (managed != native) Assert.Fail($"C#:\n{managed}\nRust:\n{native}");
        }

        // An import whose saves exactly fill the card is taken, and one a block over is refused whole, by both halves.
        [Fact]
        public void An_import_that_exactly_fills_the_card_and_one_a_block_over_do_the_same()
        {
            foreach (int extra in new[] { 0, 1 })
            {
                var shown = new List<string>();
                foreach (string half in new[] { "m", "n" })
                {
                    string dir = Directory.CreateDirectory(Path.Combine(_root, $"fill{extra}", half)).FullName;
                    byte[] Card(params (string Name, int[] Blocks)[] saves) => saves.Aggregate(PsxCardFormat.Instance.Managed.Format()!, (card, s) => SaveMediaManagerTests.WithSave(card, s.Name, s.Name, s.Blocks, 0x5A));
                    File.WriteAllBytes(Path.Combine(dir, "To.mcr"), Card(extra == 0 ? [] : [("BASLUS-00001HELD", new[] { 0 })]));
                    File.WriteAllBytes(Path.Combine(dir, "One.mcr"), Card(("BASLUS-00002BIG", Enumerable.Range(0, 8).ToArray())));
                    File.WriteAllBytes(Path.Combine(dir, "Two.mcr"), Card(("BASLUS-00003REST", Enumerable.Range(0, 7).ToArray())));
                    string[] from = [Path.Combine(dir, "One.mcr"), Path.Combine(dir, "Two.mcr")];
                    shown.Add(Shown(() => half == "m" ? SaveMediaManager.Managed.Import(Path.Combine(dir, "To.mcr"), from) : SaveMediaNative.Import(Path.Combine(dir, "To.mcr"), from)) + "\n" + Tree(dir));
                }
                Same(shown[0], shown[1]);
                Assert.StartsWith(extra == 0 ? "True" : "False", shown[0]);
            }
        }

        // The lock's two tables: each half's own sequence answers alike, and a file one half holds is another program's to the other.
        [Fact]
        public void The_library_lock_and_the_csharp_lock_answer_alike_and_refuse_each_other()
        {
            string m = Path.Combine(_root, "lock", "m.mcr"), n = Path.Combine(_root, "lock", "n.mcr");
            string Sequence(Func<string, string, bool, MediumLock?> take, Func<string, string?> holder, string card)
            {
                var log = new List<string>();
                using (MediumLock? one = take(card, "Game A", true))
                {
                    log.Add($"{one is not null} {holder(card)}");
                    using MediumLock? two = take(card, "Game B", true);
                    log.Add($"{two is not null} {holder(card)} {take(card, "the card manager", false) is not null}");
                }
                log.Add($"{holder(card) ?? "none"} {File.Exists(card + MediumLock.Suffix)}");
                using (MediumLock? solo = take(card, "Game C", false)) log.Add($"{solo is not null} {take(card, "Game D", true) is not null} {holder(card)}");
                return string.Join("\n", log);
            }
            Assert.True(MediumLockNative.Takes(n, "x"));
            Same(Sequence(MediumLock.Managed.TryTake, MediumLock.Managed.HolderOf, m), Sequence(MediumLock.TryTake, MediumLock.HolderOf, n));
            using (MediumLock.Managed.TryTake(m, "Game A"))
                Assert.Equal(("another program", true), (MediumLock.HolderOf(m), MediumLock.TryTake(m, "x") is null));
            using (MediumLock.TryTake(n, "Game A"))
                Assert.Equal(("another program", true), (MediumLock.Managed.HolderOf(n), MediumLock.Managed.TryTake(n, "x") is null));
        }

        // What a read or a write that failed reports is the same from both halves.
        [Fact]
        public void A_card_that_will_not_read_is_reported_alike()
        {
            foreach (string half in new[] { "m", "n" })
            {
                string dir = Directory.CreateDirectory(Path.Combine(_root, "said", half)).FullName;
                File.WriteAllBytes(Path.Combine(dir, "Locked.mcr"), PsxCardFormat.Instance.Managed.Format()!);
                File.SetUnixFileMode(Path.Combine(dir, "Locked.mcr"), UnixFileMode.None);
            }
            string Said(string half, Func<string, MediaOutcome> op)
            {
                _said.Clear();
                string dir = Path.Combine(_root, "said", half);
                string shown = Shown(() => op(dir));
                return shown + "\n" + string.Join("\n", _said).Replace(dir, "<dir>", StringComparison.Ordinal);
            }
            Same(Said("m", d => SaveMediaManager.Managed.CopySave(Path.Combine(d, "Locked.mcr"), "1", Path.Combine(d, "B.mcr"))),
                Said("n", d => SaveMediaNative.CopySave(Path.Combine(d, "Locked.mcr"), "1", Path.Combine(d, "B.mcr"))));
            Same(Said("m", d => SaveMediaManager.Managed.Duplicate(Path.Combine(d, "Locked.mcr"), Path.Combine(d, "C.mcr"))),
                Said("n", d => SaveMediaNative.Duplicate(Path.Combine(d, "Locked.mcr"), Path.Combine(d, "C.mcr"))));
        }
    }
}
