using System.Security.Cryptography;
using EmuSen.WiseMan.Bindings.SaveMedia;
using EmuSen.WiseMan.Bindings.Services;
using EmuSen.WiseMan.Bindings.Systems.Genesis;
using EmuSen.WiseMan.Bindings.Systems.N64;
using EmuSen.WiseMan.Bindings.Systems.Psx;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Native;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Crystal
{
    // emusen-savemedia held to the C# card manager's answers, recorded as goldens before it was deleted, on generated trees of every medium's cards, free and held, outcomes and files compared - see EmuSen_Backend.md §15.2, §40.1.
    [Collection(TestCollections.ProcessGlobals)]
    public class SaveMediaManagerParityTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenCardManagerParity_" + Guid.NewGuid().ToString("N"));
        private readonly Action<string>? _sink = ConfigDiagnostics.Sink;
        private readonly List<string> _said = new();
        private readonly ITestOutputHelper _output;

        public SaveMediaManagerParityTests(ITestOutputHelper output)
        {
            _output = output;
            Assert.True(ServiceLibrary.Ready, ServiceLibrary.Report);
            Assert.True(PlatformLibrary.Available, PlatformLibrary.Report);
            Directory.CreateDirectory(_root);
            ConfigDiagnostics.Sink = _said.Add;
        }

        public void Dispose()
        {
            ConfigDiagnostics.Sink = _sink;
            foreach (string path in Directory.EnumerateFiles(_root, "*", SearchOption.AllDirectories)) try { File.SetUnixFileMode(path, UnixFileMode.UserRead | UnixFileMode.UserWrite); } catch (IOException) { }
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
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

        private ParityGolden? _golden;

        private void Same(string answer) => _golden!.Add(_golden.Count.ToString(), answer.Replace(_root, "<root>", StringComparison.Ordinal));

        // An import whose saves exactly fill the card is taken, and one a block over is refused whole.
        [Fact]
        public void An_import_that_exactly_fills_the_card_and_one_a_block_over_do_the_same()
        {
            foreach (int extra in new[] { 0, 1 })
            {
                string dir = Directory.CreateDirectory(Path.Combine(_root, $"fill{extra}", "m")).FullName;
                byte[] Card(params (string Name, int[] Blocks)[] saves) => saves.Aggregate(PsxCardFormat.Instance.Format()!, (card, s) => SaveMediaManagerTests.WithSave(card, s.Name, s.Name, s.Blocks, 0x5A));
                File.WriteAllBytes(Path.Combine(dir, "To.mcr"), Card(extra == 0 ? [] : [("BASLUS-00001HELD", new[] { 0 })]));
                File.WriteAllBytes(Path.Combine(dir, "One.mcr"), Card(("BASLUS-00002BIG", Enumerable.Range(0, 8).ToArray())));
                File.WriteAllBytes(Path.Combine(dir, "Two.mcr"), Card(("BASLUS-00003REST", Enumerable.Range(0, 7).ToArray())));
                string[] from = [Path.Combine(dir, "One.mcr"), Path.Combine(dir, "Two.mcr")];
                string shown = Shown(() => SaveMediaManager.Import(Path.Combine(dir, "To.mcr"), from)) + "\n" + Tree(dir);
                _golden = new ParityGolden($"CardManagerFill{extra}");
                Same(shown.Replace(dir, "<dir>", StringComparison.Ordinal));
                _golden.Check(1);
                Assert.StartsWith(extra == 0 ? "True" : "False", shown);
            }
        }

        // The library's lock answers its sequence as the C#'s table did, and a lock file another program holds, or one the library holds, is the other's to each.
        [Fact]
        public void The_library_lock_answers_as_the_csharp_lock_did_and_refuses_another_program()
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
            _golden = new ParityGolden("MediumLockSequence");
            Same(Sequence(MediumLock.TryTake, MediumLock.HolderOf, n));
            _golden.Check(1);
            Directory.CreateDirectory(Path.GetDirectoryName(m)!);
            using (new FileStream(m + MediumLock.Suffix, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None))
                Assert.Equal(("another program", true), (MediumLock.HolderOf(m), MediumLock.TryTake(m, "x") is null));
            using (MediumLock.TryTake(n, "Game A"))
                Assert.Throws<IOException>(() => new FileStream(n + MediumLock.Suffix, FileMode.OpenOrCreate, FileAccess.ReadWrite, FileShare.None).Dispose());
        }

    }
}
