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

        private static ShaderPanel Choose(ShaderSettingsWindow window, string console, string stored)
        {
            ShaderPanel panel = window.PanelFor(console);
            panel.List.SelectedIndex = panel.List.Models.ToList().FindIndex(e => e.Stored == stored && !e.Recent);
            Dispatcher.UIThread.RunJobs();
            panel.Use();
            Dispatcher.UIThread.RunJobs();
            return panel;
        }

        // The row that replaced the dropdown: every tab names its console's shader and opens the Shaders window on that console - see EmuSen_Settings_Reference.md §4.48.4.
        [Fact]
        public Task Every_console_s_tab_names_its_shader_and_opens_the_shaders_window_on_that_console() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, "slang:crt/crt-royale.slangp");
            string? told = null;
            var window = new GraphicsSettingsWindow(config, console => told = console, null);
            window.Show();

            foreach (string console in CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console))
            {
                Assert.Single(window.GetLogicalDescendants().OfType<Button>().Where(b => b.Name == $"{console}.Shaders").Distinct());
                string expected = console == "SNES" ? "crt-royale (RetroArch, crt)" : ScreenFilters.None;
                Assert.Equal(expected, window.GetLogicalDescendants().OfType<TextBlock>().Distinct().Single(t => t.Name == $"{console}.ShaderInUse").Text);
            }

            window.GetLogicalDescendants().OfType<Button>().Distinct().Single(b => b.Name == "NES.Shaders").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            ShaderSettingsWindow shaders = Assert.IsType<ShaderSettingsWindow>(window.ShaderWindow);
            Assert.Equal("NES", (shaders.GetLogicalDescendants().OfType<Tabs>().Single().SelectedItem as TabItem)?.Header);
            Choose(shaders, "NES", "Scanlines");
            Assert.Equal("NES", told);
            Assert.Equal("Scanlines", GraphicsConfig.Load().Value("NES", GraphicsSettingsWindow.ScreenFilterKey));
            shaders.Close();
            Dispatcher.UIThread.RunJobs();
            Assert.Equal("Scanlines", window.GetLogicalDescendants().OfType<TextBlock>().Distinct().Single(t => t.Name == "NES.ShaderInUse").Text);
            window.Close();
        }, default);

        [Fact]
        public Task A_game_starts_with_its_console_s_filter_and_a_change_reaches_it_at_once() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, "Simple CRT");
            config.SetValue("NES", GraphicsSettingsWindow.ScreenFilterKey, "Scanlines");
            config.Save();

            var window = new MainWindow();
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Equal(ShaderEffect.Crt, frame.ActiveEffect);

            typeof(MainWindow).GetMethod("ShowShaderSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            ShaderSettingsWindow settings = window.OwnedWindows.OfType<ShaderSettingsWindow>().Single();
            Choose(settings, "NES", ScreenFilters.None);
            Assert.Equal(ShaderEffect.Crt, frame.ActiveEffect);

            Choose(settings, "SNES", ScreenFilters.None);
            Assert.Equal(ShaderEffect.None, frame.ActiveEffect);
            settings.Close();
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
            config.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, "slang:crt/crt-royale.slangp");
            config.Save();

            var window = new MainWindow();
            window.Show();
            MethodInfo load = typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!;
            load.Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            Assert.Equal(Path.GetFullPath(Path.Combine(pack, "crt/crt-royale.slangp")), frame.ActiveSlangPreset);
            Assert.Null(frame.ActiveFilter);

            var own = (GraphicsConfig)typeof(MainWindow).GetField("_graphics", BindingFlags.Instance | BindingFlags.NonPublic)!.GetValue(window)!;
            own.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, "slang:crt/gone.slangp");
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
