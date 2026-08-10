using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Galaxia.Models;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.LunaP
{
    // Windows must USE the toolkit's controls rather than rebuild them - see EmuSen_LunaP.md §7.2.
    //
    // This is not an accessibility test, though accessibility is how the problem surfaced.
    // CheatDatabaseWindow spelled out a read-only TextBox beside a "Browse..." Button, which is
    // exactly what luna:PathPickerRow is, and PreferencesWindow used the real control three times.
    // The two rows were indistinguishable on screen. They stopped being indistinguishable the
    // moment LunaP 0.5.0 taught PathPickerRow to name its own parts: the real one announced itself
    // and the copy stayed silent, in a window whose other nine tab stops were fine.
    //
    // THE GENERAL POINT IS THE ONE §6.1 MADE ABOUT THE FRAME HAND-OFF. Three byte-identical copies
    // of a mechanism meant one bug in three places and nowhere to fix it once. A copy of a control
    // is the same trade in slower motion: it does not go wrong on the day it is written, it goes
    // wrong the day the original learns something and the copy does not. Naming the copy by hand
    // would have closed the visible gap and kept the thing that produced it.
    public class HandRolledControlTests
    {
        private static Window[] Windows() => new Window[]
        {
            new CheatDatabaseWindow(),
            new PreferencesWindow(new AppSettings()),
            new RomBrowserWindow(),
            new ActiveCheatsWindow(),
            new MainWindow(),
        };

        // A "Browse..." button that is not a part of a PathPickerRow is a picker row somebody built
        // by hand. Checking the TEMPLATED PARENT rather than counting buttons is what makes this
        // specific: the real control's button is a template part and reports the row as its parent,
        // a hand-rolled one reports nothing.
        [Fact]
        public Task No_window_rebuilds_a_path_picker_by_hand() => UiTest.Run(() =>
        {
            var handRolled = new List<string>();

            foreach (Window window in Windows())
            {
                window.Show();
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();
                window.Measure(new Size(1200, 800));
                window.Arrange(new Rect(0, 0, 1200, 800));
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();

                foreach (Button button in window.GetVisualDescendants().OfType<Button>())
                {
                    if (button.Content is not string caption) continue;
                    if (!caption.StartsWith("Browse", StringComparison.OrdinalIgnoreCase)) continue;
                    if (button.TemplatedParent is PathPickerRow) continue;

                    handRolled.Add($"{window.GetType().Name}: a '{caption}' button outside a PathPickerRow");
                }

                window.Close();
            }

            Assert.True(handRolled.Count == 0,
                "hand-rolled path pickers: " + string.Join("; ", handRolled));
        });

        // And the control is genuinely in use, so the guard above cannot pass by there being no
        // path rows anywhere at all - the empty-subject failure Pegasus_Design.md §13.5 records.
        [Fact]
        public Task The_windows_that_pick_a_folder_use_the_toolkit_control() => UiTest.Run(() =>
        {
            var counts = new Dictionary<string, int>();

            foreach (Window window in new Window[] { new CheatDatabaseWindow(), new PreferencesWindow(new AppSettings()) })
            {
                window.Show();
                Avalonia.Threading.Dispatcher.UIThread.RunJobs();

                counts[window.GetType().Name] = window.GetVisualDescendants().OfType<PathPickerRow>().Count();
                window.Close();
            }

            Assert.Equal(1, counts[nameof(CheatDatabaseWindow)]);
            Assert.Equal(3, counts[nameof(PreferencesWindow)]);
        });
    }
}
