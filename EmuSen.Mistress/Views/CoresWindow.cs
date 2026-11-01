using System;
using System.Linq;
using System.Threading.Tasks;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Tools.Cores;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for Cores: its own window on the desktop, a framed sheet in big picture, the runtime and the pad handed to the pane - see EmuSen_Cores.md §3.4.
    public sealed class CoresWindow : ToolWindow, IPadDriven
    {
        // The approval question, which a test answers in place of a person.
        internal static Func<Avalonia.Controls.Window, string, string, Task<bool>> Confirm =
            (owner, title, message) => Dialogs.ConfirmAsync(owner, title, message, "Approve", "Not Now");

        private bool _listed;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public CoresWindow() : this(null, null) { }

        public CoresWindow(CoresRuntime? runtime, CoresHost? host)
        {
            Title = "Cores";
            Width = 1120;
            Height = 720;
            MinWidth = 760;
            MinHeight = 480;
            CanResize = true;
            var asked = host ?? new CoresHost();
            Pane = new CoresPane(runtime, new CoresHost { Options = asked.Options, Firmware = asked.Firmware, OpenFolder = asked.OpenFolder, Confirm = (title, message) => Confirm(this, title, message) });
            Content = Pane;
            Pane.AttachedToVisualTree += (_, _) => Adapt();
            Pane.MarksChanged += ShowHints;
            Pane.SectionChosen += _ => ShowHints();
        }

        public CoresPane Pane { get; }

        // Big screen when the sheet is framed as a menu, the desktop's form otherwise; the first time, the cores are listed.
        private void Adapt()
        {
            Pane.Form = SheetLayer.PresenterOf(this)?.DrawsMenu(this) == true ? AdaptiveForm.BigScreen : AdaptiveForm.Desktop;
            ShowHints();
            if (_listed) return;
            _listed = true;
            Pane.Refresh();
        }

        private void ShowHints() =>
            MenuLook.SetHints(this, Pane.Form == AdaptiveForm.BigScreen ? Pane.Hints.Select(h => h with { Button = h.Button is { } b ? PadHints.Glyph(b) : null }).ToList() : null);

        // A pad's button in ES-DE's grammar, offered to the frame first; what it declines is the router's, B at the top closing the sheet.
        public bool OnPad(UiButton button) => Glyph(button) is { } glyph && Pane.Press(glyph);

        private static PadGlyphButton? Glyph(UiButton button) => button switch
        {
            UiButton.Accept => PadGlyphButton.South,
            UiButton.Back => PadGlyphButton.East,
            UiButton.Screensaver => PadGlyphButton.West,
            UiButton.Search => PadGlyphButton.North,
            UiButton.PageUp => PadGlyphButton.LeftShoulder,
            UiButton.PageDown => PadGlyphButton.RightShoulder,
            UiButton.Menu => PadGlyphButton.Start,
            UiButton.Options => PadGlyphButton.Select,
            _ => null,
        };
    }
}
