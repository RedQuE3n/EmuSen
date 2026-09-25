using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Avalonia.Threading;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using EmuSen.Serenity;
using EmuSen.Serenity.Shaders;
using EmuSen.WiseMan.Fixtures;
using Xunit.Abstractions;

namespace EmuSen.WiseMan.Mistress
{
    // The Shaders window: one list of built-in filters and pack presets per console, and the shown one's parameters as sliders - see EmuSen_Settings_Reference.md §4.48.
    [Collection(TestCollections.ProcessGlobals)]
    public class ShaderSettingsWindowTests : IDisposable
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ShaderSettingsWindowTests).GetTypeInfo().Assembly);

        private readonly ITestOutputHelper _out;
        private readonly string _root = Path.Combine(Path.GetTempPath(), "EmuSenShaderWindowTests", Guid.NewGuid().ToString("N"));
        private readonly string _rom;

        public ShaderSettingsWindowTests(ITestOutputHelper output)
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

        private static void Pump(ShaderPanel panel)
        {
            for (int i = 0; i < 2000 && !panel.Reading.IsCompleted; i++) { Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(1); }
            Dispatcher.UIThread.RunJobs();
            Assert.True(panel.Reading.IsCompleted);
        }

        private static void ChooseRow(ShaderPanel panel, Func<ShaderEntry, bool> which)
        {
            panel.List.SelectedIndex = panel.List.Models.ToList().FindIndex(e => which(e));
            Dispatcher.UIThread.RunJobs();
            Pump(panel);
        }

        private static string? Stored(string console) => GraphicsConfig.Load().Value(console, GraphicsSettingsWindow.ScreenFilterKey);

        [Fact]
        public Task Each_console_s_tab_lists_its_built_in_filters_None_first_and_marks_the_one_in_use() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("NES", GraphicsSettingsWindow.ScreenFilterKey, "Scanlines");
            var window = new ShaderSettingsWindow(config, null, "N64");
            window.Show();

            Assert.Equal("N64", ((window.GetLogicalDescendants().OfType<Tabs>().Single().SelectedItem as TabItem)?.Header));
            foreach (string console in CoreCatalog.ConsolesInReleaseOrder.Select(c => c.Console))
            {
                ShaderPanel panel = window.PanelFor(console);
                Assert.Equal(ScreenFilters.NamesFor(console), panel.List.Models.Select(e => e.Stored));
                Assert.All(panel.List.Models, e => Assert.Equal(ShaderCatalog.BuiltIn, e.Group));
                string inUse = console == "NES" ? "Scanlines" : ScreenFilters.None;
                Assert.Equal(inUse, panel.List.Selected?.Stored);
                Assert.Equal(inUse, panel.Shown?.Stored);
            }
            window.Close();
        }, default);

        [Fact]
        public Task Use_stores_the_shown_shader_for_that_console_alone_tells_the_frontend_and_moves_the_mark() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            var told = new List<string>();
            var window = new ShaderSettingsWindow(config, told.Add, "SNES");
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");

            ChooseRow(snes, e => e.Stored == "Scanlines");
            Assert.Empty(told);
            Assert.Null(Stored("SNES"));

            snes.List.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = Key.Enter });
            Assert.Equal(new[] { "SNES" }, told);
            Assert.Equal("Scanlines", Stored("SNES"));
            Assert.Null(Stored("NES"));
            Assert.Equal("In Use", snes.GetLogicalDescendants().OfType<Button>().Single(b => b.Name == "SNES.UseShader").Content);
            window.Close();
        }, default);

        [Fact]
        public Task Presets_are_grouped_by_folder_named_for_reading_and_found_by_any_word_of_name_folder_or_path() => Session.Dispatch(() =>
        {
            FakePack();
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, "SNES");
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");

            ShaderEntry bezel = snes.List.Models.Single(e => e.IsPreset && e.Relative!.StartsWith("bezel/"));
            Assert.Equal("MBZ 0 SMOOTH-ADV GDV", bezel.Name);
            Assert.Equal("bezel / Mega Bezel / Presets / Base CRT Presets", bezel.Group);
            Assert.Equal(new[] { "None", "CRT (Lottes)", "Scanlines", "Simple CRT" }, snes.List.Models.Where(e => !e.IsPreset).Select(e => e.Stored));
            var groups = snes.List.Models.Select(e => e.Group).ToList();
            Assert.Equal(groups.Distinct().Count(), groups.Where((g, i) => i == 0 || groups[i - 1] != g).Count());

            Search(snes, "royale kuro");
            Assert.Equal(new[] { ScreenFilters.None, "slang:crt/crt-royale-kurozumi.slangp" }, snes.List.Models.Select(e => e.Stored));

            Search(snes, "mega gdv");
            Assert.Equal(new[] { ScreenFilters.None, bezel.Stored }, snes.List.Models.Select(e => e.Stored));

            Search(snes, "handheld/lcd");
            Assert.Equal(new[] { ScreenFilters.None, "slang:handheld/lcd-grid-v2.slangp" }, snes.List.Models.Select(e => e.Stored));

            Search(snes, "");
            Dropdown category = snes.Filter.GetVisualDescendants().OfType<Dropdown>().Single();
            Assert.Equal(new[] { ShaderCatalog.AllCategories, ShaderCatalog.BuiltIn, "bezel", "crt", "handheld" }, category.Items.Cast<object>().Select(o => o.ToString()));
            category.SelectedItem = "crt";
            Dispatcher.UIThread.RunJobs();
            Assert.Equal(new[] { ScreenFilters.None, "slang:crt/crt-lottes.slangp", "slang:crt/crt-royale-kurozumi.slangp" }, snes.List.Models.Select(e => e.Stored));

            ChooseRow(snes, e => e.Relative == "crt/crt-royale-kurozumi.slangp");
            TextBlock path = snes.GetLogicalDescendants().OfType<TextBlock>().Single(t => t.Name == "SNES.ShaderPath");
            Assert.StartsWith("crt/crt-royale-kurozumi.slangp\nin ", path.Text);
            Assert.Equal(Avalonia.Media.TextWrapping.Wrap, path.TextWrapping);
            snes.Use();
            Assert.Equal("slang:crt/crt-royale-kurozumi.slangp", Stored("SNES"));
            Assert.Equal("crt-royale-kurozumi (RetroArch, crt)", ShaderSettingsWindow.Describe(Stored("SNES")));
            window.Close();
        }, default);

        [Fact]
        public Task The_shaders_used_last_head_the_list_newest_first_and_step_aside_for_a_search() => Session.Dispatch(() =>
        {
            FakePack();
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, "SNES");
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");
            foreach (string stored in new[] { "slang:crt/crt-lottes.slangp", "Scanlines", "slang:handheld/lcd-grid-v2.slangp", ScreenFilters.None })
            {
                ChooseRow(snes, e => e.Stored == stored && !e.Recent);
                snes.Use();
            }

            var recent = snes.List.Models.TakeWhile(e => e.Recent).ToList();
            Assert.Equal(new[] { "slang:handheld/lcd-grid-v2.slangp", "Scanlines", "slang:crt/crt-lottes.slangp" }, recent.Select(e => e.Stored));
            Assert.All(recent, e => Assert.Equal(ShaderCatalog.RecentGroup, e.Group));
            Assert.Equal(recent.Select(e => e.Stored), GraphicsConfig.Load().RecentFor("SNES"));
            Assert.Empty(GraphicsConfig.Load().RecentFor("NES"));

            Search(snes, "lcd");
            Assert.DoesNotContain(snes.List.Models, e => e.Recent);
            window.Close();
        }, default);

        [Fact]
        public Task Without_a_pack_the_download_stands_where_the_presets_would_and_fills_every_tab() => Session.Dispatch(() =>
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
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, "SNES", () => new System.Net.Http.HttpClient(server));
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");
            Assert.Contains("Not downloaded", snes.GetLogicalDescendants().OfType<TextBlock>().Single(t => t.Name == "SNES.PackStatus").Text);
            Button download = snes.GetLogicalDescendants().OfType<Button>().Single(b => b.Name == "SNES.DownloadPack");
            Assert.Equal("Download Pack", download.Content);
            Assert.DoesNotContain(snes.List.Models, e => e.IsPreset);

            download.RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            for (int i = 0; i < 5000 && window.Downloading is { IsCompleted: false }; i++) { Dispatcher.UIThread.RunJobs(); System.Threading.Thread.Sleep(1); }
            Dispatcher.UIThread.RunJobs();

            foreach (string console in new[] { "SNES", "GB" })
            {
                ShaderPanel panel = window.PanelFor(console);
                Assert.Contains(panel.List.Models, e => e.Stored == "slang:crt/crt-new.slangp");
                Assert.Equal("Update Pack", panel.GetLogicalDescendants().OfType<Button>().Single(b => b.Name == $"{console}.DownloadPack").Content);
            }
            window.Close();
        }, default);

        [Fact]
        public Task A_built_in_filter_s_parameters_are_sliders_saved_per_console_and_per_shader_and_reset_one_or_all() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            var told = new List<string>();
            var window = new ShaderSettingsWindow(config, told.Add, "SNES");
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");
            ChooseRow(snes, e => e.Stored == CrtFilters.Lottes.Name);

            Assert.Equal(CrtFilters.LottesParameters.Select(p => p.Description), snes.Parameters.Select(s => s.Label));
            SliderRow boost = Row(snes, "Brightness boost");
            Assert.Equal((0.0, 2.0, 0.05, 1.0), (boost.Minimum, boost.Maximum, boost.Step, boost.DefaultValue), new ToleranceComparer());

            SliderRow warp = Row(snes, "Curvature, vertical");
            Assert.Contains(warp.GetLogicalDescendants().OfType<TextBlock>(), t => t.Text == "0.041");

            Press(Row(snes, "Brightness boost"), Key.Right);
            Assert.Equal("1.05", GraphicsConfig.Load().ParametersFor("SNES", CrtFilters.Lottes.Name)["brightBoost"]);
            Assert.Empty(told);
            Press(Row(snes, snes.Parameters[0].Label), Key.Left);
            Assert.Equal(2, GraphicsConfig.Load().ParametersFor("SNES", CrtFilters.Lottes.Name).Count);
            Assert.Empty(GraphicsConfig.Load().ParametersFor("NES", CrtFilters.Lottes.Name));

            snes.Use();
            Assert.Equal(new[] { "SNES" }, told);
            Press(Row(snes, "Brightness boost"), Key.Right);
            Assert.Equal(new[] { "SNES", "SNES" }, told);

            Press(Row(snes, "Brightness boost"), Key.Left);
            Press(Row(snes, "Brightness boost"), Key.Left);
            Assert.False(GraphicsConfig.Load().ParametersFor("SNES", CrtFilters.Lottes.Name).ContainsKey("brightBoost"));

            snes.GetLogicalDescendants().OfType<Button>().Single(b => b.Name == "SNES.ResetShader").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Assert.Empty(GraphicsConfig.Load().ShaderParameters);
            Assert.All(snes.Sliders, s => Assert.True(s.IsDefault));
            Assert.All(snes.Parameters, s => Assert.True(s.IsDefault));
            window.Close();
        }, default);

        [Fact]
        public Task A_preset_s_parameters_take_its_own_values_as_their_defaults_and_its_headings_as_headings() => Session.Dispatch(() =>
        {
            FakePack();
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, "SNES");
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");

            ChooseRow(snes, e => e.Relative == "crt/crt-royale-kurozumi.slangp");
            Assert.Equal(new[] { ("Glow strength", 0.25), ("Scanline weight", 5.0) }, snes.Sliders.Select(s => (s.Label, s.DefaultValue)));
            Assert.Contains(snes.GetLogicalDescendants().OfType<SectionHeader>(), h => h.Text == "--- Glow ---");

            ChooseRow(snes, e => e.Relative == "crt/crt-lottes.slangp");
            Assert.Equal(3.0, snes.Sliders.Single(s => s.Label == "Scanline weight").DefaultValue);
            window.Close();
        }, default);

        // The whole path, from a slider in the window to the uniform the running game's filter draws with - see EmuSen_Settings_Reference.md §4.48.3.
        [Fact]
        public Task A_slider_moved_while_a_game_runs_reaches_the_filter_chain_it_draws_with() => Session.Dispatch(() =>
        {
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, CrtFilters.Lottes.Name);
            config.SetParameter("SNES", CrtFilters.Lottes.Name, "maskDark", "0.8");
            config.Save();

            var window = new MainWindow { Width = 400, Height = 300 };
            window.Show();
            typeof(MainWindow).GetMethod("LoadRom", BindingFlags.Instance | BindingFlags.NonPublic, null, new[] { typeof(string), typeof(string) }, null)!
                .Invoke(window, new object[] { _rom, "Game.sfc" });
            GameFrameControl frame = window.GetControl<GameFrameControl>("GameFrame");
            // Small, since a runtime effect drawn on the CPU costs about 150 microseconds a pixel, 18 s for a 400x300 frame.
            frame.Width = frame.Height = 24;
            frame.UpdateFrame(new byte[16 * 16 * 4], 16, 16);
            UiTest.Capture(window);
            Assert.Equal(0.8f, Held(frame, "maskDark"));
            Assert.Equal(1f, Held(frame, "brightBoost"));

            typeof(MainWindow).GetMethod("ShowShaderSettings", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            var shaders = window.OwnedWindows.OfType<ShaderSettingsWindow>().Single();
            ShaderPanel snes = shaders.PanelFor("SNES");
            Assert.Equal(CrtFilters.Lottes.Name, snes.Shown?.Stored);
            Press(Row(snes, "Brightness boost"), Key.Left);
            UiTest.Capture(window);
            Assert.Equal(0.95f, Held(frame, "brightBoost"), 4);
            Assert.Equal(0.8f, Held(frame, "maskDark"));

            ShaderPanel nes = shaders.PanelFor("NES");
            Tabs tabs = shaders.GetLogicalDescendants().OfType<Tabs>().Single();
            tabs.SelectedItem = tabs.Items.OfType<TabItem>().Single(t => (string?)t.Header == "NES");
            ChooseRow(nes, e => e.Stored == CrtFilters.Lottes.Name);
            Press(Row(nes, "Brightness boost"), Key.Right);
            Assert.Equal("1.05", GraphicsConfig.Load().ParametersFor("NES", CrtFilters.Lottes.Name)["brightBoost"]);
            UiTest.Capture(window);
            Assert.Equal(0.95f, Held(frame, "brightBoost"), 4);
            shaders.Close();
            typeof(MainWindow).GetMethod("StopEmulationThread", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(window, null);
            window.Close();
        }, default);

        // The window at its own size on a desktop, a real pack where one is named, with a filter's sliders and a preset's shown; EMUSEN_UI_DUMP keeps the pictures.
        [Fact]
        public Task The_window_lays_out_at_desktop_size_with_a_filter_s_sliders_and_a_preset_s() => Session.Dispatch(() =>
        {
            string? pack = Environment.GetEnvironmentVariable(EmuSen.WiseMan.Serenity.SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) FakePack();
            var config = new GraphicsConfig();
            config.SetValue("SNES", GraphicsSettingsWindow.ScreenFilterKey, "slang:crt/crt-lottes.slangp");
            config.NoteRecent("SNES", "slang:crt/crt-lottes.slangp", ShaderCatalog.RecentKept);
            config.NoteRecent("SNES", CrtFilters.Lottes.Name, ShaderCatalog.RecentKept);
            config.SetParameter("SNES", CrtFilters.Lottes.Name, "maskDark", "0.8");
            var window = new ShaderSettingsWindow(config, null, "SNES", pack: string.IsNullOrEmpty(pack) ? null : pack);
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");
            Pump(snes);
            UiTest.AssertLaidOut(window, "shaders-desktop-preset");

            ChooseRow(snes, e => e.Stored == CrtFilters.Lottes.Name && !e.Recent);
            UiTest.AssertLaidOut(window, "shaders-desktop-lottes");

            ChooseRow(snes, e => e.IsPreset && e.Relative!.StartsWith("bezel/"));
            UiTest.AssertLaidOut(window, "shaders-desktop-bezel");

            Search(snes, "royale");
            UiTest.AssertLaidOut(window, "shaders-desktop-search");
            window.Close();

            var none = new ShaderSettingsWindow(new GraphicsConfig(), null, "GB", pack: Path.Combine(Path.GetTempPath(), "EmuSenNoPack", Guid.NewGuid().ToString("N")));
            none.Show();
            UiTest.AssertLaidOut(none, "shaders-desktop-nopack");
            none.Close();
        }, default);

        // Where a real pack is named, how long its biggest preset's sliders take to build and show - see EmuSen_Settings_Reference.md §4.48.2.
        [Fact]
        public Task A_pack_preset_with_hundreds_of_parameters_is_shown_in_measured_time() => Session.Dispatch(() =>
        {
            string? pack = Environment.GetEnvironmentVariable(EmuSen.WiseMan.Serenity.SlangPresetTests.PackVariable);
            if (string.IsNullOrEmpty(pack)) return;
            var window = new ShaderSettingsWindow(new GraphicsConfig(), null, "SNES", pack: pack);
            window.Show();
            ShaderPanel snes = window.PanelFor("SNES");
            _out.WriteLine($"{snes.List.Models.Count} rows");
            foreach (string relative in new[] { "crt/crt-royale.slangp", "bezel/Mega_Bezel/Presets/MBZ__0__SMOOTH-ADV.slangp" })
            {
                var clock = System.Diagnostics.Stopwatch.StartNew();
                ChooseRow(snes, e => e.Relative == relative);
                long read = clock.ElapsedMilliseconds;
                UiTest.Capture(window);
                _out.WriteLine($"{relative}: {snes.Sliders.Count()} sliders, read and built in {read} ms, drawn by {clock.ElapsedMilliseconds} ms");
                Assert.NotEmpty(snes.Sliders);
            }
            window.Close();
        }, default);

        // Typed into the search box, as a person would; SearchText set from code is not a change.
        internal static void Search(ShaderPanel panel, string text)
        {
            panel.Filter.GetVisualDescendants().OfType<TextBox>().Single(t => t.Name == "PART_Search").Text = text;
            Dispatcher.UIThread.RunJobs();
        }

        // A parameter's row scrolled into view, since the list builds only the rows in view - see EmuSen_Settings_Reference.md §4.48.9.
        internal static SliderRow Row(ShaderPanel panel, string label) =>
            panel.ParameterList.Reveal(panel.Parameters.Single(p => p.Label == label)) ?? throw new InvalidOperationException(label + " has no row after scrolling to it.");

        internal static void Press(SliderRow row, Key key)
        {
            Slider slider = row.GetLogicalDescendants().OfType<Slider>().Single();
            slider.RaiseEvent(new KeyEventArgs { RoutedEvent = InputElement.KeyDownEvent, Key = key, Source = slider });
            Dispatcher.UIThread.RunJobs();
        }

        private static float Held(GameFrameControl control, string id) =>
            (float)typeof(GameFrameControl).GetMethod("FilterParameter", BindingFlags.Instance | BindingFlags.NonPublic)!.Invoke(control, new object[] { id })!;

        private sealed class ToleranceComparer : IEqualityComparer<(double, double, double, double)>
        {
            public bool Equals((double, double, double, double) a, (double, double, double, double) b) =>
                Math.Abs(a.Item1 - b.Item1) < 1e-6 && Math.Abs(a.Item2 - b.Item2) < 1e-6 && Math.Abs(a.Item3 - b.Item3) < 1e-6 && Math.Abs(a.Item4 - b.Item4) < 1e-6;
            public int GetHashCode((double, double, double, double) obj) => 0;
        }
    }
}
