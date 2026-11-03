using System;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.LogicalTree;
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
    public sealed class ThemeStore : IAsyncLifetime
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

        public Task InitializeAsync() => Task.CompletedTask;

        // On the session's dispatcher: off it there is no Application, and Apply does nothing - see EmuSen_Settings_Reference.md §4.78.3.
        public async Task DisposeAsync()
        {
            await UiTest.Run(() => LunaTheme.Apply(LunaTheme.BuiltIn));
            LunaSettings.Store = _previous;
            try { Directory.Delete(Root, recursive: true); } catch { }
        }
    }

    // The theme picker `man theme` promises, Settings ▸ Display ▸ Window colours on the desktop - see EmuSen_LunaP.md §8.2.
    [Collection(TestCollections.ProcessGlobals)]
    public class WindowColoursTests : IClassFixture<ThemeStore>, IAsyncLifetime
    {
        private readonly string _root;

        public WindowColoursTests(ThemeStore store)
        {
            _root = store.Root;

            foreach (string file in Directory.GetFiles(LunaTheme.Directory)) File.Delete(file);
        }

        // The applied theme is process-global, so a test that left one on would tint every window after it; reset on the dispatcher, where Apply has an Application to act on.
        public Task InitializeAsync() => UiTest.Run(() => LunaTheme.Apply(LunaTheme.BuiltIn));

        public Task DisposeAsync() => UiTest.Run(() => LunaTheme.Apply(LunaTheme.BuiltIn));

        private void WriteTheme(string name, string surface) =>
            File.WriteAllText(Path.Combine(LunaTheme.Directory, name + ".css"),
                ":root {\n  --luna-surface: " + surface + ";\n}\n");

        // Settings' Display rows on the desktop, where Window colours is offered.
        private static EmuSen.Mistress.Pages.SettingsScreen Shown() => new(new SettingsHostFake { Desktop = true }, EmuSen.Mistress.Pages.SettingsScreen.Display);

        private static PageRow ThemeRow(EmuSen.Mistress.Pages.SettingsScreen screen) => screen.Rows.Single(r => r.Key == "colours");

        // Right or Left to the theme named, as the row is stepped.
        private static void Choose(EmuSen.Mistress.Pages.SettingsScreen screen, string name)
        {
            PageRow row = ThemeRow(screen);
            int by = row.Choices!.ToList().IndexOf(name) - row.Choices!.ToList().IndexOf(row.Status!);
            screen.Step("colours", by);
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
        }

        [Fact]
        public Task The_theme_row_lists_the_built_in_theme_and_every_theme_file() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");
            WriteTheme("aurora", "#12233a");

            var screen = Shown();

            Assert.Equal(new[] { LunaTheme.BuiltIn, "aurora", "midnight" }, ThemeRow(screen).Choices);
            Assert.Equal(LunaTheme.Current, ThemeRow(screen).Status);
            Assert.DoesNotContain(new EmuSen.Mistress.Pages.SettingsScreen(new SettingsHostFake(), EmuSen.Mistress.Pages.SettingsScreen.Display).Rows, r => r.Key == "colours");
        });

        [Fact]
        public Task Choosing_a_theme_applies_it_and_remembers_it() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");

            var screen = Shown();
            Choose(screen, "midnight");

            Assert.Equal("midnight", LunaTheme.Current);
            Assert.Equal("midnight", LunaTheme.Saved);
            Assert.Contains("midnight", File.ReadAllText(Path.Combine(_root, "luna.json")));
        });

        [Fact]
        public Task A_theme_that_will_not_load_leaves_the_applied_one_alone() => UiTest.Run(() =>
        {
            WriteTheme("midnight", "#101018");
            File.WriteAllText(Path.Combine(LunaTheme.Directory, "broken.css"), "this is not a theme file {{{\n");

            var screen = Shown();
            Choose(screen, "midnight");
            Choose(screen, "broken");

            Assert.Equal("midnight", LunaTheme.Current);
            Assert.Equal("midnight", ThemeRow(screen).Status);
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

            Choose(Shown(), "midnight");

            Color after = ((ISolidColorBrush)already.Background!).Color;

            Assert.NotEqual(before, after);
            Assert.Equal(Color.Parse("#101018"), after);

            already.Close();
        });
    }
}
