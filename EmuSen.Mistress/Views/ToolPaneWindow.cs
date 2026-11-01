using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for a tool's pane: its own window on the desktop, a framed sheet in big picture, the pad's buttons handed to the pane - see EmuSen_LibraryTools.md §1b.
    public abstract class ToolPaneWindow : ToolWindow, IPadDriven
    {
        private readonly AdaptiveLayout _pane;
        private bool _shown;

        protected ToolPaneWindow(string title, AdaptiveLayout pane)
        {
            Title = title;
            Width = 1120;
            Height = 720;
            MinWidth = 760;
            MinHeight = 480;
            CanResize = true;
            _pane = pane;
            Content = pane;
            // In big picture a tool is a screen of its own, full screen in the safe area, not a centred sheet (EmuSen_Interface.md §14.9).
            SheetLayer.SetScreen(this, true);
            pane.AttachedToVisualTree += (_, _) => Adapt();
            pane.SectionChosen += _ => ShowHints();
        }

        // What the pane does the first time it is shown, such as listing.
        protected abstract void FirstShown();

        // The form the host shows the pane in: big screen when the sheet is framed as a menu, the desktop's otherwise.
        private void Adapt()
        {
            _pane.Form = SheetLayer.PresenterOf(this)?.DrawsMenu(this) == true ? AdaptiveForm.BigScreen : AdaptiveForm.Desktop;
            ShowHints();
            if (_shown) return;
            _shown = true;
            FirstShown();
        }

        // The frame's help bar from the pane's roles and actions, its face buttons as the player's swap draws them.
        protected void ShowHints() =>
            MenuLook.SetHints(this, _pane.Form == AdaptiveForm.BigScreen ? _pane.Hints.Select(h => h with { Button = h.Button is { } b ? PadHints.Glyph(b) : null }).ToList() : null);

        // A pad's button in ES-DE's grammar, offered to the pane first; what it declines is the router's, B at the top closing the sheet.
        public bool OnPad(UiButton button) => Glyph(button) is { } glyph && _pane.Press(glyph);

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
