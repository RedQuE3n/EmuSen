using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.Mistress.Views;
using EmuSen.WiseMan.Fixtures;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    public partial class WindowFitAuditTests
    {
        // The Cheat Database's games walked by the d-pad alone to the last, through the row the 1280 by 800 sheet shows 5 pixels of - see EmuSen_InterfaceSpec.md §6.2.
        [Theory]
        [InlineData(1280, 800)]
        [InlineData(1920, 1200)]
        public Task The_cheat_database_s_games_are_walked_to_the_last_by_the_pad(int width, int height) => Session.Dispatch(() =>
        {
            (MainWindow window, PadDriver pad) = InGame(width, height);
            try
            {
                OpenInGame(window, pad, "CheatDatabase");
                Settle(window);
                Control root = Sheets(window).SheetOf(Sheets(window).Current!)!;
                ListBox systems = root.GetVisualDescendants().OfType<ListBox>().First(l => l.Name == "SystemsList");
                ListBox games = root.GetVisualDescendants().OfType<ListBox>().First(l => l.Name == "GamesList");
                int Row(ListBox list) => window.FocusManager!.GetFocusedElement() is ListBoxItem r && ReferenceEquals(r.FindAncestorOfType<ListBox>(), list) ? list.IndexFromContainer(r) : -1;

                // From the folder box down to the Super Nintendo's row, across to its games, and down each.
                pad.Down(3);
                Settle(window);
                Assert.Equal(2, Row(systems));
                pad.Right();
                Settle(window);
                Assert.Equal(0, Row(games));
                Assert.Equal(4, games.ItemCount);
                for (int i = 1; i < games.ItemCount; i++)
                {
                    pad.Down();
                    Settle(window);
                    Assert.Equal(i, Row(games));
                    Assert.Equal(i, games.SelectedIndex);
                }
            }
            finally { Stop(window); window.Close(); }
        }, default);
    }
}
