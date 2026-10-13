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
        private readonly bool _batteryWas = CoreOptions.BatteryRamDisabled;

        public DiscStreamingTests()
        {
            Directory.CreateDirectory(_root);
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            CoreDiscovery.UseDevelopment(false);
            CoreOptions.BatteryRamDisabled = true;
        }

        public void Dispose()
        {
            CoreOptions.BatteryRamDisabled = _batteryWas;
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
            disc.Prefetch();
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
            var engine = new CoreEngine(Diamond);
            engine.LoadRom(sheet);
            engine.RunFrame();
            Assert.True(Mapped(track), "the track is read where it is while the game runs");
            engine.Dispose();
            Assert.False(Mapped(track), "the mapping is let go with the machine");

            string tst = Path.Combine(_root, "plain.tst");
            File.WriteAllBytes(tst, CoreAdapterTests.Image(1, 2, 3));
            using var plain = new CoreEngine(CoreAdapterTests.Plain);
            plain.LoadRom(tst);
            Assert.False(Mapped(tst), "a core without BORROW has its copy, and the file is let go at once");
            using var full = new CoreEngine(CoreAdapterTests.Full);
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
            using var engine = new CoreEngine(Diamond);
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

        // The prefetch reads every file of the disc to its end beside the game, rests between reads, and stops when the disc is let go; the mapping's bytes are the file's throughout.
        [Fact]
        public void The_prefetch_reads_the_whole_disc_and_changes_no_byte_of_it()
        {
            var disc = SyntheticPsxDisc.Build(audioSeconds: 20);
            string sheet = disc.WriteSet(_root, "Fetched Set");
            using (DiscFiles files = DiscFiles.Open(sheet))
            {
                Assert.False(files.Prefetched);
                var watch = System.Diagnostics.Stopwatch.StartNew();
                files.Prefetch();
                files.Prefetch();
                while (!files.Prefetched && watch.Elapsed < TimeSpan.FromSeconds(30)) System.Threading.Thread.Sleep(5);
                Assert.True(files.Prefetched, "every file read to its end");
                Assert.Equal(disc.DataTrack, files.Companions[0].Span(0, disc.DataTrack.Length).ToArray());
                Assert.Equal(disc.AudioTrack, files.Companions[1].Span(0, disc.AudioTrack.Length).ToArray());
            }
            using (DiscFiles stopped = DiscFiles.Open(sheet))
            {
                stopped.Prefetch();
                stopped.Dispose();
                System.Threading.Thread.Sleep(100);
                Assert.False(stopped.Prefetched, "let go at once, it stops");
            }
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
