using System;
using System.Collections.Generic;
using System.IO;
using System.IO.Compression;
using System.Linq;
using System.Net;
using System.Net.Http;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Themes downloaded from a fake GitHub, updated with theme-customizations kept, refused when broken, and removed only when Mistress downloaded them - see EmuSen_BigPicture.md §16.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeDownloadsTests : IDisposable
    {
        public const string Sha1 = "1111111aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
        public const string Sha2 = "2222222bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenThemeDownloads", Guid.NewGuid().ToString("N"));

        public ThemeDownloadsTests() => DataStore.OverrideDirectory = _root;

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A theme written for the tests, zipped as GitHub's archive is: one folder named for the repository and branch.
        public static byte[] Archive(string marker, string top = "art-book-next-es-de-main/", string? capabilities = null, string? themeXml = null, string? extra = null)
        {
            using var memory = new MemoryStream();
            using (var zip = new ZipArchive(memory, ZipArchiveMode.Create, leaveOpen: true))
            {
                void Put(string name, string text)
                {
                    using var w = new StreamWriter(zip.CreateEntry(name).Open());
                    w.Write(text);
                }

                zip.CreateEntry(top);
                if (capabilities is not "") Put(top + "capabilities.xml", capabilities ?? "<themeCapabilities><themeName>Synthetic Book</themeName></themeCapabilities>");
                Put(top + "theme.xml", themeXml ?? "<theme><view name=\"system\"><text name=\"t\"><text>hello</text></text></view></theme>");
                Put(top + "marker.txt", marker);
                Put(top + "README.md", "# Synthetic Book\n\n## Credits\n* Drawn by [Nobody](https://example.invalid/nobody)\n\n## License\nThe Synthetic Licence 1.0\n");
                if (extra is not null) Put(extra, "x");
            }
            return memory.ToArray();
        }

        public static OnlineCoverTests.FakeServer GitHub(Func<byte[]> archive, Func<string> sha)
        {
            ThemeSource source = ThemeSource.ArtBookNext;
            return new OnlineCoverTests.FakeServer
            {
                Answer = url =>
                {
                    if (url == source.CommitAddress) return new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent($"{{\"sha\":\"{sha()}\",\"commit\":{{}}}}") };
                    if (url == source.ArchiveAddress) return new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(archive()) };
                    return new HttpResponseMessage(HttpStatusCode.NotFound);
                },
            };
        }

        private static string Installed => ThemeDownloads.DirectoryFor(ThemeSource.ArtBookNext);

        // Everything under home/Themes but the theme folder itself: a stray .part, .old or .zip.part is a failure.
        private static string[] Strays() => Directory.Exists(ThemeDownloads.Root)
            ? Directory.GetFileSystemEntries(ThemeDownloads.Root).Where(p => p != Installed).Select(Path.GetFileName).OfType<string>().ToArray()
            : [];

        [Fact]
        public async Task A_theme_downloads_from_GitHub_s_archive_and_is_stamped_with_the_branch_s_commit()
        {
            var server = GitHub(() => Archive("one"), () => Sha1);
            using var http = new HttpClient(server);
            ThemeStamp stamp = await ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext);

            Assert.Equal(Sha1, stamp.Commit);
            Assert.Equal("one", File.ReadAllText(Path.Combine(Installed, "marker.txt")));
            Assert.Equal(Sha1, ThemeDownloads.Stamp(Installed)!.Commit);
            Assert.True(ThemeDownloads.IsDownloaded(Installed));
            Assert.Null(ThemeDownloads.Validate(Installed));
            Assert.Equal(new[] { "Synthetic Book" }, ThemeDownloads.Installed().Select(t => t.Name));
            Assert.Empty(Strays());
            Assert.Contains(ThemeSource.ArtBookNext.ArchiveAddress, server.Asked);
            Assert.Equal("https://codeload.github.com/anthonycaccese/art-book-next-es-de/zip/refs/heads/main", ThemeSource.ArtBookNext.ArchiveAddress);
            Assert.Equal("https://api.github.com/repos/anthonycaccese/art-book-next-es-de/commits/main", ThemeSource.ArtBookNext.CommitAddress);
        }

        // P54: the player's theme-customizations survive an update byte for byte, over whatever the archive carried there.
        [Fact]
        public async Task An_update_replaces_the_theme_and_keeps_theme_customizations_byte_for_byte()
        {
            string sha = Sha1, marker = "one";
            var server = GitHub(() => Archive(marker, extra: "art-book-next-es-de-main/theme-customizations/upstream.txt"), () => sha);
            using var http = new HttpClient(server);
            await ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext);
            string custom = Path.Combine(Installed, ThemeDownloads.Customizations);
            Directory.Delete(custom, true);
            Directory.CreateDirectory(Path.Combine(custom, "artwork"));
            byte[] colours = Enumerable.Range(0, 5000).Select(i => (byte)(i * 7)).ToArray();
            File.WriteAllBytes(Path.Combine(custom, "colors.xml"), colours);
            File.WriteAllBytes(Path.Combine(custom, "artwork", "snes.png"), colours.Reverse().ToArray());

            (sha, marker) = (Sha2, "two");
            await ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext);

            Assert.Equal("two", File.ReadAllText(Path.Combine(Installed, "marker.txt")));
            Assert.Equal(Sha2, ThemeDownloads.Stamp(Installed)!.Commit);
            Assert.Equal(colours, File.ReadAllBytes(Path.Combine(custom, "colors.xml")));
            Assert.Equal(colours.Reverse().ToArray(), File.ReadAllBytes(Path.Combine(custom, "artwork", "snes.png")));
            Assert.False(File.Exists(Path.Combine(custom, "upstream.txt")));
            Assert.Empty(Strays());
        }

        public static IEnumerable<object[]> Broken() =>
        [
            ["server error", null!],
            ["no capabilities", Archive("bad", capabilities: "")],
            ["malformed capabilities", Archive("bad", capabilities: "<themeCapabilities><variant")],
            ["theme.xml that does not load", Archive("bad", themeXml: "<theme><view name=\"system\"><nosuchelement name=\"x\"/></view></theme>")],
            ["an entry outside the folder", Archive("bad", extra: "../escaped.txt")],
        ];

        // P54: a download that fails, holds no theme, or would write outside its folder leaves the old theme as it was, and nothing beside it.
        [Theory]
        [MemberData(nameof(Broken))]
        public async Task A_failed_empty_broken_or_escaping_download_leaves_the_old_theme(string why, byte[]? archive)
        {
            using (var good = new HttpClient(GitHub(() => Archive("one"), () => Sha1))) await ThemeDownloads.FetchAsync(good, ThemeSource.ArtBookNext);
            var server = archive is null
                ? new OnlineCoverTests.FakeServer { Answer = _ => new HttpResponseMessage(HttpStatusCode.InternalServerError) }
                : GitHub(() => archive, () => Sha2);
            using var http = new HttpClient(server);
            await Assert.ThrowsAnyAsync<Exception>(() => ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext));
            Assert.Equal("one", File.ReadAllText(Path.Combine(Installed, "marker.txt")));
            Assert.Equal(Sha1, ThemeDownloads.Stamp(Installed)!.Commit);
            Assert.Empty(Strays());
            Assert.False(File.Exists(Path.Combine(_root, "escaped.txt")), why);
            Assert.False(File.Exists(Path.Combine(ThemeDownloads.Root, "escaped.txt")), why);
        }

        // P55's store half: a cancel during the body stops the download and leaves the old theme and no partial file.
        [Fact]
        public async Task A_cancelled_download_leaves_the_old_theme_and_no_partial_file()
        {
            using (var good = new HttpClient(GitHub(() => Archive("one"), () => Sha1))) await ThemeDownloads.FetchAsync(good, ThemeSource.ArtBookNext);
            var stall = new Stall();
            using var http = new HttpClient(stall.Server);
            using var cancel = new CancellationTokenSource();
            Task<ThemeStamp> fetch = ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext, cancel: cancel.Token);
            await stall.Started.Task.WaitAsync(TimeSpan.FromSeconds(10));
            Assert.True(File.Exists(Installed + ".zip.part"));
            cancel.Cancel();
            await Assert.ThrowsAnyAsync<OperationCanceledException>(() => fetch.WaitAsync(TimeSpan.FromSeconds(10)));
            Assert.Equal("one", File.ReadAllText(Path.Combine(Installed, "marker.txt")));
            Assert.Empty(Strays());
        }

        // A server whose archive sends a few bytes and then waits until the request is cancelled.
        public sealed class Stall
        {
            public readonly TaskCompletionSource Started = new(TaskCreationOptions.RunContinuationsAsynchronously);
            public readonly OnlineCoverTests.FakeServer Server;

            public Stall() => Server = new OnlineCoverTests.FakeServer
            {
                Answer = url => url == ThemeSource.ArtBookNext.ArchiveAddress
                    ? new HttpResponseMessage(HttpStatusCode.OK) { Content = new StreamContent(new StallingStream(Started)) }
                    : new HttpResponseMessage(HttpStatusCode.OK) { Content = new StringContent($"{{\"sha\":\"{Sha2}\"}}") },
            };
        }

        private sealed class StallingStream(TaskCompletionSource started) : Stream
        {
            private bool _sent;
            public override bool CanRead => true;
            public override bool CanSeek => false;
            public override bool CanWrite => false;
            public override long Length => throw new NotSupportedException();
            public override long Position { get => throw new NotSupportedException(); set => throw new NotSupportedException(); }
            public override void Flush() { }
            public override int Read(byte[] buffer, int offset, int count) => ReadAsync(buffer.AsMemory(offset, count)).AsTask().GetAwaiter().GetResult();
            public override long Seek(long offset, SeekOrigin origin) => throw new NotSupportedException();
            public override void SetLength(long value) => throw new NotSupportedException();
            public override void Write(byte[] buffer, int offset, int count) => throw new NotSupportedException();

            public override async ValueTask<int> ReadAsync(Memory<byte> buffer, CancellationToken cancel = default)
            {
                if (!_sent)
                {
                    _sent = true;
                    buffer.Span[0] = (byte)'P';
                    return 1;
                }
                started.TrySetResult();
                await Task.Delay(Timeout.Infinite, cancel);
                return 0;
            }
        }

        [Fact]
        public async Task Removal_refuses_a_folder_Mistress_did_not_download()
        {
            using var theme = new SyntheticTheme();
            theme.Capabilities("").Theme("<view name=\"system\"><text name=\"t\"><text>x</text></text></view>");
            Assert.Throws<InvalidOperationException>(() => ThemeDownloads.Remove(theme.Root));
            Assert.True(File.Exists(theme.PathOf("capabilities.xml")));

            string lookalike = Path.Combine(ThemeDownloads.Root, "lookalike");
            Directory.CreateDirectory(lookalike);
            File.WriteAllText(Path.Combine(lookalike, "capabilities.xml"), "<themeCapabilities/>");
            Assert.Throws<InvalidOperationException>(() => ThemeDownloads.Remove(lookalike));
            Assert.True(Directory.Exists(lookalike));

            using (var http = new HttpClient(GitHub(() => Archive("one"), () => Sha1))) await ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext);
            ThemeDownloads.Remove(Installed);
            Assert.False(Directory.Exists(Installed));
        }

        // The attribution is read from the theme's own README at display time, and a folder with neither README nor LICENSE says so.
        [Fact]
        public async Task The_attribution_is_read_from_the_theme_s_own_files()
        {
            using (var http = new HttpClient(GitHub(() => Archive("one"), () => Sha1))) await ThemeDownloads.FetchAsync(http, ThemeSource.ArtBookNext);
            ThemeAttribution about = ThemeAttribution.Read(Installed);
            Assert.Equal("Synthetic Book", about.Name);
            Assert.Equal(new[] { "The Synthetic Licence 1.0" }, about.Licence);
            Assert.Equal(new[] { "• Drawn by Nobody (https://example.invalid/nobody)" }, about.Credits);
            Assert.StartsWith("anthonycaccese", about.Author);
            Assert.Equal("https://github.com/anthonycaccese/art-book-next-es-de, branch main", about.Source);
            Assert.Equal(Sha1, about.Commit);
            Assert.Contains("not part of EmuSen", about.Statement);

            File.WriteAllText(Path.Combine(Installed, "README.md"), "# Nothing here\n");
            File.WriteAllText(Path.Combine(Installed, "LICENSE"), "\nMIT License\n\nCopyright someone\n");
            Assert.Equal(new[] { "MIT License", "Copyright someone" }, ThemeAttribution.Read(Installed).Licence);
            File.Delete(Path.Combine(Installed, "LICENSE"));
            Assert.Contains("states no licence", ThemeAttribution.Read(Installed).Licence.Single());
        }

        // The licence line and credits of the real theme, read in place; it skips visibly without the clone.
        [ArtBookNextFact]
        public void Art_Book_Next_s_attribution_names_its_licence_and_credits()
        {
            string dir = ArtBookNextFactAttribute.Folder;
            ThemeAttribution about = ThemeAttribution.Read(dir);
            Assert.Equal("Art Book Next", about.Name);
            Assert.Contains(about.Licence, l => l.Contains("CC-BY-NC-SA") && l.Contains("creativecommons.org/licenses/by-nc-sa/2.0"));
            Assert.True(about.Credits.Count >= 5, string.Join(" | ", about.Credits));
            Assert.All(about.Credits, c => Assert.StartsWith("• ", c));
            Assert.StartsWith("read in place", about.Source);
        }
    }
}
