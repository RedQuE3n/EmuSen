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
        private string ChosenFirmware => Path.Combine(_root, "BIOS");

        // One file in use from the player's folder, one from EmuSen's, the chip of the longest words with a file of the wrong size in EmuSen's, and the rest as they come, so every form of words is on the page.
        private void FirmwareFiles()
        {
            Directory.CreateDirectory(DataStore.Firmware);
            Directory.CreateDirectory(ChosenFirmware);
            File.WriteAllBytes(Path.Combine(ChosenFirmware, "spc700.rom"), new byte[64]);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "bios_MD.bin"), new byte[2048]);
            File.WriteAllBytes(Path.Combine(DataStore.Firmware, "st010.rom"), new byte[100]);
        }

        // A row of the menu by its label: a fact's button, or the folder's path row, whose label is its box's.
        private static string? MenuLabelOf(Control row) => row is PathPickerRow picker ? MenuRows.GetLabel(picker.GetVisualDescendants().OfType<TextBox>().Single()) : MenuRows.GetLabel(row);

        [Theory]
        [MemberData(nameof(StatusCases))]
        public Task The_firmware_page_in_preferences_is_whole_at_both_sizes(bool bigScreen, int width, int height) => Session.Dispatch(() =>
        {
            string name = $"PreferencesFirmware-{(bigScreen ? "BigScreen" : "Desktop")}-{width}x{height}";
            (MainWindow window, PadDriver pad) = InGame(width, height, bigScreen: bigScreen, more: a => a.FirmwareDirectory = ChosenFirmware);
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
                IReadOnlyList<FirmwareRow> items = FirmwareView.Current.Systems.SelectMany(s => s.Items).ToList();
                Assert.Contains(items, i => i.FoundIn == FirmwareItem.YourFolder);
                Assert.Contains(items, i => i.FoundIn == FirmwareItem.EmuSensFolder);
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
                    Assert.Equal(ChosenFirmware, root.GetVisualDescendants().OfType<PathPickerRow>().Single(p => p.Name == "FirmwareDirectoryBox").Path);
                    // The folder row's box, Browse and Clear are reached by the d-pad, as every control of the page is.
                    root.GetVisualDescendants().OfType<TabItem>().Single(t => t.IsSelected).Focus(NavigationMethod.Directional);
                    Settle(top);
                    HashSet<InputElement> reached = PadAudit.Reachable(root, pad);
                    Assert.All(root.GetVisualDescendants().OfType<PathPickerRow>().Single(p => p.Name == "FirmwareDirectoryBox").GetVisualDescendants().OfType<InputElement>().Where(e => e is TextBox or Button), e => Assert.Contains(e, reached));
                    SavePicture(top, name);
                }
                else
                {
                    // Every row with the focus on it, since the footer's words are the focused row's.
                    Assert.Equal(PreferencesWindow.FirmwareTab, preferences.Form!.Menu.Title);
                    List<Control> rows = root.GetVisualDescendants().OfType<StackPanel>().Single(p => p.Name == "BigMenuRows").Children.Where(c => c is Button or PathPickerRow).ToList();
                    Assert.Equal(new[] { FirmwarePane.FolderLabel, FirmwarePane.ChosenLabel, FirmwarePane.FolderRowLabel }.Concat(FirmwareView.Current.Systems.SelectMany(s => s.Items.Count == 0 ? [s.Name] : s.Items.Select(FirmwarePane.MenuLabel))), rows.Select(MenuLabelOf));
                    var picker = Assert.IsType<PathPickerRow>(rows[1]);
                    Assert.Equal(ChosenFirmware, picker.Path);
                    // A file's name and EmuSen's folder's are drawn in their own case, since a player types them as they read; the look's case is for words.
                    Assert.All(rows.Where(r => items.Any(i => MenuRows.GetValue(r) == i.FileName)).Append(rows[2]), r => Assert.Equal(EmuSen.LunaP.Media.LetterCase.None, MenuRows.GetValueLetterCase(r)));
                    Assert.Equal(items.Count, rows.Count(r => items.Any(i => MenuRows.GetValue(r) == i.FileName)));
                    Assert.Equal(["Back", "Choose"], preferences.Form.Menu.Hints!.Select(h => h.Label));
                    Assert.DoesNotContain(root.GetVisualDescendants().OfType<Button>(), b => (b.Content as string) == FirmwarePane.OpenFolder);
                    for (int i = 0; i < rows.Count; i++)
                    {
                        Settle(window);
                        var focused = (Control)window.FocusManager!.GetFocusedElement()!;
                        Assert.True(ReferenceEquals(rows[i], focused) || focused.FindAncestorOfType<PathPickerRow>() == rows[i], $"{name}: row {i} does not have the focus");
                        Assert.False(string.IsNullOrEmpty(preferences.Form.Menu.Footer), $"{name}: no footer on row {i}");
                        if (i == 2) Assert.Equal((FirmwareView.Current.Folder, EmuSen.LunaP.Media.LetterCase.None), (preferences.Form.Menu.Footer, preferences.Form.Menu.FooterLetterCase));
                        else Assert.Null(preferences.Form.Menu.FooterLetterCase);
                        // Select joins the hints only on the folder's row, the one row of the page that takes a choice.
                        Assert.Equal(i == 1 ? ["Select", "Back", "Choose"] : ["Back", "Choose"], preferences.Form.Menu.Hints!.Select(h => h.Label));
                        if (i == 1) Assert.Equal(FirmwarePane.ChosenHint, preferences.Form.Menu.Footer);
                        if (preferences.Form.Menu.IsFooterCut) faults.Add($"row {i} ({MenuRows.GetLabel(rows[i])}): the footer is cut: {preferences.Form.Menu.Footer}");
                        foreach (string f in FitAudit.Check(root, WindowAllowances.For(top))) faults.Add($"row {i} ({MenuRows.GetLabel(rows[i])}): {f}");
                        if (i is 0 or 1 or 2 || i == rows.Count - 1 || items.Any(it => (it.WrongSize || it.Present) && FirmwarePane.MenuLabel(it) == MenuRows.GetLabel(rows[i]))) SavePicture(window, $"{name}-Row{i:D2}");
                        pad.Down();
                    }
                    // Every row and Back is reached by the d-pad, and B goes back to the tabs, then closes the sheet.
                    HashSet<InputElement> reached = PadAudit.Reachable(root, pad);
                    Assert.All(PadAudit.Operable(root), e => Assert.Contains(e, reached));
                    pad.B();
                    Assert.Equal("Preferences", preferences.Form.Menu.Title);
                    Assert.Equal(["Select", "Back", "Change", "Choose"], preferences.Form.Menu.Hints!.Select(h => h.Label));
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
