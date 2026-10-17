using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Media;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Cores
{
    // Since core ABI 1.3: a disc Galaxia maps and the host lends runs as one copied, the mapping is held while a machine may read it and let go after - see EmuSen_CoreAPI.md §6.24, EmuSen_Galaxia.md §3.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class DiscStreamingTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenDiscStreaming_" + Guid.NewGuid().ToString("N"));

        public DiscStreamingTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
        }

        public void Dispose()
        {
            CoreDiscovery.UseDevelopment(null);
            DataStore.OverrideDirectory = null;
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        private static CoreLibrary Diamond => CoreLibrary.Open(DiamondTests.LibraryPath);

        private static string Defaults(CoreLibrary library) => string.Join("\n", library.Settings.Select(s => $"{s.Key}={s.Default}"));

        // Whether this process maps the file now, by the system's own list; null where there is no such list.
        private static bool? Mapped(string path)
        {
            if (!OperatingSystem.IsLinux()) return null;
            string full = Path.GetFullPath(path);
            return File.ReadLines("/proc/self/maps").Any(l => l.EndsWith(" " + full, StringComparison.Ordinal));
        }

        // Each frame's picture, sound and state of the two machines, required equal frame by frame.
        private static void SameRun(CoreMachine lent, CoreMachine copied, int frames, string what)
        {
            byte[] a = Array.Empty<byte>(), b = Array.Empty<byte>();
            for (int f = 0; f < frames; f++)
            {
                lent.Advance();
                copied.Advance();
                byte[] pa = new byte[Math.Max(0, lent.CopyFrame(Array.Empty<byte>()))], pb = new byte[Math.Max(0, copied.CopyFrame(Array.Empty<byte>()))];
                lent.CopyFrame(pa);
                copied.CopyFrame(pb);
                Assert.True(pa.AsSpan().SequenceEqual(pb), $"{what}: frame {f}'s picture");
                Assert.True(lent.DrainAudio(1 << 16).Samples.AsSpan().SequenceEqual(copied.DrainAudio(1 << 16).Samples), $"{what}: frame {f}'s sound");
                if (a.Length != lent.StateSize()) (a, b) = (new byte[lent.StateSize()], new byte[copied.StateSize()]);
                lent.Save(a);
                copied.Save(b);
                Assert.True(a.AsSpan().SequenceEqual(b), $"{what}: frame {f}'s state");
            }
        }

        // The disc at path mapped through Galaxia and lent, against its bytes read and copied by the core.
        private static void LentRunsAsCopied(CoreLibrary library, string path, int frames)
        {
            var disc = DiscFiles.Open(path);
            var media = disc.Companions.Select((c, n) => (CoreInterface.FileMedia + (uint)n, Held(c))).ToList();
            using var lent = new CoreMachine(library, Held(disc.Image), media, Defaults(library), Array.Empty<(uint, byte[])>(), loan: disc);
            var files = ImageSets.FilesFor(path).Select((f, n) => (CoreInterface.FileMedia + (uint)n, File.ReadAllBytes(f))).ToList();
            using var copied = new CoreMachine(library, File.ReadAllBytes(path), Defaults(library), files);
            SameRun(lent, copied, frames, Path.GetFileName(path));
        }

        private static unsafe CoreBytes Held(MappedFile file) => new(file.Pointer, file.Length);

        [Fact]
        public void A_disc_galaxia_maps_and_the_host_lends_runs_frame_for_frame_as_one_copied()
        {
            var disc = SyntheticPsxDisc.Build(audioSeconds: 2);
            string sheet = disc.WriteSet(_root, "Lent Set");
            string track = Path.Combine(_root, "Lent Track.bin");
            File.WriteAllBytes(track, disc.DataTrack);
            string iso = Path.Combine(_root, "Lent Image.iso");
            File.WriteAllBytes(iso, disc.Iso);
            foreach (string path in new[] { sheet, track, iso }) LentRunsAsCopied(Diamond, path, 600);
            string tst = Path.Combine(_root, "game.tst");
            File.WriteAllBytes(tst, CoreAdapterTests.Image(Enumerable.Range(0, 200).Select(i => (byte)i).ToArray()));
            LentRunsAsCopied(CoreAdapterTests.Full, tst, 300);
        }

        // The engine maps a borrowing core's disc and holds it for the machine's life; a core that copies leaves nothing mapped once it is made.
        [Fact]
        public void The_engine_holds_a_lent_disc_for_the_machines_life_and_a_copied_one_not_at_all()
        {
            if (!OperatingSystem.IsLinux()) return;
            var disc = SyntheticPsxDisc.Build();
            string sheet = disc.WriteSet(_root, "Held Set");
            string track = Path.Combine(_root, "Held Set (Track 1).bin");
            var engine = new CoreEngine(Diamond) { Options = TestCores.NoBattery };
            engine.LoadRom(sheet);
            engine.RunFrame();
            Assert.True(Mapped(track), "the track is read where it is while the game runs");
            engine.Dispose();
            Assert.False(Mapped(track), "the mapping is let go with the machine");

            string tst = Path.Combine(_root, "plain.tst");
            File.WriteAllBytes(tst, CoreAdapterTests.Image(1, 2, 3));
            using var plain = new CoreEngine(CoreAdapterTests.Plain) { Options = TestCores.NoBattery };
            plain.LoadRom(tst);
            Assert.False(Mapped(tst), "a core without BORROW has its copy, and the file is let go at once");
            using var full = new CoreEngine(CoreAdapterTests.Full) { Options = TestCores.NoBattery };
            full.LoadRom(tst);
            Assert.True(Mapped(tst));
        }

        // Change Disc takes the next disc of a set mapped and lets the first go only once the core has taken the second; a refused disc leaves the first in place.
        [Fact]
        public void A_disc_change_lets_the_earlier_disc_go_once_the_core_has_taken_the_next()
        {
            if (!OperatingSystem.IsLinux()) return;
            var one = SyntheticPsxDisc.Build();
            var two = SyntheticPsxDisc.Build(audioSeconds: 3);
            one.WriteSet(_root, "Twin (Disc 1)");
            two.WriteSet(_root, "Twin (Disc 2)");
            string list = Path.Combine(_root, "Twin.m3u");
            File.WriteAllText(list, "Twin (Disc 1).cue\nTwin (Disc 2).cue\n");
            string first = Path.Combine(_root, "Twin (Disc 1) (Track 1).bin"), second = Path.Combine(_root, "Twin (Disc 2) (Track 1).bin");
            using var engine = new CoreEngine(Diamond) { Options = TestCores.NoBattery };
            engine.LoadRom(list);
            engine.RunFrame();
            Assert.Equal((true, false), (Mapped(first), Mapped(second)));
            engine.ChangeDisc(1);
            for (int f = 0; f < 90; f++) engine.RunFrame();
            Assert.Equal((false, true, 1), (Mapped(first), Mapped(second), engine.DiscIndex));
            File.WriteAllBytes(Path.Combine(_root, "Twin (Disc 1) (Track 1).bin"), new byte[4096]);
            engine.SetLid(true);
            Assert.ThrowsAny<Exception>(() => engine.InsertDisc(0));
            Assert.Equal((false, true, 1), (Mapped(first), Mapped(second), engine.DiscIndex));
        }

        // Since core ABI 1.4: the place a drive names is handed to the disc's reader, which advises the system around it and no further; a drive lent nothing, a file the call did not pass and a byte outside its file are dropped - see EmuSen_CoreAPI.md §6.25, EmuSen_Galaxia.md §3.9.
        [Fact]
        public void A_place_a_drive_names_is_read_ahead_in_the_file_galaxia_holds_for_it()
        {
            const int MiB = 1 << 20;
            string tst = Path.Combine(_root, "long.tst");
            File.WriteAllBytes(tst, CoreAdapterTests.Image(Enumerable.Range(0, 60_000).Select(i => (byte)i).ToArray()));
            string big = Path.Combine(_root, "big.bin");
            File.WriteAllBytes(big, new byte[8 * MiB + 5]);
            static void Settled(DiscFiles files, long advised)
            {
                var watch = System.Diagnostics.Stopwatch.StartNew();
                while (files.Advised != advised && watch.Elapsed < TimeSpan.FromSeconds(10)) System.Threading.Thread.Sleep(1);
                Assert.Equal(advised, files.Advised);
            }
            using (DiscFiles files = DiscFiles.Open(big))
            {
                if (!files.Image.Advise(0, 4096)) return;
                files.ReadAhead(files.Image, 4 * MiB);
                Settled(files, DiscFiles.ReadAheadBefore + DiscFiles.ReadAheadPast);
                files.ReadAhead(files.Image, 0);
                Settled(files, 2 * (long)DiscFiles.ReadAheadPast + DiscFiles.ReadAheadBefore);
                long before = files.Advised;
                files.ReadAhead(files.Image, 8 * MiB);
                Settled(files, before + DiscFiles.ReadAheadBefore + 5);
                files.Dispose();
                files.ReadAhead(files.Image, 4 * MiB);
            }
            var disc = DiscFiles.Open(tst);
            using var lent = new CoreMachine(CoreAdapterTests.Full, Held(disc.Image), Array.Empty<(uint, CoreBytes)>(), Defaults(CoreAdapterTests.Full), Array.Empty<(uint, byte[])>(), loan: disc);
            static CoreInterface.Event Place(uint drive, uint file, long at) => new() { Kind = CoreInterface.EventMediaSeek, A = (long)drive << 32 | file, B = at };
            foreach (var dropped in new[] { Place(1, CoreInterface.FileImage, 0), Place(0, CoreInterface.FileMedia, 0), Place(0, 0, 0), Place(0, CoreInterface.FileImage, disc.Image.Length), Place(0, CoreInterface.FileImage, -1) })
                CoreEngine.ReadAhead(lent, dropped);
            System.Threading.Thread.Sleep(50);
            Assert.Equal(0, disc.Advised);
            CoreEngine.ReadAhead(lent, Place(0, CoreInterface.FileImage, 100));
            Settled(disc, disc.Image.Length);
            using var copied = new CoreMachine(CoreAdapterTests.Full, File.ReadAllBytes(tst), Defaults(CoreAdapterTests.Full), Array.Empty<(uint, byte[])>());
            CoreEngine.ReadAhead(copied, Place(0, CoreInterface.FileImage, 100));
        }

        // A machine whose host reads ahead at every place its drive names runs frame for frame as one whose host skips them, through the engine and beside it; Diamond names places as its kernel reads the disc.
        [Fact]
        public void A_run_whose_host_reads_ahead_of_the_drive_is_the_run_whose_host_does_not()
        {
            var built = SyntheticPsxDisc.Build(audioSeconds: 2);
            string sheet = built.WriteSet(_root, "Named Set");
            DiscFiles acted = DiscFiles.Open(sheet), skipped = DiscFiles.Open(sheet);
            List<(uint, CoreBytes)> Media(DiscFiles d) => d.Companions.Select((c, n) => (CoreInterface.FileMedia + (uint)n, Held(c))).ToList();
            using var a = new CoreMachine(Diamond, Held(acted.Image), Media(acted), Defaults(Diamond), Array.Empty<(uint, byte[])>(), loan: acted);
            using var b = new CoreMachine(Diamond, Held(skipped.Image), Media(skipped), Defaults(Diamond), Array.Empty<(uint, byte[])>(), loan: skipped);
            int named = 0;
            byte[] sa = Array.Empty<byte>(), sb = Array.Empty<byte>();
            for (int f = 0; f < 600; f++)
            {
                a.Advance();
                b.Advance();
                foreach (var e in a.DrainEvents().Where(e => e.Kind == CoreInterface.EventMediaSeek))
                {
                    named++;
                    int file = (int)((uint)e.A - CoreInterface.FileMedia);
                    Assert.True(e.A >> 32 == 0 && file >= 0 && file < acted.Companions.Count, $"drive {e.A >> 32}, file {(uint)e.A}");
                    Assert.InRange(e.B, 0, acted.Companions[file].Length - 1);
                    CoreEngine.ReadAhead(a, e);
                }
                b.DrainEvents();
                byte[] pa = new byte[Math.Max(0, a.CopyFrame(Array.Empty<byte>()))], pb = new byte[Math.Max(0, b.CopyFrame(Array.Empty<byte>()))];
                a.CopyFrame(pa);
                b.CopyFrame(pb);
                Assert.True(pa.AsSpan().SequenceEqual(pb), $"frame {f}'s picture");
                Assert.True(a.DrainAudio(1 << 16).Samples.AsSpan().SequenceEqual(b.DrainAudio(1 << 16).Samples), $"frame {f}'s sound");
                if (sa.Length != a.StateSize()) (sa, sb) = (new byte[a.StateSize()], new byte[b.StateSize()]);
                a.Save(sa);
                b.Save(sb);
                Assert.True(sa.AsSpan().SequenceEqual(sb), $"frame {f}'s state");
            }
            Assert.True(named > 0, "the kernel's reads of the disc name their places");
            Assert.Equal(0, skipped.Advised);
            static bool Acted(DiscFiles files)
            {
                var watch = System.Diagnostics.Stopwatch.StartNew();
                while (files.Advised == 0 && watch.Elapsed < TimeSpan.FromSeconds(10)) System.Threading.Thread.Sleep(1);
                return files.Advised > 0;
            }
            bool advises = acted.Companions[0].Advise(0, 1);
            Assert.Equal(advises, Acted(acted));

            using var engine = new CoreEngine(Diamond) { Options = TestCores.NoBattery };
            engine.LoadRom(sheet);
            for (int f = 0; f < 600; f++) engine.RunFrame();
            var held = Assert.IsType<DiscFiles>(engine.Machine.LoanOf(0));
            Assert.True(!advises || Acted(held), "the engine routes a drive's places to its disc's reader");
        }

        // An unpacked folder a game has mapped stays through pruning, and goes at the next prune once the game lets it go.
        [Fact]
        public void An_unpacked_folder_a_game_holds_is_never_pruned()
        {
            var disc = SyntheticPsxDisc.Build();
            string zip = Path.Combine(_root, "Held Zip.zip");
            using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create))
            {
                foreach (var (name, bytes) in new[] { ("Held Zip.cue", System.Text.Encoding.ASCII.GetBytes(disc.Cue.Replace("{name}", "Held Zip"))), ("Held Zip (Track 1).bin", disc.DataTrack), ("Held Zip (Track 2).bin", disc.AudioTrack) })
                    using (var s = archive.CreateEntry(name).Open()) s.Write(bytes);
            }
            string folder = ArchiveImages.FolderFor(zip);
            var set = DiscSet.Of(zip, CoreCatalog.IsImageExtension);
            using (DiscFiles held = set.Open(0))
            {
                ArchiveImages.Prune(keptBytes: 0);
                Assert.True(Directory.Exists(folder) && File.Exists(Path.Combine(folder, ".unpacked")), "a held folder is kept");
                Assert.Equal(disc.DataTrack, held.Companions[0].Span(0, disc.DataTrack.Length).ToArray());
            }
            ArchiveImages.Prune(keptBytes: 0);
            Assert.False(Directory.Exists(folder), "let go, it is pruned as any other");
            using DiscFiles again = set.Open(0);
            Assert.True(File.Exists(Path.Combine(folder, ".unpacked")), "the set unpacks it anew");
        }
    }
}
