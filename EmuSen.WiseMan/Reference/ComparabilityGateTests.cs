using System.Collections.Generic;
using System.IO;
using EmuSen.Pharaoh.Reference;

namespace EmuSen.WiseMan.Reference
{
    // The gates that decide whether a difference means anything - see §3.48/§3.48a.
    public class ComparabilityGateTests
    {
        // A dump set is a directory of files, so the fixtures are written as files.
        private static string WriteSet(string backend, string board, string region, string trust,
            string screenFormat, IReadOnlyList<(long Frame, uint Ram, uint Work)> rows)
        {
            string dir = Path.Combine(Path.GetTempPath(), "emusen-gate-" + Path.GetRandomFileName());
            Directory.CreateDirectory(dir);

            var text = new System.Text.StringBuilder();
            text.Append("# emusen-probe-signature 1\n");
            text.Append($"# backend={backend}\n# system=nes\n# rom=/fake.nes\n");
            text.Append($"# board={board}\n# region={region}\n# headerTrust={trust}\n");
            text.Append("# prg=65536\n# chr=65536\n# saveLoaded=0\n");
            text.Append($"# screenFormat={screenFormat}\n");
            text.Append("frame,ram,work,screen\n");
            foreach ((long frame, uint ram, uint work) in rows)
            {
                text.Append($"{frame},{ram:x8},{work:x8},00000000\n");
            }

            File.WriteAllText(Path.Combine(dir, $"{backend}_sig.csv"), text.ToString());
            return dir;
        }

        private static List<(long, uint, uint)> Rows(int count, uint ramSeed, long divergeAt = -1)
        {
            var rows = new List<(long, uint, uint)>();
            for (int i = 0; i < count; i++)
            {
                uint ram = divergeAt >= 0 && i >= divergeAt ? 0xDEAD0000u + (uint)i : ramSeed + (uint)i;
                rows.Add((i, ram, 0x1000u));
            }
            return rows;
        }

        [Fact]
        public void A_board_mismatch_is_reported_before_any_pixel_is_compared()
        {
            string ours = WriteSet("emusen", "65", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "PaletteIndex16", Rows(40, 100));

            Comparator.Report report = Comparator.Compare(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!, 20, 4, new long[0]);

            Assert.Equal(Verdict.NotComparable, report.Verdict);
            Assert.Contains("board differs", report.Summary);
        }

        // The case the gate found on its first real outing - see Moon_Memory.md §4.14.
        [Fact]
        public void A_region_mismatch_is_not_comparable_even_when_the_boards_agree()
        {
            string ours = WriteSet("emusen", "69", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "69", "pal", "", "PaletteIndex16", Rows(40, 100));

            Comparator.Report report = Comparator.Compare(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!, 20, 4, new long[0]);

            Assert.Equal(Verdict.NotComparable, report.Verdict);
            Assert.Contains("region differs", report.Summary);
        }

        // A constant column agrees at every offset, so it must not decide alignment.
        [Fact]
        public void A_column_that_never_changes_does_not_decide_the_alignment()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(60, 500));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "Rgba8888", Rows(60, 498));

            Comparator.DivergenceResult result = Comparator.FirstDivergence(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!);

            // Our frame f carries ram 500+f and theirs 498+f, so ours[f] == theirs[f+2].
            Assert.Equal(2, result.Offset);
        }

        [Fact]
        public void Columns_are_rated_by_how_often_they_actually_agree()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(60, 500));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "Rgba8888", Rows(60, 500));

            Comparator.DivergenceResult result = Comparator.FirstDivergence(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!);

            Comparator.ColumnProfile ram = result.Profiles.Find(p => p.Name == "ram")!;
            Assert.Equal(Comparator.Comparability.Stable, ram.Rating);
        }

        // No stable column means no verdict, rather than a number nobody should trust.
        [Fact]
        public void An_unreliable_column_cannot_carry_a_divergence_claim()
        {
            // Both columns unreliable, so nothing is left that could carry a verdict.
            var noisy = new List<(long, uint, uint)>();
            for (int i = 0; i < 60; i++) noisy.Add((i, (uint)(i * 7919), (uint)(i * 104729)));

            // Differing screen formats too, so not even the screen column survives.
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(60, 500));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "PaletteIndex16", noisy);

            Comparator.DivergenceResult result = Comparator.FirstDivergence(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!);

            Comparator.ColumnProfile ram = result.Profiles.Find(p => p.Name == "ram")!;
            Assert.Equal(Comparator.Comparability.Never, ram.Rating);
            Assert.True(result.Inconclusive);
            Assert.Null(result.Frame);
        }

        [Fact]
        public void A_stable_column_locates_the_frame_the_streams_parted()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(200, 500));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "Rgba8888", Rows(200, 500, divergeAt: 150));

            Comparator.DivergenceResult result = Comparator.FirstDivergence(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!);

            Assert.False(result.Inconclusive);
            Assert.Equal(150, result.Frame);
        }

        // The inversion the first cut got wrong: a blank screen is not a pass.
        [Fact]
        public void Screen_format_differences_drop_the_screen_column_rather_than_failing()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "PaletteIndex16", Rows(40, 100));

            Comparator.DivergenceResult result = Comparator.FirstDivergence(
                DumpSet.Load(ours)!, DumpSet.Load(theirs)!);

            Assert.DoesNotContain("screen", result.Columns);
        }
    }
}
