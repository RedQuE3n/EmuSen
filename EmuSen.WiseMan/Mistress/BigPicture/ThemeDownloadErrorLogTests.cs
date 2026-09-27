using System;
using System.IO;
using System.IO.Compression;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using EmuSen.Galaxia.Library;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using static EmuSen.WiseMan.Mistress.BigPicture.ThemeBrowserSheetTests;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A failed theme download leaves its whole reason in the error log as well as on the detail's status line - see EmuSen_Settings_Reference.md §4.70.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemeDownloadErrorLogTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemeDownloadErrorLogTests).GetTypeInfo().Assembly);

        private static readonly FieldInfo Factory = typeof(MainWindow).GetField("HttpFactory", BindingFlags.Static | BindingFlags.NonPublic)!;
        private readonly object _realFactory = Factory.GetValue(null)!;
        private readonly string _dir = Path.Combine(Path.GetTempPath(), "EmuSenThemeErrorLog", Guid.NewGuid().ToString("N"));
        private readonly string? _before = ErrorLog.DirectoryOverride;

        public ThemeDownloadErrorLogTests() => ErrorLog.DirectoryOverride = _dir;

        public void Dispose()
        {
            Factory.SetValue(null, _realFactory);
            ErrorLog.DirectoryOverride = _before;
            try { Directory.Delete(_dir, recursive: true); } catch { }
        }

        private static byte[] NotATheme()
        {
            using var bytes = new MemoryStream();
            using (var zip = new ZipArchive(bytes, ZipArchiveMode.Create))
                using (var w = new StreamWriter(zip.CreateEntry("plain-shelf-es-de-master/readme.txt").Open())) w.Write("no theme here");
            return bytes.ToArray();
        }

        [Fact]
        public Task A_failed_download_is_logged_with_the_theme_and_its_whole_reason() => Session.Dispatch(() =>
        {
            var hosts = FakeThemeHosts.Standard();
            hosts["Plain Shelf"].Archive = NotATheme;
            Serve(hosts);
            using var s = new ThemedSession(settings: a => a.BigPictureTheme = null);
            ThemeDetailWindow detail = OpenDetail(s, OpenBrowser(s, OpenSettings(s)), "Plain Shelf");
            Click(Named<Button>(detail, "ThemeDetailDownload"));
            Assert.True(Pump(() => detail.Downloading is { IsCompleted: true }));
            s.Settle();

            string status = Named<TextBlock>(detail, "ThemeDetailStatus").Text!;
            Assert.StartsWith("The download failed", status);
            string log = File.ReadAllText(ErrorLog.PathFor(DateTime.Now));
            Assert.Contains("ERROR [themes] A theme download failed", log);
            Assert.Contains("context: Plain Shelf (https://github.com/bo/plain-shelf-es-de.git)", log);
            Assert.Contains("The download holds no capabilities.xml.", log);
            Assert.Contains("InvalidDataException", log);
        }, default);
    }
}
