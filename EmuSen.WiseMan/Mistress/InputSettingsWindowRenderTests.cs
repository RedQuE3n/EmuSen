using System.Threading.Tasks;
using EmuSen.Mistress.Input;
using EmuSen.Nehellania.Input;
using EmuSen.Galaxia;
using EmuSen.Galaxia.Models;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress
{
    // Real Skia render pass over the settings window - see EmuSen_Settings_Reference.md §4.7.
    public class InputSettingsWindowRenderTests
    {
        // Every tab, because only the selected one is realised - see EmuSen_Settings_Reference.md §4.6.
        [Theory]
        [InlineData(null)]
        [InlineData("NES")]
        [InlineData("SNES")]
        public Task The_window_renders_its_rows(string? console) => UiTest.Run(() =>
        {
            var window = new InputSettingsWindow(new ControllerKeyBindings(new[] { "NES", "SNES" }),
                new GamepadBindings(new[] { "NES", "SNES" }), null!, new AppSettings(), new HotkeyBindingMap(), console);
            window.Show();

            UiTest.AssertLaidOut(window, $"input-settings-{console ?? "General"}");
        });
    }
}
