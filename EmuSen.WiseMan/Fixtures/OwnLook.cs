using System.Linq;
using Avalonia.Controls;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Fixtures
{
    // A game played the way the player plays it from the library: on the big screen's own look, Games from Home and A on its cover; on the desktop, the list's row - see EmuSen_BigPicture.md §51.
    public static class OwnLook
    {
        public static void Play(MainWindow window, PadDriver pad, string? path = null, int index = 0, bool swapped = false)
        {
            Avalonia.Threading.Dispatcher.UIThread.RunJobs();
            if (window.HomeLayout is null && window.GamesLayout is null)
            {
                if (path is not null) Select(window, path);
                else window.GetControl<ListBox>("LibraryList").SelectedIndex = index;
                Accept(pad, swapped);
                return;
            }
            OpenGames(window, pad, swapped);
            GalleryLayout games = window.GamesLayout!;
            games.FocusOn(path ?? games.Items[index].Key);
            Accept(pad, swapped);
        }

        // The desktop list's row for a file.
        public static void Select(MainWindow window, string path)
        {
            var list = window.GetControl<LunaList<EmuSen.Mistress.Library.RomEntry>>("LibraryList");
            EmuSen.Mistress.Library.RomEntry entry = list.Models.FirstOrDefault(e => e.FullPath == path)
                ?? throw new System.InvalidOperationException($"{path} is not in the library's list of {list.Models.Count}: {string.Join(", ", list.Models.Select(e => e.FullPath))}");
            list.Select(entry);
        }

        // From Home to the grid by the place and All games on Choose a console, as the player goes.
        public static void OpenGames(MainWindow window, PadDriver pad, bool swapped = false)
        {
            if (window.GamesLayout is not null) return;
            if (window.ConsolesLayout is null)
            {
                HubLayout home = window.HomeLayout!;
                home.FocusOn(HubRow.Places, "games");
                Accept(pad, swapped);
            }
            window.ConsolesLayout!.FocusOn(EmuSen.Mistress.Pages.ConsolesScreen.All);
            Accept(pad, swapped);
        }

        // A, or B where the player swapped the two.
        private static void Accept(PadDriver pad, bool swapped)
        {
            if (swapped) pad.B();
            else pad.A();
        }
    }
}
