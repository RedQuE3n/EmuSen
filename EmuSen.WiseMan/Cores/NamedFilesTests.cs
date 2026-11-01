using System;
using System.IO;
using System.Linq;
using EmuSen.Cores;
using EmuSen.Cores.Native;
using EmuSen.Galaxia.Media;

namespace EmuSen.WiseMan.Cores
{
    // The store of a core's named files held to §6.41's rule for names and to no link followed, and the one image path a host gives - see EmuSen_LibretroHost.md §10d.
    public sealed class NamedFilesTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenNamedFiles_" + Guid.NewGuid().ToString("N"));

        public NamedFilesTests() => Directory.CreateDirectory(_root);

        public void Dispose()
        {
            try { Directory.Delete(_root, recursive: true); } catch (IOException) { }
        }

        [Theory]
        [InlineData("notes.txt", true)]
        [InlineData("memcards/Mcd001.ps2", true)]
        [InlineData("", false)]
        [InlineData("/etc/passwd", false)]
        [InlineData("a//b", false)]
        [InlineData("../escape", false)]
        [InlineData("a/./b", false)]
        [InlineData("a\\b", false)]
        [InlineData("c:x", false)]
        [InlineData("nul\0", false)]
        public void A_name_is_a_relative_path_of_plain_parts(string name, bool valid) => Assert.Equal(valid, NamedFiles.Valid(name));

        [Fact]
        public void A_name_of_more_than_255_bytes_is_refused() => Assert.False(NamedFiles.Valid(new string('é', 128)));

        // Stored files come back in ordinal order from FILE_NAMED; a link is neither read nor written through.
        [Fact]
        public void Files_are_read_back_numbered_and_no_link_is_followed()
        {
            var store = new NamedFiles(Path.Combine(_root, "store"), shared: true);
            Assert.True(store.Write("b/two.bin", new byte[] { 2 }));
            Assert.True(store.Write("a.bin", new byte[] { 1 }));
            string outside = Path.Combine(_root, "outside");
            Directory.CreateDirectory(outside);
            File.WriteAllBytes(Path.Combine(outside, "secret"), new byte[] { 9 });
            Directory.CreateSymbolicLink(Path.Combine(store.Folder, "linked"), outside);
            Assert.Equal(new[] { (CoreInterface.FileNamed, "a.bin"), (CoreInterface.FileNamed + 1, "b/two.bin") }, store.Read().Select(f => (f.Which, f.Name)));
            Assert.False(store.Write("linked/secret", new byte[] { 0 }));
            Assert.Equal(new byte[] { 9 }, File.ReadAllBytes(Path.Combine(outside, "secret")));
        }

        [Fact]
        public void A_removed_files_last_copy_is_kept_beside_the_folder_and_not_read_again()
        {
            var store = new NamedFiles(Path.Combine(_root, "store"), shared: true);
            store.Write("x/gone.txt", "last"u8.ToArray());
            Assert.True(store.KeepRemoved("x/gone.txt"));
            Assert.Empty(store.Read());
            Assert.Equal("last", File.ReadAllText(Path.Combine(store.RemovedFolder, "x", "gone.txt")));
        }

        // §6.41: a path only where the bytes passed are the file's own; a patched image has none.
        [Fact]
        public void Only_an_unaltered_image_has_a_path()
        {
            string path = Path.Combine(_root, "game.bin");
            File.WriteAllBytes(path, new byte[] { 1, 2, 3, 4 });
            using (var plain = DiscFiles.Open(path)) Assert.Equal(path, plain.UnalteredPath);
            using var patched = DiscFiles.Open(path);
            string other = Path.Combine(_root, "patched.bin");
            File.WriteAllBytes(other, new byte[] { 5 });
            patched.Patch(MappedFile.Open(other));
            Assert.Null(patched.UnalteredPath);
        }
    }
}
