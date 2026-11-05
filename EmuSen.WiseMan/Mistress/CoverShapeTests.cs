using System;
using System.Linq;
using System.Reflection;
using System.Threading.Tasks;
using Avalonia.Headless;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Library;
using EmuSen.Mistress.Views;
using Xunit;

namespace EmuSen.WiseMan.Mistress
{
    // A game with no art is a generated card in its console's real box shape, read from the systems table as height over width - see EmuSen_Interface.md §21.
    public class CoverShapeTests
    {
        private static readonly HeadlessUnitTestSession Session =
            HeadlessUnitTestSession.GetOrStartForAssembly(typeof(CoverShapeTests).GetTypeInfo().Assembly);

        // Each system's North American box: a tall case or long box, a landscape box, or a near-square box or jewel case.
        public static TheoryData<string, string> Boxes() => new()
        {
            { "nes", "tall" }, { "snes", "landscape" }, { "n64", "landscape" }, { "gb", "square" }, { "gbc", "square" },
            { "sg1000", "tall" }, { "sms", "tall" }, { "md", "tall" }, { "mcd", "tall" }, { "32x", "tall" },
            { "psx", "square" }, { "ps2", "tall" }, { "saturn", "tall" }, { "dreamcast", "square" },
        };

        [Theory]
        [MemberData(nameof(Boxes))]
        public Task A_generated_card_stands_in_its_console_s_box_shape(string system, string box) => Session.Dispatch(() =>
        {
            SystemFacts facts = RuntimeCatalog.Systems.FirstOrDefault(s => s.Id == system) ?? throw new Xunit.Sdk.XunitException($"{system} is not in the systems table");
            double table = facts.CoverAspect ?? throw new Xunit.Sdk.XunitException($"{system} has no cover_aspect");
            var item = (CoverItem)typeof(MainWindow).GetMethod("CoverOf", BindingFlags.Static | BindingFlags.NonPublic)!.Invoke(null, ["g.rom", "Fable of Tiles", "MD", table, false, "", "", null])!;
            var card = new CoverCard { CellWidth = 180, CellHeight = 240, Item = item };
            double drawn = card.CoverBounds.Height / card.CoverBounds.Width;
            Assert.Equal(table, drawn, 2);
            string seen = drawn > 1.1 ? "tall" : drawn < 0.9 ? "landscape" : "square";
            Assert.True(seen == box, $"{system}'s generated card is {seen} ({drawn:0.00} high per unit wide); its box is {box}");
        }, default);
    }
}
