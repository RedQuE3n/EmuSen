using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Threading;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.Serenity.Slang;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // Settings ▸ Display ▸ Shaders: a page per console of its built-in filters, the pack's folders and the recent ones, a search, the pack's download, and the parameters of the shader in use - see EmuSen_BigPicture.md §52.8.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShaderSettingsTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ShaderSettingsTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShaderSettingsTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public ShaderSettingsTests(ITestOutputHelper output)
        {
            _out = output;
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
            EmuSen.WiseMan.Mistress.Scraping.NoNetwork.HttpFactory.SetValue(null, (Func<System.Net.Http.HttpClient>)(() => new System.Net.Http.HttpClient(new OnlineCoverTests.FakeServer())));
            ConfigStore.OverrideDirectory = null;
            DataStore.OverrideDirectory = null;
            try { if (Directory.Exists(_root)) Directory.Delete(_root, recursive: true); } catch { }
        }

        // A pack laid out as libretro's is, with one real pass whose parameters the preset overrides in part.
        internal static string FakePack(bool stamped = true)
        {
            string pack = SlangPackDownload.DefaultDirectory;
            void Write(string relative, string text)
            {
                Directory.CreateDirectory(Path.GetDirectoryName(Path.Combine(pack, relative))!);
                File.WriteAllText(Path.Combine(pack, relative), text);
            }
            Write("crt/shaders/glow.slang", """
                #version 450
                #pragma parameter GLOW_HEAD "--- Glow ---" 0.0 0.0 0.0 1.0
                #pragma parameter GLOW "Glow strength" 0.25 0.0 1.0 0.05
                #pragma parameter SCAN "Scanline weight" 3.0 1.0 8.0 1.0
                #pragma stage vertex
                void main() { gl_Position = vec4(0.0); }
                #pragma stage fragment
                void main() { }
                """);
            Write("crt/crt-royale-kurozumi.slangp", "shaders = 1\nshader0 = shaders/glow.slang\nparameters = \"SCAN\"\nSCAN = 5.0\n");
            Write("crt/crt-lottes.slangp", "shaders = 1\nshader0 = shaders/glow.slang\n");
            Write("handheld/lcd-grid-v2.slangp", "shaders = 1\nshader0 = ../crt/shaders/glow.slang\n");
            Write("bezel/Mega_Bezel/Presets/Base_CRT_Presets/MBZ__0__SMOOTH-ADV__GDV.slangp", "shaders = 1\nshader0 = ../../../../crt/shaders/glow.slang\n");
            if (stamped) File.WriteAllText(Path.Combine(pack, SlangPackDownload.StampFile), "2026-09-22 02:00 UTC");
            return pack;
        }

        private static string? Stored(string console) => GraphicsConfig.Load().Value(console, GraphicsKeys.ScreenFilter);

        // A console's shader page in Settings' desktop form.
        private static MainWindow Page(string console) => SettingsDesk.Open(SettingsScreen.Display, "shaders", "shader:" + console);

        private static void Use(MainWindow window, string stored) => SettingsDesk.Choose(window, "use:" + stored);

        // The shader in use's parameters page, from its console's page.
        private static void Parameters(MainWindow window)
        {
            SettingsDesk.Choose(window, "params");
            Assert.Equal("Parameters", window.SettingsShown!.Trail[^1]);
        }

        private static string Key(SlangParameter p) => "p:" + p.Id;

        [Fact]
        public Task Each_console_s_page_lists_its_built_in_filters_None_first_and_marks_the_one_in_use() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("NES", GraphicsKeys.ScreenFilter, "Scanlines");
            config.Save();
            MainWindow window = SettingsDesk.Open(SettingsScreen.Display, "shaders");

            foreach (string console in RuntimeCatalog.Systems.Select(s => s.Console).Distinct())
            {
                SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:" + console);
                List<string> uses = SettingsDesk.Keys(window).Where(k => k.StartsWith("use:", StringComparison.Ordinal)).ToList();
                Assert.Equal(ScreenFilters.NamesFor(RuntimeCatalog.HostOf(console)).Select(n => "use:" + n), uses);
                Assert.Equal(ScreenFilters.None, uses[0]["use:".Length..]);
                string inUse = console == "NES" ? "Scanlines" : ScreenFilters.None;
                Assert.Equal(["use:" + inUse], uses.Where(k => SettingsDesk.Value(window, k) == "● In use"));
            }
            window.Close();
        }, default);

        [Fact]
        public Task Use_stores_the_shader_for_that_console_alone_and_moves_the_mark() => Session.Dispatch(() =>
        {
            MainWindow window = Page("SNES");
            Assert.Null(Stored("SNES"));
            Use(window, "Scanlines");
            Assert.Equal("Scanlines", Stored("SNES"));
            Assert.Null(Stored("NES"));
            Assert.Equal("● In use", SettingsDesk.Value(window, "use:Scanlines"));
            Assert.Null(SettingsDesk.Value(window, "use:" + ScreenFilters.None));
            window.Close();
        }, default);

        [Fact]
        public Task Presets_are_grouped_by_folder_named_for_reading_and_found_by_any_word_of_name_folder_or_path() => Session.Dispatch(() =>
        {
            FakePack();
            MainWindow window = Page("SNES");
            Assert.Equal(["use:None", "use:CRT", "use:CRT (Lottes)", "use:Scanlines", "use:Simple CRT"], SettingsDesk.Keys(window).Where(k => k.StartsWith("use:", StringComparison.Ordinal)));
            Assert.Equal(["folder:bezel", "folder:crt", "folder:handheld"], SettingsDesk.Keys(window).Where(k => k.StartsWith("folder:", StringComparison.Ordinal)));

            SettingsDesk.Choose(window, "folder:bezel");
            ShaderEntry bezel = ShaderCatalog.ForPreset("bezel/Mega_Bezel/Presets/Base_CRT_Presets/MBZ__0__SMOOTH-ADV__GDV.slangp");
            Assert.Equal("MBZ 0 SMOOTH-ADV GDV", SettingsDesk.Row(window, "use:" + bezel.Stored).Label);
            Assert.Equal("bezel / Mega Bezel / Presets / Base CRT Presets", SettingsDesk.Row(window, "use:" + bezel.Stored).Sentence);
            SettingsDesk.Back(window);

            foreach ((string words, string found) in new[] { ("royale kuro", "slang:crt/crt-royale-kurozumi.slangp"), ("mega gdv", bezel.Stored), ("handheld/lcd", "slang:handheld/lcd-grid-v2.slangp") })
            {
                SettingsDesk.Choose(window, "find");
                SettingsDesk.Answer(window, words);
                Assert.Equal(["use:" + ScreenFilters.None, "use:" + found], SettingsDesk.Keys(window));
                SettingsDesk.Back(window);
            }

            SettingsDesk.Choose(window, "folder:crt");
            Assert.Equal(["use:slang:crt/crt-lottes.slangp", "use:slang:crt/crt-royale-kurozumi.slangp"], SettingsDesk.Keys(window));
            Use(window, "slang:crt/crt-royale-kurozumi.slangp");
            Assert.Equal("slang:crt/crt-royale-kurozumi.slangp", Stored("SNES"));
            Assert.Equal("crt-royale-kurozumi (RetroArch, crt)", ShaderCatalog.Describe(Stored("SNES")));
            window.Close();
        }, default);

        [Fact]
        public Task The_shaders_used_last_head_the_page_newest_first_and_step_aside_for_a_search() => Session.Dispatch(() =>
        {
            FakePack();
            MainWindow window = Page("SNES");
            foreach ((string folder, string stored) in new[] { ("folder:crt", "slang:crt/crt-lottes.slangp"), ("", "Scanlines"), ("folder:handheld", "slang:handheld/lcd-grid-v2.slangp"), ("", ScreenFilters.None) })
            {
                if (folder.Length > 0) SettingsDesk.Choose(window, folder);
                Use(window, stored);
                if (folder.Length > 0) SettingsDesk.Back(window);
            }

            string[] recent = SettingsDesk.Keys(window).Where(k => k.StartsWith("recent:", StringComparison.Ordinal)).Select(k => k["recent:".Length..]).ToArray();
            Assert.Equal(["slang:handheld/lcd-grid-v2.slangp", "Scanlines", "slang:crt/crt-lottes.slangp"], recent);
            Assert.All(recent, r => Assert.Equal(ShaderCatalog.RecentGroup, SettingsDesk.Row(window, "recent:" + r).Sentence));
            Assert.Equal(recent, GraphicsConfig.Load().RecentFor("SNES"));
            Assert.Empty(GraphicsConfig.Load().RecentFor("NES"));

            SettingsDesk.Choose(window, "find");
            SettingsDesk.Answer(window, "lcd");
            Assert.DoesNotContain(SettingsDesk.Keys(window), k => k.StartsWith("recent:", StringComparison.Ordinal));
            window.Close();
        }, default);

        [Fact]
        public Task Without_a_pack_the_download_row_fetches_it_and_every_console_s_page_offers_its_presets() => Session.Dispatch(() =>
        {
            var server = new OnlineCoverTests.FakeServer
            {
                Answer = url =>
                {
                    using var memory = new MemoryStream();
                    using (var zip = new System.IO.Compression.ZipArchive(memory, System.IO.Compression.ZipArchiveMode.Create, leaveOpen: true))
                        using (var writer = new StreamWriter(zip.CreateEntry("crt/crt-new.slangp").Open())) writer.Write("shaders = 0\n");
                    return url == SlangPackDownload.PackAddress
                        ? new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.OK) { Content = new System.Net.Http.ByteArrayContent(memory.ToArray()) }
                        : new System.Net.Http.HttpResponseMessage(System.Net.HttpStatusCode.NotFound);
                },
            };
            EmuSen.WiseMan.Mistress.Scraping.NoNetwork.HttpFactory.SetValue(null, (Func<System.Net.Http.HttpClient>)(() => new System.Net.Http.HttpClient(server)));
            MainWindow window = SettingsDesk.Open(SettingsScreen.Display, "shaders");
            Assert.Equal("Not downloaded", SettingsDesk.Value(window, "pack"));
            SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:SNES");
            Assert.DoesNotContain(SettingsDesk.Keys(window), k => k.StartsWith("folder:", StringComparison.Ordinal));

            SettingsDesk.At(window, SettingsScreen.Display, "shaders");
            SettingsDesk.Choose(window, "pack");
            ShaderShelf shelf = window.SettingsShown!.Host.Shaders;
            for (int i = 0; i < 5000 && shelf.Downloading is { IsCompleted: false }; i++) { Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(1); }
            Dispatcher.UIThread.RunJobs();
            Assert.Null(shelf.Said);
            Assert.Equal("1 preset", SettingsDesk.Value(window, "pack"));

            foreach (string console in new[] { "SNES", "GB" })
            {
                SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:" + console, "folder:crt");
                Assert.Contains("use:slang:crt/crt-new.slangp", SettingsDesk.Keys(window));
            }
            window.Close();
        }, default);

        [Fact]
        public Task A_built_in_filter_s_parameters_are_saved_per_console_and_per_shader_and_reset_one_or_all() => Session.Dispatch(() =>
        {
            MainWindow window = Page("SNES");
            Use(window, CrtFilters.Lottes.Name);
            Parameters(window);

            Assert.Equal(CrtFilters.LottesParameters.Select(p => p.Description), SettingsDesk.Layout(window).Rows.Where(r => r.Key.StartsWith("p:", StringComparison.Ordinal)).Select(r => r.Label));
            SlangParameter boost = CrtFilters.LottesParameters.Single(p => p.Description == "Brightness boost");
            Assert.Equal("1", SettingsDesk.Value(window, Key(boost)));
            Assert.Equal("0.041", SettingsDesk.Value(window, Key(CrtFilters.LottesParameters.Single(p => p.Description == "Curvature, vertical"))));

            SettingsDesk.Step(window, Key(boost), 1);
            Assert.Equal("1.05", GraphicsConfig.Load().ParametersFor("SNES", CrtFilters.Lottes.Name)["brightBoost"]);
            SettingsDesk.Step(window, Key(CrtFilters.LottesParameters[0]), -1);
            Assert.Equal(2, GraphicsConfig.Load().ParametersFor("SNES", CrtFilters.Lottes.Name).Count);
            Assert.Empty(GraphicsConfig.Load().ParametersFor("NES", CrtFilters.Lottes.Name));

            SettingsDesk.Step(window, Key(boost), -1);
            Assert.False(GraphicsConfig.Load().ParametersFor("SNES", CrtFilters.Lottes.Name).ContainsKey("brightBoost"));

            SettingsDesk.Choose(window, "paramreset");
            Assert.Empty(GraphicsConfig.Load().ShaderParameters);
            Assert.All(CrtFilters.LottesParameters.Where(p => !SlangParameters.IsHeading(p)), p => Assert.Equal(((double)(decimal)p.Initial).ToString("0.####", System.Globalization.CultureInfo.InvariantCulture), SettingsDesk.Value(window, Key(p))));
            window.Close();
        }, default);

        // The CRT's settings that are choices read as names, step from one to the next, and are stored as their numbers - see EmuSen_Settings_Reference.md §4.40.
        [Fact]
        public Task The_crt_s_choices_are_shown_by_name_and_stored_by_number() => Session.Dispatch(() =>
        {
            MainWindow window = Page("SNES");
            Use(window, CrtFilter.Name);
            Parameters(window);

            Assert.Equal(CrtFilter.Parameters.Select(p => p.Description), SettingsDesk.Layout(window).Rows.Where(r => r.Key.StartsWith("p:", StringComparison.Ordinal)).Select(r => r.Label));
            string Shown(string label) => SettingsDesk.Value(window, Key(CrtFilter.Parameters.Single(p => p.Description == label)))!;
            Assert.Equal("Balanced", Shown("Quality"));
            Assert.Equal("Composite", Shown("Signal"));
            Assert.Equal("Consumer TV, 20-inch", Shown("Screen"));
            Assert.Equal("Bright", Shown("Picture brightness"));

            SettingsDesk.Step(window, Key(CrtFilter.Parameters.Single(p => p.Description == "Quality")), 1);
            Assert.Equal("2", GraphicsConfig.Load().ParametersFor("SNES", CrtFilter.Name)["quality"]);
            Assert.Equal("Accurate", Shown("Quality"));
            Assert.Contains("Balanced is the shader's own", SettingsDesk.Row(window, Key(CrtFilter.Parameters.Single(p => p.Description == "Quality"))).Sentence);
            window.Close();
        }, default);

        [Fact]
        public Task A_preset_s_parameters_take_its_own_values_as_their_defaults_and_its_headings_as_headings() => Session.Dispatch(() =>
        {
            FakePack();
            MainWindow window = Page("SNES");
            SettingsDesk.Choose(window, "folder:crt");
            Use(window, "slang:crt/crt-royale-kurozumi.slangp");
            SettingsDesk.Back(window);
            Parameters(window);
            Assert.Equal([("Glow strength", "0.25"), ("Scanline weight", "5")], SettingsDesk.Layout(window).Rows.Where(r => r.Key.StartsWith("p:", StringComparison.Ordinal)).Select(r => (r.Label, r.Status)));
            Assert.False(SettingsDesk.Row(window, "h:GLOW_HEAD").IsEnabled);
            Assert.Equal("--- Glow ---", SettingsDesk.Row(window, "h:GLOW_HEAD").Label);

            SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:SNES", "folder:crt");
            Use(window, "slang:crt/crt-lottes.slangp");
            SettingsDesk.Back(window);
            Parameters(window);
            Assert.Equal("3", SettingsDesk.Value(window, "p:SCAN"));
            window.Close();
        }, default);

        // The whole path, from a row in Settings to the uniform the running game's filter draws with - see EmuSen_Settings_Reference.md §4.48.3.
        [Fact]
        public Task A_parameter_stepped_while_a_game_runs_reaches_the_filter_chain_it_draws_with() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsKeys.ScreenFilter, CrtFilters.Lottes.Name);
            config.SetParameter("SNES", CrtFilters.Lottes.Name, "maskDark", "0.8");
            config.Save();

            var window = new MainWindow { Width = 400, Height = 300 };
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            frame.Width = frame.Height = 24;
            frame.WaitForFilter();
            frame.UpdateFrame(new byte[16 * 16 * 4], 16, 16);
            UiTest.Capture(window);
            if (!EmuSen.WiseMan.Serenity.ChainDevice.Missing) Assert.Equal(0.8f, Held(frame, "maskDark"));
            if (!EmuSen.WiseMan.Serenity.ChainDevice.Missing) Assert.Equal(1f, Held(frame, "brightBoost"));

            // Display ▸ Shaders opens on the running game's console.
            window.ShowSettings(SettingsScreen.Display, "shaders");
            Assert.Equal(3, window.SettingsShown!.Trail.Count);
            Assert.Equal("Shaders", window.SettingsShown.Trail[1]);
            Assert.Equal("● In use", SettingsDesk.Value(window, "use:" + CrtFilters.Lottes.Name));
            Parameters(window);
            SettingsDesk.Step(window, "p:brightBoost", -1);
            UiTest.Capture(window);
            if (!EmuSen.WiseMan.Serenity.ChainDevice.Missing) Assert.Equal(0.95f, Held(frame, "brightBoost"), 4);
            if (!EmuSen.WiseMan.Serenity.ChainDevice.Missing) Assert.Equal(0.8f, Held(frame, "maskDark"));

            // Another console's shader and parameters change nothing the running game draws with.
            SettingsDesk.ShaderPage(window, "NES");
            Use(window, CrtFilters.Lottes.Name);
            Parameters(window);
            SettingsDesk.Step(window, "p:brightBoost", 1);
            Assert.Equal("1.05", GraphicsConfig.Load().ParametersFor("NES", CrtFilters.Lottes.Name)["brightBoost"]);
            UiTest.Capture(window);
            if (!EmuSen.WiseMan.Serenity.ChainDevice.Missing) Assert.Equal(0.95f, Held(frame, "brightBoost"), 4);
            window.CloseSettings();
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            window.Close();
        }, default);

        // The pages at the desktop window's size, a real pack where one is named, with a filter's parameters and a preset's shown; EMUSEN_UI_DUMP keeps the pictures.
        [Fact]
        public Task The_pages_lay_out_at_desktop_size_with_a_filter_s_parameters_and_a_preset_s() => Session.Dispatch(() =>
        {
            if (string.IsNullOrEmpty(Environment.GetEnvironmentVariable(EmuSen.WiseMan.Serenity.SlangPresetTests.PackVariable))) FakePack();
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsKeys.ScreenFilter, "slang:crt/crt-lottes.slangp");
            config.NoteRecent("SNES", "slang:crt/crt-lottes.slangp", ShaderCatalog.RecentKept);
            config.NoteRecent("SNES", CrtFilters.Lottes.Name, ShaderCatalog.RecentKept);
            config.SetParameter("SNES", CrtFilters.Lottes.Name, "maskDark", "0.8");
            config.Save();
            MainWindow window = Page("SNES");
            UiTest.AssertLaidOut(SettingsDesk.Of(window), "shaders-desktop-page");
            Parameters(window);
            UiTest.AssertLaidOut(SettingsDesk.Of(window), "shaders-desktop-preset");
            SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:SNES");
            Use(window, CrtFilters.Lottes.Name);
            Parameters(window);
            UiTest.AssertLaidOut(SettingsDesk.Of(window), "shaders-desktop-lottes");
            SettingsDesk.At(window, SettingsScreen.Display, "shaders", "shader:SNES");
            SettingsDesk.Choose(window, "find");
            SettingsDesk.Answer(window, "royale");
            UiTest.AssertLaidOut(SettingsDesk.Of(window), "shaders-desktop-search");
            window.Close();
        }, default);

        // Where a real pack is named, how long its biggest presets' parameters take to read and show - see EmuSen_Settings_Reference.md §4.48.2.
        [Fact]
        public Task A_pack_preset_with_hundreds_of_parameters_is_shown_in_measured_time() => Session.Dispatch(() =>
        {
            string? pack = Environment.GetEnvironmentVariable(EmuSen.WiseMan.Serenity.SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;
            var shelf = new ShaderShelf(pack);
            _out.WriteLine($"{shelf.Presets.Count} presets");
            foreach (string relative in new[] { "crt/crt-royale.slangp", "bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV.slangp" })
            {
                var clock = System.Diagnostics.Stopwatch.StartNew();
                int count = shelf.ParametersOf(ShaderCatalog.SlangPrefix + relative).Count;
                _out.WriteLine($"{relative}: {count} parameters read in {clock.ElapsedMilliseconds} ms");
                Assert.True(count > 0);
            }
        }, default);

        private static float Held(GameFrameControl control, string id) =>
            (float)typeof(GameFrameControl).GetMethod("FilterParameter", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(control, new object[] { id })!;
    }
}
