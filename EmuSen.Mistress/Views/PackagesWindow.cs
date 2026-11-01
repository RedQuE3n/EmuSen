using System.Collections.Generic;
using System.Linq;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;
using EmuSen.Tools.Packages;

namespace EmuSen.Mistress.Views
{
    // Mistress's thin host for the Packages tool: its own window on the desktop, a framed sheet in big picture, the pad's buttons handed to the frame - see EmuSen_LibraryTools.md §1b and EmuSen_Packages.md §7.
    public sealed class PackagesWindow : ToolWindow, IPadDriven
    {
        private bool _listed;

        // Parameterless constructor exists only for tooling - real code always uses the one below.
        public PackagesWindow() : this(null) { }

        public PackagesWindow(string? library)
        {
            Title = "Packages";
            Width = 1120;
            Height = 720;
            MinWidth = 760;
            MinHeight = 480;
            CanResize = true;
            Pane = new PackagesPane { Library = library };
            Content = Pane;
            Pane.AttachedToVisualTree += (_, _) => Adapt();
            Pane.MarksChanged += ShowHints;
            Pane.SectionChosen += _ => ShowHints();
        }

        public PackagesPane Pane { get; }

        // The form the host shows the pane in: big screen when the sheet is framed as a menu, the desktop's otherwise; the first time, the library is listed.
        private void Adapt()
        {
            Pane.Form = SheetLayer.PresenterOf(this)?.DrawsMenu(this) == true ? AdaptiveForm.BigScreen : AdaptiveForm.Desktop;
            ShowHints();
            if (_listed) return;
            _listed = true;
            Pane.Refresh();
        }

        // The frame's help bar from the pane's roles and actions, its face buttons as the player's swap draws them.
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
