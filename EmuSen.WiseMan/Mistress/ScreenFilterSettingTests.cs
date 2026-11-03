using System;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading;
using System.Threading.Tasks;
using System.Net.Http;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using Avalonia.LogicalTree;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using EmuSen.Mistress.Pages;

namespace EmuSen.WiseMan.Mistress
{
    // The screen filter, one per console, chosen in the graphics window and drawn by the frame control - see EmuSen_Settings_Reference.md §4.40.
    [Collection(TestCollections.ProcessGlobals)]
    public class ScreenFilterSettingTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ScreenFilterSettingTests).GetTypeInfo().Assembly);

        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenScreenFilterTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public ScreenFilterSettingTests()
        {
            string roms = Path.Combine(_root, "Roms");
            Directory.CreateDirectory(roms);
            ConfigStore.OverrideDirectory = Path.Combine(_root, "Config");
            DataStore.OverrideDirectory = Path.Combine(_root, "Home");
            _rom = Path.Combine(roms, "Game.sfc");
            File.WriteAllBytes(_rom, SyntheticRom.BuildBlank());
            new AppSettings { RomDirectory = roms, ResumeOnLaunch = AppSettings.ResumeNever }.Save();
        }

        public void Dispose()
        {
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // Settings ▸ Display ▸ Shaders ▸ the console, and A on the shader's row, as the player uses one.
        private static void Choose(MainWindow window, string console, string stored)
        {
            SettingsDesk.ShaderPage(window, console);
            SettingsDesk.Choose(window, "use:" + stored);
            Dispatcher.UIThread.RunJobs();
        }

        // Display ▸ Shaders names each console's shader on its row and opens its page, where a shader is used - see EmuSen_BigPicture.md §52.9.
        [Fact]
        public Task Every_console_s_row_names_its_shader_and_opens_its_page() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter, "slang:crt/crt-royale.slangp");
            config.Save();
            MainWindow window = SettingsDesk.Open(SettingsScreen.Display, "shaders");

            foreach (string console in EmuSen.Mistress.Library.RuntimeCatalog.Systems.Select(s => s.Console).Distinct())
                Assert.Equal(console == "SNES" ? "crt-royale (RetroArch, crt)" : ScreenFilters.None, SettingsDesk.Value(window, "shader:" + console));

            SettingsDesk.Choose(window, "shader:NES");
            Assert.Contains("use:Scanlines", SettingsDesk.Keys(window));
            SettingsDesk.Choose(window, "use:Scanlines");
            Assert.Equal("Scanlines", GraphicsConfig.Load().Value("NES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter));
            Assert.Equal("● In use", SettingsDesk.Value(window, "use:Scanlines"));
            SettingsDesk.Back(window);
            Assert.Equal("Scanlines", SettingsDesk.Value(window, "shader:NES"));
            window.Close();
        }, default);

        [Fact]
        public Task A_game_starts_with_its_console_s_filter_and_a_change_reaches_it_at_once() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter, "Simple CRT");
            config.SetValue("NES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter, "Scanlines");
            config.Save();

            var window = new MainWindow();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Equal(ShaderEffect.Crt, frame.ActiveEffect);

            Choose(window, "NES", ScreenFilters.None);
            Assert.Equal(ShaderEffect.Crt, frame.ActiveEffect);

            Choose(window, "SNES", ScreenFilters.None);
            Assert.Equal(ShaderEffect.None, frame.ActiveEffect);
            window.CloseSettings();
            window.Close();
        }, default);

        // A pack laid out as libretro's is, with a stamp, under the sandbox's home.
        private static string FakePack()
        {
            string pack = EmuSen.Mistress.Library.SlangPackDownload.DefaultDirectory;
            foreach (string preset in new[] { "crt/crt-royale.slangp", "crt/crt-lottes.slangp", "handheld/lcd-grid-v2.slangp" })
            {
                Directory.CreateDirectory(Path.GetDirectoryName(Path.Combine(pack, preset))!);
                File.WriteAllText(Path.Combine(pack, preset), "shaders = 0\n");
            }
            File.WriteAllText(Path.Combine(pack, EmuSen.Mistress.Library.SlangPackDownload.StampFile), "2026-09-22 02:00 UTC");
            return pack;
        }

        [Fact]
        public Task A_game_draws_its_console_s_preset_from_the_pack_and_one_not_there_is_said_and_drawn_plain() => Session.Dispatch(() =>
        {
            string pack = FakePack();
            var config = new GraphicsConfig();
            config.SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter, "slang:crt/crt-royale.slangp");
            config.Save();

            var window = new MainWindow();
            window.Show();
            MethodInfo load = typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!;
            load.Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Equal(Path.GetFullPath(Path.Combine(pack, "crt/crt-royale.slangp")), frame.ActiveSlangPreset);
            Assert.Null(frame.ActiveFilter);

            var own = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            own.SetValue("SNES", EmuSen.Mistress.Library.GraphicsKeys.ScreenFilter, "slang:crt/gone.slangp");
            typeof(MainWindow).GetMethod("ApplyScreenFilter", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, new object[] { "SNES" });
            Assert.Null(frame.ActiveSlangPreset);
            Assert.Contains("crt/gone.slangp", window.GetControl<TextBlock>("StatusText").Text);
            window.Close();
        }, default);

        private static byte[] PackZip()
        {
            using var memory = new MemoryStream();
            using (var zip = new System.IO.Compression.ZipArchive(memory, System.IO.Compression.ZipArchiveMode.Create, leaveOpen: true))
            {
                foreach (string name in new[] { "crt/crt-new.slangp", "crt/shaders/new.slang", "README.md" })
                {
                    using var writer = new StreamWriter(zip.CreateEntry(name).Open());
                    writer.Write("x");
                }
            }
            return memory.ToArray();
        }

        [Fact]
        public async Task The_pack_downloads_from_libretro_s_address_and_replaces_the_one_there()
        {
            string pack = FakePack();
            var server = new OnlineCoverTests.FakeServer
            {
                Answer = url =>
                {
                    var content = new ByteArrayContent(PackZip());
                    content.Headers.LastModified = new DateTimeOffset(2026, 9, 23, 2, 0, 0, TimeSpan.Zero);
                    return url == EmuSen.Mistress.Library.SlangPackDownload.PackAddress
                        ? new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.OK) { Content = content }
                        : new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.NotFound);
                },
            };
            using var http = new System.Net.Http.HttpClient(server);
            string built = await EmuSen.Mistress.Library.SlangPackDownload.FetchAsync(http, pack);

            Assert.Equal("2026-09-23 02:00 UTC", built);
            Assert.Equal(new[] { "crt/crt-new.slangp" }, EmuSen.Mistress.Library.SlangPackDownload.Presets(pack));
            Assert.Equal(built, EmuSen.Mistress.Library.SlangPackDownload.Installed(pack));
            Assert.False(File.Exists(Path.Combine(pack, EmuSen.Mistress.Library.SlangPackDownload.StampFile)));
            using (var records = EmuSen.Mistress.Library.FileRecords.Load()) Assert.Equal("written", records.PackSource(pack));
            Assert.Equal(new[] { pack }, Directory.GetDirectories(Path.GetDirectoryName(pack)!));
            Assert.Empty(Directory.GetFiles(Path.GetDirectoryName(pack)!));
        }

        [Theory]
        [InlineData(false)]
        [InlineData(true)]
        public async Task A_failed_or_empty_download_leaves_the_old_pack(bool empty)
        {
            string pack = FakePack();
            var server = new OnlineCoverTests.FakeServer
            {
                Answer = _ => empty
                    ? new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.OK) { Content = new ByteArrayContent(EmptyZip()) }
                    : new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.InternalServerError),
            };
            using var http = new System.Net.Http.HttpClient(server);
            await Assert.ThrowsAnyAsync<Exception>(() => EmuSen.Mistress.Library.SlangPackDownload.FetchAsync(http, pack));
            Assert.Equal(3, EmuSen.Mistress.Library.SlangPackDownload.Presets(pack).Length);
            Assert.Equal(new[] { pack }, Directory.GetDirectories(Path.GetDirectoryName(pack)!));
            Assert.Empty(Directory.GetFiles(Path.GetDirectoryName(pack)!));
        }

        private static byte[] EmptyZip()
        {
            using var memory = new MemoryStream();
            using (var zip = new System.IO.Compression.ZipArchive(memory, System.IO.Compression.ZipArchiveMode.Create, leaveOpen: true))
                using (var writer = new StreamWriter(zip.CreateEntry("README.md").Open())) writer.Write("x");
            return memory.ToArray();
        }

        [Theory]
        [InlineData(null)]
        [InlineData("A filter from a newer build")]
        public void An_unknown_filter_name_draws_no_filter(string? name)
        {
            Assert.Equal(ShaderEffect.None, ScreenFilters.ByName(name));
        }
    }
}
