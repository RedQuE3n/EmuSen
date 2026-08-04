using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Headless;
using Avalonia.VisualTree;
using EmuSen.Mistress.Input;
using EmuSen.Nehellania.Input;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Mistress
{
    // Column headers really sit over their own column - see EmuSen_Settings_Reference.md §4.6.
    public class InputSettingsWindowLayoutTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(InputSettingsWindowLayoutTests).GetTypeInfo().Assembly);

        private static InputSettingsWindow LaidOutWindow()
        {
            var window = new InputSettingsWindow(new ControllerKeyBindings(new[] { "NES", "SNES" }), new GamepadBindings(new[] { "NES", "SNES" }), null!, new AppSettings(), new HotkeyBindingMap(), "SNES");
            window.Show();
            window.CaptureRenderedFrame(); // forces the layout pass the assertions read
            return window;
        }

        private static double LeftEdge(Visual control, Visual relativeTo) =>
            control.TranslatePoint(new Point(0, 0), relativeTo)!.Value.X;

        // Built in code inside the selected console tab, so there is no XAML name scope to ask.
        private static T ByName<T>(InputSettingsWindow w, string name) where T : Control =>
            w.GetVisualDescendants().OfType<T>().First(c => c.Name == name);

        private static TextBlock HeaderCell(InputSettingsWindow w, string text) =>
            ByName<Grid>(w, "ButtonHeaderRow").Children.OfType<TextBlock>().Single(t => t.Text == text);

        private static Control FirstRowCell(InputSettingsWindow w, int column)
        {
            var row = (Grid)ByName<StackPanel>(w, "BindingsPanel").Children[0];
            return row.Children.Cast<Control>().Single(c => Grid.GetColumn(c) == column);
        }

        [Fact]
        public Task The_gamepad_header_sits_over_the_gamepad_column() => Session.Dispatch(() =>
        {
            var window = LaidOutWindow();

            double header = LeftEdge(HeaderCell(window, "Gamepad"), window);
            double cell = LeftEdge(FirstRowCell(window, 4), window);

            Assert.True(System.Math.Abs(header - cell) < 1,
                $"'Gamepad' header is at x={header:F1} but the pad binding it labels is at x={cell:F1}.");
        }, default);

        [Fact]
        public Task The_keyboard_header_sits_over_the_keyboard_column() => Session.Dispatch(() =>
        {
            var window = LaidOutWindow();

            double header = LeftEdge(HeaderCell(window, "Keyboard"), window);
            double cell = LeftEdge(FirstRowCell(window, 1), window);

            Assert.True(System.Math.Abs(header - cell) < 1,
                $"'Keyboard' header is at x={header:F1} but the key binding it labels is at x={cell:F1}.");
        }, default);

        [Fact]
        public Task The_button_header_sits_over_the_button_names() => Session.Dispatch(() =>
        {
            var window = LaidOutWindow();

            double header = LeftEdge(HeaderCell(window, "Button"), window);
            double cell = LeftEdge(FirstRowCell(window, 0), window);

            Assert.True(System.Math.Abs(header - cell) < 1,
                $"'Button' header is at x={header:F1} but the button name it labels is at x={cell:F1}.");
        }, default);

        // "Press a key..." is wider than "Rebind Key" - see EmuSen_Settings_Reference.md §4.6.
        [Fact]
        public Task Listening_for_a_key_does_not_shift_the_columns() => Session.Dispatch(() =>
        {
            var window = LaidOutWindow();
            double before = LeftEdge(FirstRowCell(window, 4), window);

            ((Button)FirstRowCell(window, 2)).RaiseEvent(new Avalonia.Interactivity.RoutedEventArgs(Button.ClickEvent));
            window.CaptureRenderedFrame();

            Assert.Equal("Press a key...", ((Button)FirstRowCell(window, 2)).Content);
            double after = LeftEdge(FirstRowCell(window, 4), window);
            Assert.True(System.Math.Abs(before - after) < 1,
                $"The gamepad column moved from x={before:F1} to x={after:F1} while waiting for a key.");
        }, default);
    }
}
