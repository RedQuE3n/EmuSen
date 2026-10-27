using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.LogicalTree;
using EmuSen.LunaP.Windowing;
using EmuSen.Mistress.Views;
using Xunit;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // A question asked in a big-screen session is the menu's message box over the themed list, never a desktop dialog - see EmuSen_BigPicture.md §34.2a.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedQuestionsTests
    {
        [Fact]
        public Task Scrape_this_game_asks_in_the_menus_message_box() => ThemedLibraryPadTests.Run(s =>
        {
            ThemedLibraryPadTests.Enter(s, "snes");
            var confirm = (System.Func<MainWindow, string, string, Task<bool>>)typeof(MainWindow).GetField("ConfirmScrape", BindingFlags.Static | BindingFlags.NonPublic)!.GetValue(null)!;
            Task<bool> asked = confirm(s.Window, "Scrape This Game", "One game, ScreenScraper first.");
            s.Settle();
            SheetLayer sheets = ThemedGameOptionsTests.Sheets(s);
            Assert.Equal("DialogWindow", sheets.Current!.GetType().Name);
            Assert.True(SheetLayer.GetChromeless(sheets.Current!));
            Assert.Equal("Scrape", ThemedGameOptionsTests.Sheet(s).GetLogicalDescendants().OfType<Avalonia.Controls.Button>().First(b => b.Name == "PART_Accept").Content);
            sheets.Current!.Close(false);
            s.Settle();
            Assert.False(asked.Result);
        });
    }
}
