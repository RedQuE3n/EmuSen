using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.VisualTree;
using EmuSen.Cores;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // Preferences ▸ Firmware at 1280 by 800 and 1920 by 1200, on the desktop and as the big-screen menu reached from the pad menu's Settings - see EmuSen_Settings_Reference.md §4.89.
    public partial class WindowFitAuditTests
    {
        // One file in use, the chip of the longest words with a file of the wrong size, and the rest as they come, so every form of words is on the page.
        private static void FirmwareFiles()
        {
            Directory.CreateDirectory(DataStore.Firmware);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "spc700.rom"), new byte[64]);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "st010.rom"), new byte[100]);
        }

        [Theory]
        [MemberData(nameof(StatusCases))]
        public Task The_firmware_page_in_preferences_is_whole_at_both_sizes(bool bigScreen, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"PreferencesFirmware-{(bigScreen ? "BigScreen" : "Desktop")}-{width}x{height}";
            (MainWindow window, PadDriver pad) = InGame(width, height, bigScreen: bigScreen);
            try
            {
                FirmwareFiles();
                Call(window, "ToggleLibrary");
                window.GetControl<ListBox>("LibraryList").SelectedIndex = 0;
                pad.Start();
                PadMenu.Choose(window, pad, PreferencesWindow.FirmwareTab, exact: true);
                Settle(window);
                Window top = Sheets(window).Current ?? window.OwnedWindows.OfType<PreferencesWindow>().Last();
                var preferences = Assert.IsType<PreferencesWindow>(top);
                Control root = Sheets(window).SheetOf(top) ?? top;
                IReadOnlyList<FirmwareItem> items = FirmwareOverview.Build().SelectMany(s => s.Items).ToList();
                Assert.Contains(items, i => i.Present);
                Assert.Contains(items, i => i.WrongSize);

                var faults = new List<string>();
                if (!bigScreen)
                {
                    Settle(top);
                    Assert.True(top.Bounds.Height <= height && top.Bounds.Width <= width, $"{name}: the window is {top.Bounds.Size}");
                    // A desktop window keeps the desktop's text sizes and scrolls its pane behind a scroll bar, as the players' rows do.
                    faults.AddRange(FitAudit.Check(root, WindowAllowances.For(top), smallest: 0).Where(f => !f.StartsWith("cut at a scrolling edge", StringComparison.Ordinal)));
                    Assert.All(items, i => Assert.Contains(root.GetVisualDescendants().OfType<FieldRow>(), r => r.Label == i.Title && r.GetVisualDescendants().OfType<TextBlock>().Any(t => t.Text == i.Change)));
                    Assert.Contains(root.GetVisualDescendants().OfType<Button>(), b => b.Name == "FirmwareOpenFolderButton");
                    SavePicture(top, name);
                }
                else
                {
                    // Every row with the focus on it, since the footer's words are the focused row's.
                    Assert.Equal(PreferencesWindow.FirmwareTab, preferences.Form!.Menu.Title);
                    List<Button> rows = root.GetVisualDescendants().OfType<StackPanel>().Single(p => p.Name == "BigMenuRows").Children.OfType<Button>().ToList();
                    Assert.Equal(new[] { FirmwarePane.FolderLabel, FirmwarePane.FolderRowLabel }.Concat(FirmwareOverview.Build().SelectMany(s => s.Items.Count == 0 ? [s.Name] : s.Items.Select(i => i.Title))), rows.Select(r => MenuRows.GetLabel(r)));
                    Assert.DoesNotContain(root.GetVisualDescendants().OfType<Button>(), b => (b.Content as string) == FirmwarePane.OpenFolder);
                    for (int i = 0; i < rows.Count; i++)
                    {
                        Settle(window);
                        Assert.Same(rows[i], window.FocusManager!.GetFocusedElement());
                        Assert.False(string.IsNullOrEmpty(preferences.Form.Menu.Footer), $"{name}: no footer on row {i}");
                        if (i == 1) Assert.Equal((FirmwareOverview.Folder, EmuSen.LunaP.Media.LetterCase.None), (preferences.Form.Menu.Footer, preferences.Form.Menu.FooterLetterCase));
                        else Assert.Null(preferences.Form.Menu.FooterLetterCase);
                        if (preferences.Form.Menu.IsFooterCut) faults.Add($"row {i} ({MenuRows.GetLabel(rows[i])}): the footer is cut: {preferences.Form.Menu.Footer}");
                        foreach (string f in FitAudit.Check(root, WindowAllowances.For(top))) faults.Add($"row {i} ({MenuRows.GetLabel(rows[i])}): {f}");
                        if (i is 0 or 1 || i == rows.Count - 1 || items.Any(it => it.WrongSize && it.Title == MenuRows.GetLabel(rows[i]))) SavePicture(window, $"{name}-Row{i:D2}");
                        pad.Down();
                    }
                    // Every row and Back is reached by the d-pad, and B goes back to the tabs, then closes the sheet.
                    HashSet<InputElement> reached = PadAudit.Reachable(root, pad);
                    Assert.All(PadAudit.Operable(root), e => Assert.Contains(e, reached));
                    pad.B();
                    Assert.Equal("Preferences", preferences.Form.Menu.Title);
                    pad.B();
                    Settle(window);
                    Assert.False(Sheets(window).IsPresenting);
                }
                foreach (string f in faults) _out.WriteLine($"{name}: {f}");
                Assert.Empty(faults);
            }
            finally
            {
                Stop(window);
                window.Close();
            }
        }, default);
    }
}
