using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Globalization;
using System.IO;
using System.Text;
using EmuSen.Galaxia;
using EmuSen.Pharaoh.Reference;

namespace EmuSen.WiseMan.Reference
{
    // The gate for deleting the C# comparator: both implementations, same inputs,
    // byte-identical output - see EmuSen_Stack.md §3.
    public class PythonPortDifferentialTests : IDisposable
    {
        private readonly string _dir;

        public PythonPortDifferentialTests()
        {
            _dir = Path.Combine(Path.GetTempPath(), "emusen-diff-" + Path.GetRandomFileName());
            Directory.CreateDirectory(_dir);
        }

        public void Dispose()
        {
            try { Directory.Delete(_dir, recursive: true); } catch (IOException) { }
        }

        private static string AnalysisDirectory =>
            Path.Combine(ConfigRoot.Directory, "EmuSen.WiseMan", "Reference", "analysis");

        // A dump set is a directory of files, so the fixtures are written as files -
        // the same shape ComparabilityGateTests uses, so that a verdict differing
        // between the two implementations could not be blamed on the fixture.
        private string WriteSet(string backend, string board, string region, string trust,
            string screenFormat, IReadOnlyList<(long Frame, uint Ram, uint Work)> rows)
        {
            string dir = Path.Combine(_dir, backend + "-" + Path.GetRandomFileName());
            Directory.CreateDirectory(dir);

            var text = new StringBuilder();
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

        // A real framebuffer, so the vacuity gate and the pixel arithmetic are
        // exercised rather than skipped for want of a screen.
        private static void WriteScreen(string dir, string backend, long frame,
            Func<int, int, (byte R, byte G, byte B)> colour)
        {
            var raw = new byte[256 * 240 * 4];
            for (int y = 0; y < 240; y++)
            {
                for (int x = 0; x < 256; x++)
                {
                    (byte r, byte g, byte b) = colour(x, y);
                    int at = ((y * 256) + x) * 4;
                    raw[at] = r;
                    raw[at + 1] = g;
                    raw[at + 2] = b;
                    raw[at + 3] = 0xFF;
                }
            }
            File.WriteAllBytes(Path.Combine(dir, $"{backend}_screen_f{frame:D5}.bin"), raw);
        }

        private static (byte, byte, byte) Busy(int x, int y) =>
            ((byte)(x * 7 % 251), (byte)(y * 13 % 241), (byte)((x + y) * 3 % 239));

        private static (byte, byte, byte) Flat(int x, int y) => (16, 16, 16);

        private (string Output, int Exit) RunCSharp(string[] args)
        {
            TextWriter previous = Console.Out;
            var captured = new StringWriter();
            try
            {
                Console.SetOut(captured);
                int exit = CompareRunner.Run(args);
                return (captured.ToString(), exit);
            }
            finally
            {
                Console.SetOut(previous);
            }
        }

        private (string Output, int Exit) RunPython(string[] args)
        {
            var start = new ProcessStartInfo("python3")
            {
                WorkingDirectory = AnalysisDirectory,
                RedirectStandardOutput = true,
                RedirectStandardError = true,
                UseShellExecute = false,
            };
            start.ArgumentList.Add("compare.py");
            // A database per run, so an ingest from one case cannot answer another.
            start.ArgumentList.Add("--db");
            start.ArgumentList.Add(Path.Combine(_dir, Path.GetRandomFileName() + ".db"));
            for (int i = 1; i < args.Length; i++) start.ArgumentList.Add(args[i]);

            using Process process = Process.Start(start)!;
            string output = process.StandardOutput.ReadToEnd();
            string error = process.StandardError.ReadToEnd();
            process.WaitForExit();

            Assert.True(error.Length == 0, $"python3 wrote to stderr:\n{error}");
            return (output, process.ExitCode);
        }

        // The whole point: same argv, same bytes out, same exit code.
        private void AssertIdentical(params string[] args)
        {
            (string csharp, int csharpExit) = RunCSharp(args);
            (string python, int pythonExit) = RunPython(args);

            Assert.Equal(csharp.Replace("\r\n", "\n"), python.Replace("\r\n", "\n"));
            Assert.Equal(csharpExit, pythonExit);
        }

        // CompareRunner takes argv with the verb still at [0].
        private static string[] Argv(string ours, string theirs, long frame, params string[] rest)
        {
            var args = new List<string> { "--compare", ours, theirs, frame.ToString(CultureInfo.InvariantCulture) };
            args.AddRange(rest);
            return args.ToArray();
        }

        [Fact]
        public void A_board_mismatch_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "65", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "", "PaletteIndex16", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_region_mismatch_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "pal", "clean", "Rgba8888", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void An_archaic_header_warning_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "archaic", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_one_sided_board_warning_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "", "ntsc", "clean", "Rgba8888", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void The_column_ratings_read_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 200));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_located_divergence_frame_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100, divergeAt: 25));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_divergence_blamed_on_an_input_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100, divergeAt: 25));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4", "--input", "5", "--input", "20"));
        }

        [Fact]
        public void A_screen_format_difference_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "PaletteIndex16", Rows(40, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_stream_offset_reads_the_same_from_both()
        {
            var shifted = new List<(long, uint, uint)>();
            foreach ((long frame, uint ram, uint work) in Rows(40, 100)) shifted.Add((frame + 2, ram, work));

            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", shifted);

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void Two_agreeing_screens_read_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            for (long f = 16; f <= 24; f++)
            {
                WriteScreen(ours, "emusen", f, (x, y) => Busy(x, y + (int)f));
                WriteScreen(theirs, "mesen", f, (x, y) => Busy(x, y + (int)f));
            }

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_uniform_screen_is_vacuous_the_same_way_in_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            for (long f = 16; f <= 24; f++)
            {
                WriteScreen(ours, "emusen", f, Flat);
                WriteScreen(theirs, "mesen", f, Flat);
            }

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        // 384 differing pixels out of 256x240 is exactly 0.625%, the midpoint case
        // where C#'s away-from-zero rounding and Python's half-to-even disagree.
        [Fact]
        public void The_percentage_midpoint_rounds_the_same_way_in_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            for (long f = 16; f <= 24; f++)
            {
                WriteScreen(ours, "emusen", f, (x, y) => Busy(x, y + (int)f));
                long at = f;
                WriteScreen(theirs, "mesen", f, (x, y) =>
                    at == 20 && (y * 256) + x < 384 ? ((byte)1, (byte)2, (byte)3) : Busy(x, y + (int)at));
            }

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_vertically_shifted_screen_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            for (long f = 16; f <= 24; f++)
            {
                WriteScreen(ours, "emusen", f, (x, y) => Busy(x, y));
                WriteScreen(theirs, "mesen", f, (x, y) => Busy(x, y - 3));
            }

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_static_reference_across_the_window_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            for (long f = 16; f <= 24; f++)
            {
                WriteScreen(ours, "emusen", f, Busy);
                WriteScreen(theirs, "mesen", f, Busy);
            }

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        [Fact]
        public void A_missing_screen_on_one_side_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(40, 100));
            WriteScreen(ours, "emusen", 20, Busy);

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }

        // The committed Mesen dump set: real manifests and real PaletteIndex16
        // screens, so the NES palette decode is exercised on actual frames rather
        // than on generated ones. Compared against itself because it is the only
        // real set in the repo - the emusen side of a genuine pair is 27 MB and is
        // produced by --probe, not committed. See EmuSen_Stack.md §3 for the
        // emusen-vs-mesen result this stands in for.
        [Fact]
        public void The_committed_reference_dump_reads_the_same_from_both()
        {
            string dumps = Path.Combine(ConfigRoot.Directory, "EmuSen.WiseMan", "Reference", "dumps", "SMB3");
            Assert.True(Directory.Exists(dumps), $"the committed dump set is missing: {dumps}");

            AssertIdentical(Argv(dumps, dumps, 160, "6"));
        }

        [Fact]
        public void A_set_with_no_signature_at_all_reads_the_same_from_both()
        {
            string ours = WriteSet("emusen", "4", "ntsc", "clean", "Rgba8888", Rows(0, 100));
            string theirs = WriteSet("mesen", "4", "ntsc", "clean", "Rgba8888", Rows(0, 100));

            AssertIdentical(Argv(ours, theirs, 20, "4"));
        }
    }
}
