using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Threading;
using EmuSen.Galaxia.Media;

namespace EmuSen.WiseMan.Galaxia
{
    // A file mapped by Galaxia's C# and by the library: the same bytes at one address for its life, and the same exception for what cannot be mapped - see EmuSen_Galaxia.md §3.8.
    public partial class GalaxiaParityTests
    {
        private static (Type? Fault, byte[] Bytes) MapWith(Func<string, MappedFile> open, string path)
        {
            try
            {
                using MappedFile mapped = open(path);
                unsafe { Assert.True(mapped.Length == 0 ? mapped.Pointer == null : mapped.Pointer != null); }
                return (null, mapped.Span(0, (int)mapped.Length).ToArray());
            }
            catch (Exception e) { return (e.GetType(), Array.Empty<byte>()); }
        }

        [Fact]
        public void A_file_is_mapped_to_the_same_bytes_and_refused_with_the_same_exception_both_ways()
        {
            byte[] bytes = new byte[300_007];
            new Random(9).NextBytes(bytes);
            string file = Put(At("disc", "track.bin")), empty = Put(At("disc", "empty.bin"), "");
            File.WriteAllBytes(file, bytes);
            foreach (string path in new[] { file, empty, At("disc", "absent.bin"), At("no folder", "absent.bin"), At("disc") })
            {
                var csharp = MapWith(MappedFile.Managed.Open, path);
                var library = MapWith(MappedFile.Open, path);
                Assert.Equal(csharp.Fault, library.Fault);
                Assert.Equal(csharp.Bytes, library.Bytes);
            }
            Assert.Equal(bytes, MapWith(MappedFile.Open, file).Bytes);
            Assert.Equal((typeof(FileNotFoundException), typeof(DirectoryNotFoundException), typeof(UnauthorizedAccessException)),
                (MapWith(MappedFile.Open, At("disc", "absent.bin")).Fault, MapWith(MappedFile.Open, At("no folder", "absent.bin")).Fault, MapWith(MappedFile.Open, At("disc")).Fault));
            using MappedFile held = MappedFile.Open(file);
            Assert.Equal(bytes.AsSpan(300_000).ToArray(), held.Span(300_000, 100).ToArray());
            Assert.True(held.Span(300_007, 1).IsEmpty && held.Span(-1, 1).IsEmpty);
            Assert.Equal(bytes.AsSpan(0, 16).ToArray(), held.Head(16));
        }

        private static string Fault(Action act)
        {
            try { act(); return "none"; }
            catch (Exception e) { return e.GetType().Name; }
        }

        // While a file is mapped, another opener may do the same with it both ways: on Windows a reader comes in and a writer, a rename and a deletion are refused, as FileShare.Read has it - see EmuSen_Galaxia.md §3.8.
        [Fact]
        public void A_held_mapping_lets_in_the_same_openers_both_ways()
        {
            string file = At("held", "track.bin");
            var seen = new List<string>();
            foreach ((string side, Func<string, MappedFile> open) in new (string, Func<string, MappedFile>)[] { ("C#", MappedFile.Managed.Open), ("the library", MappedFile.Open) })
            {
                Put(file);
                File.WriteAllBytes(file, new byte[70_000]);
                using (MappedFile mapped = open(file))
                {
                    string reader = Fault(() => new FileStream(file, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete).Dispose());
                    string writer = Fault(() => new FileStream(file, FileMode.Open, FileAccess.Write, FileShare.ReadWrite | FileShare.Delete).Dispose());
                    string rename = Fault(() => { File.Move(file, file + ".moved"); File.Move(file + ".moved", file); });
                    string deletion = Fault(() => File.Delete(file));
                    seen.Add($"reader {reader}, writer {writer}, rename {rename}, deletion {deletion}");
                    Assert.Equal(70_000, mapped.Length);
                }
                if (File.Exists(file)) File.Delete(file);
            }
            Assert.True(seen[0] == seen[1], $"C#: {seen[0]}; the library: {seen[1]}");
            if (OperatingSystem.IsWindows()) Assert.Equal("reader none, writer IOException, rename IOException, deletion IOException", seen[1]);
        }

        [DllImport("libc", EntryPoint = "mincore")]
        private static extern unsafe int InMemory(byte* address, nuint length, byte* pages);

        [DllImport("libc", EntryPoint = "posix_fadvise")]
        private static extern int FileAdvise(int descriptor, long offset, long length, int advice);

        // How many of the pages of a mapping's range the system holds in memory; Linux's own answer.
        private static unsafe int Resident(MappedFile file, long offset, int length)
        {
            long page = Environment.SystemPageSize, first = offset / page * page;
            byte[] pages = new byte[(offset + length - first + page - 1) / page];
            fixed (byte* p = pages) Assert.Equal(0, InMemory(file.Pointer + first, (nuint)(offset + length - first), p));
            return pages.Count(b => (b & 1) != 0);
        }

        // Advice is taken for the same ranges by Galaxia's C# and by the library, changes no byte, and on Linux brings the advised bytes into memory untouched while the rest stay on the medium - see EmuSen_Galaxia.md §3.9.
        [Fact]
        public void Advice_is_taken_for_the_same_ranges_both_ways_and_the_advised_bytes_reach_memory()
        {
            const int MiB = 1 << 20;
            // Beside the test's own assemblies, on a disk, since a temporary folder kept in memory has nothing to bring in.
            string file = Path.Combine(AppContext.BaseDirectory, $"advised-{Guid.NewGuid():N}.bin");
            byte[] bytes = new byte[16 * MiB];
            new Random(11).NextBytes(bytes);
            try
            {
                using (var stream = new FileStream(file, FileMode.Create, FileAccess.ReadWrite))
                {
                    stream.Write(bytes);
                    stream.Flush(flushToDisk: true);
                }
                string empty = Put(At("disc", "none.bin"), "");
                (long At, long Length)[] ranges = { (0, 4096), (5 * MiB + 1234, 128 * 1024), (16 * MiB - 1, 1), (16 * MiB - 10, 1L << 30), (16 * MiB, 4096), (long.MaxValue, 1), (100, 0), (-1, 4096) };
                bool[] taken = { true, true, true, true, false, false, false, false };
                foreach (Func<string, MappedFile> open in new Func<string, MappedFile>[] { MappedFile.Managed.Open, MappedFile.Open })
                {
                    using MappedFile mapped = open(file);
                    bool advises = OperatingSystem.IsLinux() || OperatingSystem.IsMacOS() || OperatingSystem.IsWindows();
                    Assert.Equal(taken.Select(t => t && advises), ranges.Select(r => mapped.Advise(r.At, r.Length)));
                    Assert.True(mapped.Span(5 * MiB, 4096).SequenceEqual(bytes.AsSpan(5 * MiB, 4096)));
                    using MappedFile none = open(empty);
                    Assert.False(none.Advise(0, 4096));
                }
                if (!OperatingSystem.IsLinux()) return;
                foreach (Func<string, MappedFile> open in new Func<string, MappedFile>[] { MappedFile.Managed.Open, MappedFile.Open })
                {
                    using (var stream = new FileStream(file, FileMode.Open, FileAccess.Read)) Assert.Equal(0, FileAdvise((int)stream.SafeFileHandle.DangerousGetHandle(), 0, 0, 4));
                    using MappedFile mapped = open(file);
                    if (Resident(mapped, 0, 16 * MiB) != 0) continue;
                    const long at = 9 * MiB + 77;
                    const int length = 128 * 1024;
                    Assert.True(mapped.Advise(at, length));
                    long page = Environment.SystemPageSize;
                    int pages = (int)((at + length + page - 1) / page - at / page);
                    var watch = Stopwatch.StartNew();
                    while (Resident(mapped, at, length) < pages && watch.Elapsed < TimeSpan.FromSeconds(10)) Thread.Sleep(1);
                    Assert.Equal(pages, Resident(mapped, at, length));
                    Assert.Equal(0, Resident(mapped, 13 * MiB, MiB));
                    Assert.True(mapped.Span(at, length).SequenceEqual(bytes.AsSpan((int)at, length)));
                }
            }
            finally
            {
                File.Delete(file);
            }
        }
    }
}
