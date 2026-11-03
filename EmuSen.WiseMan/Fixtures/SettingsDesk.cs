using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.LogicalTree;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Pages;
using EmuSen.Mistress.Views;
using Xunit;

namespace EmuSen.WiseMan.Fixtures
{
    // Settings' desktop form, as the menu bar's Settings opens it: the window at a section and a page, a row reached and turned through the window's own router, as a pad or the keys do - see EmuSen_BigPicture.md §52.6.
    public static class SettingsDesk
    {
        // A desktop main window with Settings open at the section and the rows' path, such as Consoles and "console:N64".
        public static MainWindow Open(string section, params string[] path)
        {
            // A main window reads and writes the settings and the library's records, so a test's own folders must be in place first.
            Assert.NotNull(EmuSen.Galaxia.ConfigStore.OverrideDirectory);
            Assert.NotNull(EmuSen.Galaxia.Library.DataStore.OverrideDirectory);
            var window = new MainWindow { Width = 1280, Height = 800 };
            window.Show();
            At(window, section, path);
            return window;
        }

        public static void At(MainWindow window, string section, params string[] path)
        {
            window.ShowSettings(section, path);
            Assert.IsType<SettingsWindow>(Of(window));
        }

        // Display ▸ Shaders ▸ the console's page; in a game Shaders opens on the running console's, so B first returns to the list.
        public static void ShaderPage(MainWindow window, string console)
        {
            At(window, SettingsScreen.Display, "shaders");
            if (window.SettingsShown!.Trail.Count > 2) Back(window);
            Choose(window, "shader:" + console);
        }

        public static Window Of(MainWindow window) => window.OwnedWindows.OfType<SettingsWindow>().Single();

        public static SettingsLayout Layout(MainWindow window) => window.SettingsLayoutShown!;

        public static IReadOnlyList<string> Keys(MainWindow window) => Layout(window).Rows.Select(r => r.Key).ToList();

        public static PageRow Row(MainWindow window, string key) => Layout(window).Rows.Single(r => r.Key == key);

        public static string? Value(MainWindow window, string key) => Row(window, key).Status;

        public static void Press(MainWindow window, UiButton button) => PadWindowRouter.Send(Of(window), button);

        // Down or Up from the focused row to the one asked for.
        public static void To(MainWindow window, string key)
        {
            List<string> keys = Keys(window).ToList();
            Assert.Contains(key, keys);
            for (int i = 0; i <= keys.Count && Layout(window).Focused != key; i++)
                Press(window, Layout(window).Focused is { } at && keys.IndexOf(at) > keys.IndexOf(key) ? UiButton.Up : UiButton.Down);
            Assert.Equal(key, Layout(window).Focused);
        }

        // Right by so many steps, or Left for a negative count.
        public static void Step(MainWindow window, string key, int by)
        {
            To(window, key);
            for (int i = 0; i < System.Math.Abs(by); i++) Press(window, by > 0 ? UiButton.Right : UiButton.Left);
        }

        // Steps a choice row until it shows the text asked for, failing after a whole turn of its choices.
        public static void Pick(MainWindow window, string key, string text)
        {
            To(window, key);
            // A number with no list of choices is stepped by the difference.
            if (Row(window, key).Choices is null && int.TryParse(Value(window, key), out int now) && int.TryParse(text, out int wanted))
                for (int i = 0; i < Math.Abs(wanted - now); i++) Press(window, wanted > now ? UiButton.Right : UiButton.Left);
            int turn = Row(window, key).Choices?.Count ?? 0;
            for (int i = 0; i < turn && Value(window, key) != text; i++) Press(window, UiButton.Right);
            Assert.Equal(text, Value(window, key));
        }

        public static void Choose(MainWindow window, string key)
        {
            To(window, key);
            Press(window, UiButton.Accept);
        }

        public static void Back(MainWindow window) => Press(window, UiButton.Back);

        // The question a row asked, answered with the words typed and its OK, wherever it is shown: a sheet or a window of its own.
        public static void Answer(MainWindow window, string text)
        {
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Window prompt = Prompts(window).Single();
            prompt.GetLogicalDescendants().OfType<TextBox>().Single(t => t.Name == "PART_Answer").Text = text;
            prompt.GetLogicalDescendants().OfType<Button>().Single(b => b.Name == "PART_Accept").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
        }

        // The question a row asked, put away with Cancel.
        public static void Cancel(MainWindow window)
        {
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            Window prompt = Prompts(window).Single();
            prompt.GetLogicalDescendants().OfType<Button>().Single(b => b.Name == "PART_Cancel").RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
        }

        public static Window Prompt(MainWindow window)
        {
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            return Prompts(window).Single();
        }

        private static IEnumerable<Window> Prompts(Window owner)
        {
            var sheets = owner.GetLogicalDescendants().OfType<EmuSen.LunaP.Windowing.SheetLayer>().FirstOrDefault();
            if (sheets?.Current is { } current && current.GetType().Name == "PromptWindow") yield return current;
            foreach (Window owned in owner.OwnedWindows)
            {
                if (owned.GetType().Name == "PromptWindow") yield return owned;
                foreach (Window deeper in Prompts(owned)) yield return deeper;
            }
        }
    }
}
