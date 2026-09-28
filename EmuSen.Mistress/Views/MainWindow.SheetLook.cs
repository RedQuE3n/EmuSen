using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.LogicalTree;
using Avalonia.Media;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // The windows a big-screen session shows as sheets, framed as ES-DE's menus and drawn in their look - see EmuSen_Settings_Reference.md §4.80.
    public partial class MainWindow
    {
        // Barlow Condensed as a font family, for the stock controls of a framed sheet; the menus draw the same file by its path.
        internal static readonly FontFamily BigMenuFontFamily = new("avares://EmuSen.Mistress/Assets/Fonts#Barlow Condensed");

        private void SetUpSheetLook()
        {
            Resources[MenuLook.FontFamilyKey] = BigMenuFontFamily;
            Sheets.MenuHintsFor = SheetMenuHints;
            Sheets.MenuFrameFor = FramedAsMenu;
        }

        // The windows checked in ES-DE's look so far; the others keep the plain sheet until theirs is (§4.80).
        internal static bool FramedAsMenu(Window window) => window is ScrapeStatusWindow or ActiveCheatsWindow or CheatDatabaseWindow or InputSettingsWindow;

        // A framed sheet's help bar, from what its window holds: choose and back always, tabs and sideways values where there are any.
        internal static IReadOnlyList<HintEntry> SheetMenuHints(Window window)
        {
            IEnumerable<ILogical> all = (window.Content as ILogical)?.GetSelfAndLogicalDescendants() ?? [];
            if (SheetLayer.PresenterOf(window)?.SheetOf(window) is ILogical sheet) all = sheet.GetSelfAndLogicalDescendants();
            List<ILogical> controls = all.ToList();
            bool tabs = controls.OfType<TabControl>().Any(t => t.IsVisible && t.ItemCount > 1);
            bool sideways = controls.OfType<Control>().Any(c => c is ComboBox or Slider or ISidewaysAdjustable && c.IsVisible);
            var hints = new List<HintEntry>
            {
                new("Select") { Button = AcceptGlyph },
                new("Back") { Button = BackGlyph },
            };
            if (tabs)
            {
                hints.Add(new HintEntry("") { Button = PadGlyphButton.LeftShoulder });
                hints.Add(new HintEntry("Tab") { Button = PadGlyphButton.RightShoulder });
            }
            if (sideways) hints.Add(new HintEntry("Change") { Button = PadGlyphButton.DPadLeftRight });
            hints.Add(new HintEntry("Choose") { Button = PadGlyphButton.DPadUpDown });
            return hints;
        }

        // A framed sheet's text box keeps the keys it types with; Up and Down still move out of it, as a pad would (§4.80).
        private bool FramedTextBoxKeepsKey(Key key) =>
            Sheets.Current is { } sheet && Sheets.DrawsMenu(sheet) && !SheetLayer.GetChromeless(sheet) && key is not (Key.Up or Key.Down);
    }
}
