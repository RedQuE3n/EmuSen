using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.TextFormatting;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.LunaP.Media;
using EmuSen.Mistress.Input;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using SDL3;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Every page of the pad menu, its submenus and its questions, at 1280 by 800 and 1920 by 1200, on the desktop and as the big panel - see EmuSen_Settings_Reference.md §4.69.8.
    public partial class WindowFitAuditTests
    {
        public static TheoryData<string, bool, int, int> PadMenuCases()
        {
            var data = new TheoryData<string, bool, int, int>();
            foreach (string menu in new[] { "InGame", "InGameLongTitle", "Main" })
                foreach (bool bigScreen in new[] { true, false })
                    foreach ((int width, int height) in WindowLookPictureTool.Sizes) data.Add(menu, bigScreen, width, height);
            return data;
        }

        [Theory]
        [MemberData(nameof(PadMenuCases))]
        public Task Every_page_of_the_pad_menu_is_whole_inside_the_window_and_nothing_on_it_overlaps(string menu, bool bigScreen, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"PadMenu-{menu}-{(bigScreen ? "BigScreen" : "Desktop")}-{width}x{height}";
            (MainWindow window, PadDriver pad) = InGame(width, height, game: menu == "InGameLongTitle" ? LongTitle : "Cobalt Harbor (Synthetic)", bigScreen: bigScreen);
            try
            {
                if (menu == "Main")
                {
                    Call(window, "ToggleLibrary");
                    window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
                    pad.Start();
                }
                else pad.Chord(SDL.GamepadButton.Back, SDL.GamepadButton.Start);
                Assert.True(PadMenu.IsOpen(window));

                var faults = new List<string>(PadMenuPageFaults(window, bigScreen, $"{name}-First"));
                List<PadMenuEntry> top = PadMenu.Entries(window).ToList();
                for (int i = 0; i < top.Count; i++)
                {
                    if (top[i].Submenu is null && top[i].Question is null) continue;
                    Down(window, pad, i);
                    pad.A();
                    faults.AddRange(PadMenuPageFaults(window, bigScreen, $"{name}-{Short(top[i].Text())}"));
                    List<PadMenuEntry> page = PadMenu.Entries(window).ToList();
                    for (int j = 0; top[i].Submenu is not null && j < page.Count; j++)
                    {
                        if (page[j].Question is null) continue;
                        Down(window, pad, j);
                        pad.A();
                        faults.AddRange(PadMenuPageFaults(window, bigScreen, $"{name}-{Short(top[i].Text())}-{Short(page[j].Text())}"));
                        pad.B();
                    }
                    pad.B();
                    Assert.Equal(1, PadMenu.Depth(window));
                }

                // The slot's card beside the menu, on the Save State row.
                if (menu != "Main")
                {
                    Down(window, pad, 2);
                    faults.AddRange(PadMenuPageFaults(window, bigScreen, $"{name}-SaveState"));
                }
                Assert.Empty(faults);
            }
            finally
            {
                Stop(window);
                window.Close();
            }
        }, default);

        private static string Short(string text) => new(text.Where(char.IsLetterOrDigit).ToArray());

        private static void Down(MainWindow w, PadDriver pad, int to)
        {
            int count = PadMenu.Entries(w).Count;
            pad.Down((to - Math.Max(PadMenu.Selected(w), 0) + count) % count);
        }

        // The page shown: the fit audit over the menu, every big row's words inside their parts, every subtitle line inside the panel, no rows past it unless the title takes two lines, and the slot's card clear of the panel and the help.
        private List<string> PadMenuPageFaults(MainWindow window, bool bigScreen, string name)
        {
            Settle(window);
            Control root = bigScreen ? window.GetControl<Control>("PadMenuPanel") : window.GetControl<Control>("PadMenuDesk");
            var faults = FitAudit.Check(root, null, smallest: bigScreen ? FitAudit.SmallestText : 0).Select(f => $"{name}: {f}").ToList();
            Rect area = new(window.GetControl<Control>("PadMenuPanel").Bounds.Size);
            Rect box;
            Rect help = default;
            if (bigScreen)
            {
                MenuPanel panel = PadMenu.Big(window);
                double u = MenuPanel.GetScale(panel);
                box = panel.PanelBounds;
                help = panel.HelpBar.Bounds;
                if (panel.IsTitleCut) faults.Add($"{name}: title cut: '{panel.Title}'");
                if (panel.ScrollIndicator != MenuScrollIndicator.None && panel.TitleLines <= 1) faults.Add($"{name}: rows run past the panel under a one-line title");
                GlyphTypeface face = MenuPanel.GetFontPath(panel) is { } path && FontFiles.Load(path) is { } f ? f : FontFiles.Default;
                foreach (string line in (panel.Subtitle ?? "").Split('\n').Where(l => l.Length > 0))
                {
                    string cased = panel.SubtitleLetterCase == LetterCase.Upper ? line.ToUpper(CultureInfo.CurrentCulture) : line;
                    if (Measure(face, panel.SubtitleSize * u, cased) > box.Width - 48 * u + 1) faults.Add($"{name}: subtitle line cut: '{line}'");
                }
                foreach (MenuRow row in PadMenu.List(window).GetVisualDescendants().OfType<MenuRow>().Where(r => r.IsEffectivelyVisible))
                {
                    MenuRowLayout parts = row.Layout(row.Bounds.Size);
                    string label = (MenuRows.GetLabel((Control)row.TemplatedParent!) ?? row.Label ?? "").ToUpper(CultureInfo.CurrentCulture);
                    if (Measure(face, row.TextSize * u, label) > parts.Label.Width + 1) faults.Add($"{name}: row label cut: '{label}'");
                    string value = (row.Value ?? "").ToUpper(CultureInfo.CurrentCulture);
                    if (value.Length > 0 && Measure(face, row.TextSize * u, value) > parts.Value.Width + 1) faults.Add($"{name}: row value cut: '{value}'");
                }
                _out.WriteLine($"PADMENU {name}: {panel.TitleLines} title line(s), {PadMenu.Lines(window).Length} rows, scroll {panel.ScrollIndicator}");
            }
            else
            {
                box = window.GetControl<Control>("PadMenuDesk").Bounds;
                if (!area.Contains(box)) faults.Add($"{name}: the menu {box} leaves the window {area}");
                if (PadMenu.List(window).GetVisualDescendants().OfType<ScrollViewer>().FirstOrDefault() is { } viewer && viewer.Extent.Height > viewer.Viewport.Height + 0.5)
                    faults.Add($"{name}: the desktop's rows scroll, {viewer.Extent.Height:F0} in {viewer.Viewport.Height:F0}");
            }
            Control card = window.GetControl<Control>("PadSlotCard");
            if (card.IsVisible)
            {
                if (!area.Contains(card.Bounds)) faults.Add($"{name}: the slot card {card.Bounds} leaves the window {area}");
                if (card.Bounds.Intersects(box)) faults.Add($"{name}: the slot card {card.Bounds} overlaps the menu {box}");
                if (help.Width > 0 && card.Bounds.Intersects(help)) faults.Add($"{name}: the slot card {card.Bounds} overlaps the help {help}");
                _out.WriteLine($"PADMENU {name}: slot card {card.Bounds}");
            }
            foreach (string fault in faults) _out.WriteLine(fault);
            SavePicture(window, name);
            return faults;
        }

        private static double Measure(GlyphTypeface face, double size, string text) =>
            text.Length == 0 ? 0 : TextShaper.Current.ShapeText(text.AsMemory(), new TextShaperOptions(face, size, 0, CultureInfo.CurrentCulture, 0, 0, null)).Sum(g => g.GlyphAdvance);
    }
}
