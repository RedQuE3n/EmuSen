using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Media;
using EmuSen.Galaxia;
using EmuSen.LunaP.Settings;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // A user theme really does repaint the kit - see EmuSen_LunaP.md §13.
    public class ThemeTests : IDisposable
    {
        private readonly string _configDir;

        // What LunaP reported, captured through the hook a host normally owns.
        private string? _reported;

        public ThemeTests()
        {
            _configDir = Path.Combine(Path.GetTempPath(), "lunap-theme-" + Guid.NewGuid().ToString("N"));
            LunaSettings.Store = new JsonSettingsStore(_configDir);
            _reported = null;
            LunaSettings.Diagnostics = m => _reported = m;
        }

        // The applied dictionary is global to the headless application, so every test here has to put it back.
        public void Dispose()
        {
            UiTest.Run(() => LunaTheme.Apply(LunaTheme.BuiltIn)).GetAwaiter().GetResult();
            LunaSettings.Store = new JsonSettingsStore(Path.Combine(Path.GetTempPath(), "lunap-unset"));
            if (Directory.Exists(_configDir)) Directory.Delete(_configDir, recursive: true);
        }

        private void WriteTheme(string name, string body)
        {
            Directory.CreateDirectory(LunaTheme.Directory);
            File.WriteAllText(Path.Combine(LunaTheme.Directory, name + LunaTheme.Extension),
                "<ResourceDictionary xmlns=\"https://github.com/avaloniaui\" xmlns:x=\"http://schemas.microsoft.com/winfx/2006/xaml\">"
                + body + "</ResourceDictionary>");
        }

        [Fact]
        public Task A_theme_repaints_a_control_that_is_already_on_screen() => UiTest.Run(() =>
        {
            WriteTheme("Nocturne", "<Color x:Key=\"LunaSectionHeaderColor\">#7AA2F7</Color>"
                + "<SolidColorBrush x:Key=\"LunaSectionHeader\" Color=\"#7AA2F7\" />");

            var header = new SectionHeader { Text = "Load" };
            var window = new Window { Width = 300, Height = 200, Content = header };
            window.Show();

            Assert.Equal(Color.Parse("#9CDCFE"), Brush(header));

            Assert.True(LunaTheme.Apply("Nocturne"));

            // Live, with no restart: every consumer resolves the palette through DynamicResource.
            Assert.Equal(Color.Parse("#7AA2F7"), Brush(header));

            window.Close();
        });

        // The whole point of moving the ramp off a computed brush and onto pseudo-classes.
        [Fact]
        public Task A_theme_reaches_the_load_ramp() => UiTest.Run(() =>
        {
            WriteTheme("Hotter", "<SolidColorBrush x:Key=\"LunaHot\" Color=\"#FF00FF\" />");

            var row = new MeterRow { Label = "S-CPU", Percent = 95, ValueText = "95%" };
            var window = new Window { Width = 400, Height = 100, Content = row };
            window.Show();

            ProgressBar bar = row.FindPart<ProgressBar>()!;
            Assert.Equal(Color.Parse("#FF4500"), ((ISolidColorBrush)bar.Foreground!).Color);

            Assert.True(LunaTheme.Apply("Hotter"));

            Assert.Equal(Color.Parse("#FF00FF"), ((ISolidColorBrush)bar.Foreground!).Color);

            window.Close();
        });

        [Fact]
        public Task A_theme_reaches_a_window_surface() => UiTest.Run(() =>
        {
            WriteTheme("Pale", "<SolidColorBrush x:Key=\"LunaSurface\" Color=\"#FFFFFF\" />");

            var window = new ToolWindow { Width = 200, Height = 150 };
            window.Show();
            Assert.Equal(Color.Parse("#1E1E1E"), ((ISolidColorBrush)window.Background!).Color);

            Assert.True(LunaTheme.Apply("Pale"));
            Assert.Equal(Color.Parse("#FFFFFF"), ((ISolidColorBrush)window.Background!).Color);

            window.Close();
        });

        // A theme that only overrides two keys must not blank out the rest of the palette.
        [Fact]
        public Task Keys_a_theme_does_not_mention_keep_their_built_in_value() => UiTest.Run(() =>
        {
            WriteTheme("Sparse", "<SolidColorBrush x:Key=\"LunaSectionHeader\" Color=\"#7AA2F7\" />");

            var hint = new HintText { Text = "unchanged" };
            var window = new Window { Width = 300, Height = 200, Content = hint };
            window.Show();

            Assert.True(LunaTheme.Apply("Sparse"));
            Assert.Equal(Color.Parse("#808080"), Brush(hint));

            window.Close();
        });

        [Fact]
        public Task Switching_themes_replaces_rather_than_stacks() => UiTest.Run(() =>
        {
            WriteTheme("First", "<SolidColorBrush x:Key=\"LunaSectionHeader\" Color=\"#111111\" />");
            WriteTheme("Second", "<SolidColorBrush x:Key=\"LunaMuted\" Color=\"#222222\" />");

            var header = new SectionHeader { Text = "Load" };
            var window = new Window { Width = 300, Height = 200, Content = header };
            window.Show();

            LunaTheme.Apply("First");
            Assert.Equal(Color.Parse("#111111"), Brush(header));

            // Second says nothing about the header, so First's override must be gone rather than still layered underneath.
            LunaTheme.Apply("Second");
            Assert.Equal(Color.Parse("#9CDCFE"), Brush(header));

            window.Close();
        });

        // A theme is a file a user edits by hand, so a broken one must not take the program down.
        [Fact]
        public Task A_malformed_theme_is_refused_and_reported() => UiTest.Run(() =>
        {
            Directory.CreateDirectory(LunaTheme.Directory);
            File.WriteAllText(Path.Combine(LunaTheme.Directory, "Broken" + LunaTheme.Extension), "<not-xaml");

            var header = new SectionHeader { Text = "Load" };
            var window = new Window { Width = 300, Height = 200, Content = header };
            window.Show();

            Assert.False(LunaTheme.Apply("Broken"));
            Assert.Equal(Color.Parse("#9CDCFE"), Brush(header));
            Assert.Contains("Broken", _reported ?? "");

            window.Close();
        });

        [Fact]
        public Task A_missing_theme_is_refused_rather_than_throwing() => UiTest.Run(() =>
        {
            Assert.False(LunaTheme.Apply("NoSuchTheme"));
            Assert.Contains("NoSuchTheme", _reported ?? "");
        });

        [Fact]
        public Task The_catalog_lists_built_in_first_then_what_is_on_disk() => UiTest.Run(() =>
        {
            WriteTheme("Zebra", "");
            WriteTheme("Amber", "");

            Assert.Equal(new[] { LunaTheme.BuiltIn, "Amber", "Zebra" }, LunaTheme.Available());
        });

        [Fact]
        public Task The_chosen_theme_survives_a_restart() => UiTest.Run(() =>
        {
            WriteTheme("Nocturne", "<SolidColorBrush x:Key=\"LunaSectionHeader\" Color=\"#7AA2F7\" />");
            LunaTheme.Apply("Nocturne");

            Assert.Equal("Nocturne", LunaTheme.Saved);

            // What LunaApp does at startup.
            LunaTheme.ApplySaved();
            Assert.Equal("Nocturne", LunaTheme.Current);
        });

        // A theme deleted between runs must not also erase the choice - fixing the file and restarting should be enough.
        [Fact]
        public Task A_theme_that_stopped_resolving_falls_back_without_forgetting_the_choice() => UiTest.Run(() =>
        {
            WriteTheme("Nocturne", "<SolidColorBrush x:Key=\"LunaSectionHeader\" Color=\"#7AA2F7\" />");
            LunaTheme.Apply("Nocturne");

            File.Delete(Path.Combine(LunaTheme.Directory, "Nocturne" + LunaTheme.Extension));
            LunaTheme.ApplySaved();

            Assert.Equal(LunaTheme.BuiltIn, LunaTheme.Current);
            Assert.Equal("Nocturne", LunaTheme.Saved);
        });

        private static Color Brush(TextBlock text) => ((ISolidColorBrush)text.Foreground!).Color;
    }
}
