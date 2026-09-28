using System.IO;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Mistress.BigPicture
{
    // The pad in a gamelist whose primary element is a grid: all four directions move it, the shoulders page by its whole rows, even after an up or down - see EmuSen_BigPicture.md §16.
    [Collection(TestCollections.ProcessGlobals)]
    public class ThemedGridPadTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(ThemedGridPadTests).GetTypeInfo().Assembly);

        // The session's theme with its list replaced by a grid of two columns and one whole row.
        private static SyntheticTheme GridTheme()
        {
            var theme = new SyntheticTheme();
            ThemedSession.Write(theme);
            string xml = File.ReadAllText(theme.PathOf("theme.xml"));
            int a = xml.IndexOf("<textlist name=\"gamelist\">"), b = xml.IndexOf("</textlist>") + "</textlist>".Length;
            xml = xml[..a] + "<grid name=\"gamelist\"><pos>0.05 0.08</pos><size>0.45 0.9</size><itemSize>0.2 0.4</itemSize><itemSpacing>0.01 0.01</itemSpacing>" +
                  "<itemScale>1</itemScale><imageType>cover</imageType><textColor>FFFFFF</textColor></grid>" + xml[b..];
            File.WriteAllText(theme.PathOf("theme.xml"), xml);
            return theme;
        }

        [Fact]
        public Task All_four_directions_move_the_grid_and_the_shoulders_change_the_system_or_page_by_the_quick_select() => Session.Dispatch(() =>
        {
            using SyntheticTheme theme = GridTheme();
            using var s = new ThemedSession(themeDirectory: theme.Root);
            ThemedLibraryPadTests.Enter(s, "snes");
            Assert.Equal(0, s.Themed.Stage!.Current.Index);
            Assert.Equal(2, s.Themed.Stage.Current.Grid()!.Value.Columns);
            Assert.Equal(2, s.Themed.Stage.Current.Grid()!.Value.WholeRows);

            s.Pad.Right();
            s.Run(300);
            Assert.Equal(("snes", 1), (s.System, s.Themed.Stage.Current.Index));
            s.Pad.Down();
            s.Run(300);
            Assert.Equal(3, s.Themed.Stage.Current.Index);
            s.Pad.Left();
            s.Run(300);
            Assert.Equal(2, s.Themed.Stage.Current.Index);
            s.Pad.Up();
            s.Run(300);
            Assert.Equal(0, s.Themed.Stage.Current.Index);
            Assert.Equal("snes", s.System);

            // ES-DE's default quick system select gives a grid's shoulders the system, since left and right move the grid (UG "UI settings", §29).
            s.Pad.R1();
            s.Run(300);
            Assert.NotEqual("snes", s.System);
            Assert.Contains("quicksysselect", s.Sounds);
            s.Pad.L1();
            s.Run(300);
            Assert.Equal("snes", s.System);

            // With left and right chosen for it, the shoulders page again, ten games at most.
            s.Themed.Interface = new EmuSen.Galaxia.Models.BigPictureInterface { QuickSystemSelect = EmuSen.Galaxia.Models.BigPictureInterface.QuickSelectLeftRight };
            s.Pad.R1();
            s.Run(300);
            Assert.Equal(("snes", 4), (s.System, s.Themed.Stage.Current.Index));
            Assert.Contains("scroll", s.Sounds);
        }, default);
    }
}
