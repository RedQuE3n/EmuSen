using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia.Threading;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Input;

namespace EmuSen.Mistress.Views
{
    // Change Disc as a page of the Pause Card: the game's own discs, the console's other games, the tray, and on the desktop any file - see EmuSen_Interface.md §18.5 and EmuSen_Settings_Reference.md §4.106.
    public partial class MainWindow
    {
        public const string ChangeDiscMenu = "Change Disc", OpenTrayRow = "Open the tray", AnyFileRow = "Any file…", InsertedValue = "Inserted";
        internal const string DiscSentence = "A opens the tray, swaps the disc and closes it, as the console sees it.";

        // The first page's row, for a running machine with a drive.
        private IEnumerable<PadMenuEntry> DiscRows()
        {
            if (DriveEngine is null) yield break;
            yield return new PadMenuEntry(() => ChangeDiscMenu, OpenChangeDiscPage, closes: false) { Opens = true, TurnsPage = true };
        }

        private void OpenChangeDiscPage()
        {
            if (DriveEngine is null) return;
            EnterPadMenuPage(new PadMenuPage(ChangeDiscMenu, ChangeDiscRows) { AcceptWord = "Put in", StartResumes = false }, Math.Max(PadMenuList.SelectedIndex, 0), key: null);
        }

        // The menu bar's Change Disc: the Pause Card opened on its page.
        private void OpenChangeDisc()
        {
            if (!GameOnScreen || DriveEngine is null) return;
            if (!_padMenuOpen) OpenPadMenu();
            if (_padMenuPages.Count != 1 || !_padMenuInGame) return;
            PadMenuList.SelectedIndex = Math.Max(0, _padMenuEntries.FindIndex(e => e.Text() == ChangeDiscMenu));
            OpenChangeDiscPage();
        }

        // The game's own discs, the one in the drive marked; then the console's other games; then the tray, and any file on the desktop.
        private List<PadMenuEntry> ChangeDiscRows()
        {
            var rows = new List<PadMenuEntry>();
            if (DriveEngine is not { } session) return rows;
            (int count, int index, _, bool lidOpen) = session.Discs;
            string? own = _currentRomPath;
            if (own is not null)
            {
                string title = RunningGameTitle();
                for (int i = 0; i < Math.Max(1, count); i++)
                {
                    int disc = i;
                    bool inserted = disc == index && !lidOpen;
                    string name = count > 1 ? $"{title} (Disc {disc + 1})" : title;
                    string said = count > 1 ? $"{title}, Disc {disc + 1} of {count}" : title;
                    rows.Add(new PadMenuEntry(() => name, () => { if (!inserted) ChangeDisc(disc); })
                    {
                        Label = () => name, Value = () => inserted ? InsertedValue : null,
                        PaneBeside = () => (CoverForDisc(own), said, inserted ? "In the console now. B goes back." : DiscSentence),
                    });
                }
            }
            foreach ((string title, string path) in DiscsFor(_activeConsole).Where(d => !string.Equals(d.Path, own, StringComparison.Ordinal)))
                rows.Add(new PadMenuEntry(() => title, () => InsertDisc(path)) { PaneBeside = () => (CoverForDisc(path), title, DiscSentence) });
            if (index >= 0)
                rows.Add(new PadMenuEntry(() => OpenTrayRow, EjectDisc) { PaneSentences = () => (["A opens the tray and leaves it open, with no disc in, as the console sees it."], null) });
            if (!_bigScreen) rows.Add(new PadMenuEntry(() => AnyFileRow, () => PickDiscFileAsync().Logged("mistress", "PickDiscFileAsync"), closes: false) { PaneSentences = () => (["A chooses a disc image anywhere on this computer, and puts it in."], null) });
            return rows;
        }

        // A game's cover as the library draws it, its own picture or the generated card.
        private CoverItem CoverForDisc(string path) => ((ICollectionsHost)this).CoverFor(path);

        // At the next frame boundary, as the runtime applies a state's load, so no frame runs half through the change.
        private void ChangeDisc(int disc)
        {
            if (DriveEngine is not { } session) return;
            System.Threading.Tasks.Task.Run(() =>
            {
                string status;
                try
                {
                    session.Crystal.ChangeDisc((uint)disc);
                    status = $"Disc {disc + 1} is in";
                }
                catch (CrystalException ex)
                {
                    status = $"Change Disc failed: {ex.Message}";
                }
                Dispatcher.UIThread.Post(() => { StatusText.Text = status; Notify(status); });
            });
        }
    }
}
