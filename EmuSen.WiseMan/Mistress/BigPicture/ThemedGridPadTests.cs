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
        public Task All_four_directions_move_the_grid_and_the_shoulders_page_by_its_rows() => Session.Dispatch(() =>
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

            s.Pad.R1();
            s.Run(300);
            Assert.Equal(4, s.Themed.Stage.Current.Index);
            Assert.Contains("scroll", s.Sounds);
        }, default);
    }
}
