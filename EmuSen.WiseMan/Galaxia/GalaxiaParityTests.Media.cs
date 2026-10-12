using System;
using System.IO;
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
    }
}
