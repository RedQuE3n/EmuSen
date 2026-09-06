using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Settings;
using EmuSen.LunaP.Theme;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // One store for the whole class: LunaP reads the remembered choice through whichever store it first saw - see EmuSen_LunaP.md §8.2.
    public sealed class ThemeStore : IDisposable
    {
        private readonly ISettingsStore? _previous = LunaSettings.Store;

        public ThemeStore()
        {
            Root = Path.Combine(Path.GetTempPath(), "EmuSenPreferencesThemeTests", Guid.NewGuid().ToString("N"));
            Directory.CreateDirectory(Root);
            LunaSettings.Store = new JsonSettingsStore(Root);
            Directory.CreateDirectory(LunaTheme.Directory);
        }

        public string Root { get; }

        public void Dispose()
        {
            LunaTheme.Apply(LunaTheme.BuiltIn);
            LunaSettings.Store = _previous;
            try { Directory.Delete(Root, recursive: true); } catch { }
        }
    }

    // The theme picker `man theme` promises - see EmuSen_LunaP.md §8.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class PreferencesThemeTests : IClassFixture<ThemeStore>, IDisposable
    {
        private readonly string _root;

        public PreferencesThemeTests(ThemeStore store)
        {
            _root = store.Root;

            foreach (string file in Directory.GetFiles(LunaTheme.Directory)) File.Delete(file);
            LunaTheme.Apply(LunaTheme.BuiltIn);
        }

        // The applied theme is process-global, so a test that left one on would tint every window after it.
        public void Dispose() => LunaTheme.Apply(LunaTheme.BuiltIn);

        private void WriteTheme(string name, string surface) =>
            File.WriteAllText(Path.Combine(LunaTheme.Directory, name + ".css"),
                ":root {\n  --luna-surface: " + surface + ";\n}\n");

        private static Dropdown ThemeRow(PreferencesWindow window) =>
            window.GetVisualDescendants().OfType<Dropdown>().Single(d => d.Name == "ThemeDropdown");

        private static PreferencesWindow Shown()
        {
            var window = new PreferencesWindow(new AppSettings());
            window.Show();
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            return window;
        }

        [Fact]
        public Task The_theme_row_lists_the_built_in_theme_and_every_theme_file() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");
            WriteTheme("aurora", "#12233a");

            var window = Shown();

            Assert.Equal(
                new[] { LunaTheme.BuiltIn, "aurora", "midnight" },
                ThemeRow(window).Items.Cast<string>().ToArray());
            Assert.Equal(LunaTheme.Current, ThemeRow(window).SelectedItem);

            window.Close();
        });

        [Fact]
        public Task Choosing_a_theme_applies_it_and_remembers_it() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");

            var window = Shown();
            ThemeRow(window).SelectedItem = "midnight";
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();

            Assert.Equal("midnight", LunaTheme.Current);
            Assert.Equal("midnight", LunaTheme.Saved);
            Assert.Contains("midnight", File.ReadAllText(Path.Combine(_root, "luna.json")));

            window.Close();
        });

        [Fact]
        public Task A_theme_that_will_not_load_leaves_the_applied_one_alone() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");
            File.WriteAllText(Path.Combine(LunaTheme.Directory, "broken.css"), "this is not a theme file {{{\n");

            var window = Shown();
            ThemeRow(window).SelectedItem = "midnight";
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            ThemeRow(window).SelectedItem = "broken";
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();

            Assert.Equal("midnight", LunaTheme.Current);
            Assert.Equal("midnight", ThemeRow(window).SelectedItem);

            window.Close();
        });

        // `man theme` promises no restart, and a window opened before the choice is where that is decided.
        [Fact]
        public Task A_chosen_theme_reaches_a_window_that_was_already_open() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");

            var already = new ToolWindow { Width = 200, Height = 200 };
            already.Show();
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Color before = ((ISolidColorBrush)already.Background!).Color;

            var window = Shown();
            ThemeRow(window).SelectedItem = "midnight";
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();

            Color after = ((ISolidColorBrush)already.Background!).Color;

            Assert.NotEqual(before, after);
            Assert.Equal(Color.Parse("#101018"), after);

            window.Close();
            already.Close();
        });
    }
}
