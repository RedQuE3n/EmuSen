using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Net.Http;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.BigPicture;
using EmuSen.WiseMan.Fixtures;
using Microsoft.Data.Sqlite;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // ES-DE's list parsed and kept in themes.db, screenshots cached, the host's answers, install, update, local changes and removal from a fake GitHub and GitLab - see EmuSen_BigPicture.md §25.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeBrowserModelTests : IDisposable
    {
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenThemeBrowser", Guid.NewGuid().ToString("N"));
        private DateTimeOffset _now = new(2026, 9, 26, 12, 0, 0, TimeSpan.Zero);

        public ThemeBrowserModelTests() => DataStore.OverrideDirectory = _root;

        public void Dispose()
        {
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        private ThemeBrowser Browser(FakeThemeHosts hosts) => new(hosts.Client, () => _now);

        private static string Dir(FakeTheme t) => ThemeDownloads.DirectoryFor(t.Source);

        private static async Task Install(FakeThemeHosts hosts, FakeTheme t, bool replace = false)
        {
            using HttpClient http = hosts.Client();
            await ThemeDownloads.FetchAsync(http, t.Source, replaceLocalChanges: replace);
        }

        // Every field the browser shows comes from the list as stated; the Android list, and entries on no GitHub or GitLab, are left out and named.
        [Fact]
        public void The_list_parses_every_field_the_browser_shows()
        {
            var hosts = FakeThemeHosts.Standard();
            hosts.NotListed.Add("elsewhere-theme");
            ThemeList list = ThemeList.Parse(hosts.ListJson(), _now);
            Assert.Equal(new[] { "Synthetic Book", "Plain Shelf", "Lab Wheel" }, list.Themes.Select(t => t.Name));
            Assert.Equal(new[] { "elsewhere-theme" }, list.Skipped);
            Assert.Equal("99", list.LatestStableRelease);
            ThemeListEntry book = list.Themes[0];
            Assert.Equal(("synthetic-book-es-de", "Ada Example", false), (book.RepoName, book.Author, book.NewEntry));
            Assert.Equal(new[] { "List", "Grid" }, book.Variants);
            Assert.Equal(new[] { "Dark", "Light", "Night" }, book.ColorSchemes);
            Assert.Equal(new[] { "16:9", "16:10", "4:3" }, book.AspectRatios);
            Assert.Equal(new[] { "Medium", "Large" }, book.FontSizes);
            Assert.Equal(new[] { "Instant", "Slide" }, book.Transitions);
            Assert.Equal(new[] { "en_US", "de_DE" }, book.Languages);
            Assert.Equal(3, book.Screenshots.Count);
            Assert.Equal("Synthetic Book, screenshot 2", book.Screenshots[1].Caption);
            Assert.Equal(new ThemeSource("ada", "synthetic-book-es-de", "", ThemeHost.GitHub), book.Source);
            Assert.Equal(new ThemeSource("cy-group/themes", "lab-wheel-es-de", "", ThemeHost.GitLab), list.Themes[2].Source);
            Assert.True(list.Themes[2].NewEntry);
            Assert.Empty(list.Themes[1].ColorSchemes);
            Assert.Equal("2 variants · 3 colour schemes · 3 aspect ratios", EmuSen.Mistress.Views.ThemeBrowserWindow.Counts(book));
            Assert.Equal("2 variants · no colour schemes stated · 3 aspect ratios", EmuSen.Mistress.Views.ThemeBrowserWindow.Counts(list.Themes[1]));
        }

        // The hosts' addresses, GitLab's with its groups; a path that climbs out of the list's repository or a URL on another host is refused.
        [Fact]
        public void Each_host_s_addresses_are_built_from_the_list_s_url()
        {
            ThemeSource hub = ThemeSource.FromUrl("https://github.com/RickAndTired/adroit-es-de.git", "main")!;
            Assert.Equal("https://codeload.github.com/RickAndTired/adroit-es-de/zip/refs/heads/main", hub.ArchiveAddress);
            Assert.Equal("https://api.github.com/repos/RickAndTired/adroit-es-de", hub.RepositoryAddress);
            Assert.Equal("https://raw.githubusercontent.com/RickAndTired/adroit-es-de/main/README.md", hub.ReadmeAddress);
            ThemeSource lab = ThemeSource.FromUrl("https://gitlab.com/es-de/themes/modern-es-de.git", "master")!;
            Assert.Equal(("es-de/themes", "modern-es-de", ThemeHost.GitLab), (lab.Owner, lab.Repository, lab.Host));
            Assert.Equal("https://gitlab.com/es-de/themes/modern-es-de/-/archive/master/modern-es-de-master.zip", lab.ArchiveAddress);
            Assert.Equal("https://gitlab.com/api/v4/projects/es-de%2Fthemes%2Fmodern-es-de/repository/commits/master", lab.CommitAddress);
            Assert.Equal("https://gitlab.com/api/v4/projects/es-de%2Fthemes%2Fmodern-es-de?license=true", lab.RepositoryAddress);
            Assert.Null(ThemeSource.FromUrl("https://example.org/x/y.git"));
            Assert.Null(ThemeSource.FromUrl("http://github.com/x/y.git"));
            Assert.Null(ThemeSource.FromUrl("https://github.com/a/b/c.git"));
            Assert.Null(ThemeList.ScreenshotAddress("../secret.png"));
            Assert.Equal(ThemeList.RepositoryRaw + "screenshots/a%20b/c.jpg", ThemeList.ScreenshotAddress("screenshots/a b/c.jpg"));
            Assert.Throws<InvalidDataException>(() => ThemeDownloads.DirectoryFor(new ThemeSource("x", "..", "main")));
            Assert.Throws<InvalidDataException>(() => ThemeDownloads.DirectoryFor(new ThemeSource("x", ".list", "main")));
        }

        // The list is kept in themes.db and read back equal; within a day it is not asked again, after a day or on Refresh it is.
        [Fact]
        public async Task The_list_is_kept_in_themes_db_and_asked_again_only_after_a_day_or_on_refresh()
        {
            var hosts = FakeThemeHosts.Standard();
            using ThemeBrowser browser = Browser(hosts);
            Assert.Null(ThemeBrowser.Cached());
            ThemeList first = await browser.ListAsync(refresh: false);
            Assert.Equal(1, hosts.AskedFor(ThemeList.Address));
            ThemeList kept = ThemeBrowser.Cached()!;
            Assert.Equal(JsonSerializer.Serialize(first.Themes), JsonSerializer.Serialize(kept.Themes));
            Assert.Equal(first.FetchedAt, kept.FetchedAt);

            _now += TimeSpan.FromHours(23);
            await browser.ListAsync(refresh: false);
            Assert.Equal(1, hosts.AskedFor(ThemeList.Address));
            await browser.ListAsync(refresh: true);
            Assert.Equal(2, hosts.AskedFor(ThemeList.Address));
            _now += TimeSpan.FromHours(25);
            await browser.ListAsync(refresh: false);
            Assert.Equal(3, hosts.AskedFor(ThemeList.Address));
            Assert.Equal(3, hosts.Requests);
        }

        // A screenshot is fetched once and kept as a file indexed in themes.db; after a month it is asked again.
        [Fact]
        public async Task A_screenshot_is_fetched_once_and_kept_for_a_month()
        {
            var hosts = FakeThemeHosts.Standard();
            using ThemeBrowser browser = Browser(hosts);
            ThemeList list = await browser.ListAsync(false);
            ThemeScreenshot shot = list.Themes[0].Screenshots[1];
            string? file = await browser.ScreenshotAsync(shot);
            Assert.NotNull(file);
            Assert.StartsWith(ThemeBrowser.ScreenshotFolder, file);
            Assert.Equal(FakeThemeHosts.Picture("Synthetic Book", 2), File.ReadAllBytes(file!));
            int asked = hosts.Requests;
            Assert.Equal(file, await browser.ScreenshotAsync(shot));
            Assert.Equal(asked, hosts.Requests);
            using (ThemeRecords db = ThemeRecords.Open()) Assert.Equal(new FileInfo(file!).Length, db.Screenshot(shot.Image)!.Bytes);
            _now += TimeSpan.FromDays(31);
            await browser.ScreenshotAsync(shot);
            Assert.Equal(asked + 1, hosts.Requests);
            Assert.Empty(Directory.GetFiles(ThemeBrowser.ScreenshotFolder, "*.part"));
        }

        // The licence line: the README's licence section, else the host's named licence, else a statement that there is none.
        [Fact]
        public async Task The_licence_line_comes_from_the_readme_then_the_host_then_says_none()
        {
            var hosts = FakeThemeHosts.Standard();
            using ThemeBrowser browser = Browser(hosts);
            ThemeList list = await browser.ListAsync(false);
            ThemeRemote book = await browser.RemoteAsync(list.Themes[0]);
            Assert.Equal(("main", "readme"), (book.Branch, book.LicenceFrom));
            Assert.Equal(new[] { "Creative Commons BY-NC-SA 4.0, as stated for this synthetic theme." }, book.Licence);
            Assert.Equal(hosts["Synthetic Book"].Sha, book.Head);
            Assert.Equal(hosts["Synthetic Book"].Date, book.HeadDate);
            ThemeRemote shelf = await browser.RemoteAsync(list.Themes[1]);
            Assert.Equal(("master", "host"), (shelf.Branch, shelf.LicenceFrom));
            Assert.Equal("MIT License, as GitHub reads the repository's licence file.", shelf.Licence.Single());
            ThemeRemote wheel = await browser.RemoteAsync(list.Themes[2]);
            Assert.Equal(("trunk", "none"), (wheel.Branch, wheel.LicenceFrom));
            Assert.Contains("states no licence", wheel.Licence.Single());

            int asked = hosts.Requests;
            await browser.RemoteAsync(list.Themes[0]);
            Assert.Equal(asked, hosts.Requests);
            Assert.Equal("readme", ThemeBrowser.KnownRemote(list.Themes[0])!.LicenceFrom);
        }

        // Installed, update available and local changes, each as the browser marks it; a file re-timed but unchanged, an added file and theme-customizations are not local changes.
        [Fact]
        public async Task Installed_update_and_local_changes_are_told_apart()
        {
            var hosts = FakeThemeHosts.Standard();
            FakeTheme book = hosts["Synthetic Book"];
            using ThemeBrowser browser = Browser(hosts);
            ThemeList list = await browser.ListAsync(false);
            Assert.All(ThemeBrowser.Entries(list), e => Assert.Equal(ThemeListState.NotInstalled, e.State));

            ThemeRemote remote = await browser.RemoteAsync(list.Themes[0]);
            await browser.DownloadAsync(list.Themes[0], remote, null, null, false, default);
            ThemeBrowserEntry e0 = ThemeBrowser.Entries(list)[0];
            Assert.Equal(ThemeListState.Installed, e0.State);
            Assert.True(e0.Changes.Known);
            Assert.Equal(ThemeListState.NotInstalled, ThemeBrowser.Entries(list)[1].State);

            string dir = Dir(book);
            File.SetLastWriteTimeUtc(Path.Combine(dir, "marker.txt"), DateTime.UtcNow.AddDays(1));
            File.WriteAllText(Path.Combine(dir, "mine.xml"), "<theme/>");
            Directory.CreateDirectory(Path.Combine(dir, ThemeDownloads.Customizations));
            File.WriteAllText(Path.Combine(dir, ThemeDownloads.Customizations, "colors.xml"), "custom");
            Assert.Equal(ThemeListState.Installed, ThemeBrowser.Entries(list)[0].State);

            File.WriteAllText(Path.Combine(dir, "colors.xml"), "<theme><variables><bg>FF0000</bg></variables></theme>");
            File.Delete(Path.Combine(dir, "marker.txt"));
            ThemeLocalChanges changes = ThemeDownloads.LocalChanges(dir);
            Assert.Equal(new[] { "colors.xml" }, changes.Modified);
            Assert.Equal(new[] { "marker.txt" }, changes.Missing);
            Assert.Equal(ThemeListState.Installed | ThemeListState.LocalChanges, ThemeBrowser.Entries(list)[0].State);

            book.Sha = "2222222bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
            await browser.HeadAsync(list.Themes[0], ThemeBrowser.Entries(list)[0].Installed!, refresh: true);
            Assert.Equal(ThemeListState.Installed | ThemeListState.LocalChanges | ThemeListState.UpdateAvailable, ThemeBrowser.Entries(list)[0].State);
            Assert.Equal("Installed, update available, local changes", EmuSen.Mistress.Views.ThemeBrowserWindow.StateText(ThemeBrowser.Entries(list)[0]));
        }

        // An update over local changes is refused unless the player agreed; agreed, it replaces them, keeps theme-customizations and the files the player added, and a file the update adds wins.
        [Fact]
        public async Task An_update_asks_before_replacing_local_changes_and_keeps_the_player_s_files()
        {
            var hosts = FakeThemeHosts.Standard();
            FakeTheme book = hosts["Synthetic Book"];
            await Install(hosts, book);
            string dir = Dir(book);
            File.WriteAllText(Path.Combine(dir, "colors.xml"), "edited");
            File.WriteAllText(Path.Combine(dir, "mine.xml"), "mine");
            File.WriteAllText(Path.Combine(dir, "later.xml"), "the player's");
            Directory.CreateDirectory(Path.Combine(dir, ThemeDownloads.Customizations, "art"));
            byte[] custom = Enumerable.Range(0, 5000).Select(i => (byte)(i * 13)).ToArray();
            File.WriteAllBytes(Path.Combine(dir, ThemeDownloads.Customizations, "art", "snes.png"), custom);

            (book.Sha, book.Marker) = ("2222222bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "two");
            book.Extra["later.xml"] = "upstream";
            var refused = await Assert.ThrowsAsync<ThemeLocalChangesException>(() => Install(hosts, book));
            Assert.Equal(new[] { "colors.xml" }, refused.Changes.Modified);
            Assert.Equal("one", File.ReadAllText(Path.Combine(dir, "marker.txt")));
            Assert.Equal("edited", File.ReadAllText(Path.Combine(dir, "colors.xml")));

            await Install(hosts, book, replace: true);
            Assert.Equal("two", File.ReadAllText(Path.Combine(dir, "marker.txt")));
            Assert.StartsWith("<theme>", File.ReadAllText(Path.Combine(dir, "colors.xml")));
            Assert.Equal("mine", File.ReadAllText(Path.Combine(dir, "mine.xml")));
            Assert.Equal("upstream", File.ReadAllText(Path.Combine(dir, "later.xml")));
            Assert.Equal(custom, File.ReadAllBytes(Path.Combine(dir, ThemeDownloads.Customizations, "art", "snes.png")));
            Assert.Equal(book.Sha, ThemeDownloads.Stamp(dir)!.Commit);
            Assert.False(ThemeDownloads.LocalChanges(dir).Any);
            Assert.Empty(Directory.GetFileSystemEntries(ThemeDownloads.Root).Select(Path.GetFileName).Where(n => n!.EndsWith(".part") || n.EndsWith(".old")));
        }

        // A GitLab theme downloads from GitLab's archive, into a folder named for its repository, with GitLab recorded as its host.
        [Fact]
        public async Task A_GitLab_theme_downloads_from_GitLab_s_archive()
        {
            var hosts = FakeThemeHosts.Standard();
            FakeTheme wheel = hosts["Lab Wheel"];
            await Install(hosts, wheel);
            Assert.Contains(wheel.Source.ArchiveAddress, hosts.Asked);
            Assert.Contains(wheel.Source.CommitAddress, hosts.Asked);
            ThemeStamp stamp = ThemeDownloads.Stamp(Dir(wheel))!;
            Assert.Equal((ThemeHost.GitLab, "cy-group/themes", "trunk", wheel.Sha), (stamp.Host, stamp.Owner, stamp.Branch, stamp.Commit));
            Assert.Equal(wheel.Date, stamp.Committed);
            Assert.Equal("Lab Wheel", ThemeDownloads.NameOf(Dir(wheel)));
            Assert.Contains("GitLab", ThemeAttribution.Read(Dir(wheel)).Author);
        }

        // Nothing Mistress did not download is removed or replaced: a folder read in place, a look-alike, and a folder copied by hand over a downloaded one's path.
        [Fact]
        public async Task Removal_and_replacement_refuse_every_folder_Mistress_did_not_write()
        {
            var hosts = FakeThemeHosts.Standard();
            FakeTheme book = hosts["Synthetic Book"];
            string dir = Dir(book);

            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, "capabilities.xml"), "<themeCapabilities/>");
            Assert.Throws<InvalidOperationException>(() => ThemeDownloads.Remove(dir));
            await Assert.ThrowsAsync<InvalidOperationException>(() => Install(hosts, book));
            Assert.True(File.Exists(Path.Combine(dir, "capabilities.xml")));
            Directory.Delete(dir, true);

            await Install(hosts, book);
            string stamp = File.ReadAllText(Path.Combine(dir, ThemeDownloads.StampFile));
            Directory.Delete(dir, true);
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, "capabilities.xml"), "<themeCapabilities/>");
            File.WriteAllText(Path.Combine(dir, ThemeDownloads.StampFile), "{\"Id\":\"someone-else\"}");
            Assert.False(ThemeDownloads.IsDownloaded(dir));
            Assert.Throws<InvalidOperationException>(() => ThemeDownloads.Remove(dir));
            await Assert.ThrowsAsync<InvalidOperationException>(() => Install(hosts, book));
            Assert.True(Directory.Exists(dir));

            File.WriteAllText(Path.Combine(dir, ThemeDownloads.StampFile), stamp);
            Assert.True(ThemeDownloads.IsDownloaded(dir));
            ThemeDownloads.Remove(dir);
            Assert.False(Directory.Exists(dir));
            using ThemeRecords db = ThemeRecords.Open();
            Assert.Empty(db.Installed());
        }

        // Two repositories of one name share a folder name; a download of the second never replaces the first.
        [Fact]
        public async Task A_same_named_repository_of_another_owner_does_not_replace_a_theme()
        {
            var hosts = FakeThemeHosts.Standard();
            FakeTheme book = hosts["Synthetic Book"];
            await Install(hosts, book);
            var twin = new FakeTheme { Name = "Twin Book", Source = new ThemeSource("someone-else", book.Source.Repository, "main"), Marker = "twin" };
            hosts.Themes.Add(twin);
            var refused = await Assert.ThrowsAsync<InvalidOperationException>(() => Install(hosts, twin));
            Assert.Contains("github.com/ada/synthetic-book-es-de", refused.Message);
            Assert.Equal("one", File.ReadAllText(Path.Combine(Dir(book), "marker.txt")));
            Assert.Equal("ada", ThemeDownloads.Stamp(Dir(book))!.Owner);
            Assert.DoesNotContain(twin.Source.ArchiveAddress, hosts.Asked);
        }

        // A theme downloaded by stage (f), whose stamp held the whole record, is moved into themes.db once, its source and commit kept.
        [Fact]
        public void A_stage_f_stamp_is_imported_into_themes_db()
        {
            string dir = Path.Combine(ThemeDownloads.Root, "art-book-next-es-de");
            Directory.CreateDirectory(dir);
            File.WriteAllText(Path.Combine(dir, "capabilities.xml"), "<themeCapabilities><themeName>Old Book</themeName></themeCapabilities>");
            File.WriteAllText(Path.Combine(dir, ThemeDownloads.StampFile),
                JsonSerializer.Serialize(new ThemeStamp("anthonycaccese", "art-book-next-es-de", "main", ThemeDownloadsTests.Sha1, new DateTime(2026, 9, 25, 10, 0, 0, DateTimeKind.Utc))));
            ThemeStamp stamp = ThemeDownloads.Stamp(dir)!;
            Assert.Equal(("anthonycaccese", "main", ThemeDownloadsTests.Sha1, ThemeHost.GitHub), (stamp.Owner, stamp.Branch, stamp.Commit, stamp.Host));
            Assert.Equal(new DateTime(2026, 9, 25, 10, 0, 0, DateTimeKind.Utc), stamp.Downloaded);
            Assert.Equal(stamp.Id, ThemeDownloads.FolderId(dir));
            Assert.DoesNotContain("Owner", File.ReadAllText(Path.Combine(dir, ThemeDownloads.StampFile)));
            Assert.False(ThemeDownloads.LocalChanges(dir).Known);
            Assert.Equal(stamp.Id, ThemeDownloads.Stamp(dir)!.Id);
            using ThemeRecords db = ThemeRecords.Open();
            Assert.Single(db.Installed());
        }

        // themes.db carries its schema version, and a file from a newer build is refused rather than written.
        [Fact]
        public void Themes_db_is_versioned_and_a_newer_one_is_refused()
        {
            using (ThemeRecords.Open()) { }
            Assert.Equal(ThemeRecords.SchemaVersion, UserVersion());
            SetUserVersion(ThemeRecords.SchemaVersion + 1);
            Assert.Throws<InvalidDataException>(() => ThemeRecords.Open());
            Assert.Equal(ThemeRecords.SchemaVersion + 1, UserVersion());
        }

        private static long UserVersion()
        {
            using var c = new SqliteConnection($"Data Source={ThemeRecords.DefaultPath};Pooling=False");
            c.Open();
            using SqliteCommand q = c.CreateCommand();
            q.CommandText = "PRAGMA user_version";
            return (long)q.ExecuteScalar()!;
        }

        private static void SetUserVersion(int v)
        {
            using var c = new SqliteConnection($"Data Source={ThemeRecords.DefaultPath};Pooling=False");
            c.Open();
            using SqliteCommand q = c.CreateCommand();
            q.CommandText = $"PRAGMA user_version = {v}";
            q.ExecuteNonQuery();
        }

        // A browser closed during a download stops it, leaves the old state and no partial file, and holds no file open under home.
        [Fact]
        public async Task A_closed_browser_stops_its_download_and_releases_its_files()
        {
            var hosts = FakeThemeHosts.Standard();
            hosts.StallArchiveOf = "Synthetic Book";
            var browser = Browser(hosts);
            ThemeList list = await browser.ListAsync(false);
            await browser.ScreenshotAsync(list.Themes[0].Screenshots[0]);
            ThemeRemote remote = await browser.RemoteAsync(list.Themes[0]);
            Task<ThemeStamp> download = browser.DownloadAsync(list.Themes[0], remote, null, null, false, CancellationToken.None);
            await hosts.Stalled.Task.WaitAsync(TimeSpan.FromSeconds(10));
            string dir = Dir(hosts["Synthetic Book"]);
            Assert.True(File.Exists(dir + ".zip.part"));
            Assert.NotEmpty(OpenUnder(_root));

            browser.Dispose();
            await Assert.ThrowsAnyAsync<OperationCanceledException>(() => download.WaitAsync(TimeSpan.FromSeconds(5)));
            Assert.False(File.Exists(dir + ".zip.part"));
            Assert.False(Directory.Exists(dir + ".part"));
            Assert.False(Directory.Exists(dir));
            Assert.Empty(OpenUnder(_root));
            Assert.Throws<ObjectDisposedException>(() => { _ = browser.ListAsync(true).GetAwaiter().GetResult(); });
        }

        // The files this process holds open under a folder, read from /proc/self/fd.
        public static string[] OpenUnder(string root)
        {
            var open = new List<string>();
            foreach (string fd in Directory.GetFiles("/proc/self/fd"))
            {
                try
                {
                    if (new FileInfo(fd).LinkTarget is { } target && target.StartsWith(root, StringComparison.Ordinal)) open.Add(target);
                }
                catch (IOException) { }
                catch (UnauthorizedAccessException) { }
            }
            return open.ToArray();
        }
    }
}
