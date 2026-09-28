using System.Collections.Generic;
using System.Linq;
using Avalonia.Controls;
using Avalonia.VisualTree;
using EmuSen.LunaP.Controls;
using EmuSen.Mistress.Views;

namespace EmuSen.WiseMan.Fixtures
{
    // The cuts the fit audit accepts per window, each because its full text is reachable another way - see EmuSen_Settings_Reference.md §4.81.
    public static class WindowAllowances
    {
        private static bool In(Control c, string name) => c.GetVisualAncestors().OfType<Control>().Any(a => a.Name == name);

        public static readonly FitAudit.Allowance CheatCells = new(
            "a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen",
            c => c is TextBlock && In(c, "CheatsList"));

        public static readonly FitAudit.Allowance RecentGames = new(
            "a recent game's line cut in the list is whole in the panel's footer while its row is chosen",
            c => c is TextBlock && In(c, "ScrapeStatusRecent"));

        public static readonly FitAudit.Allowance ReelTiles = new(
            "the rewind reel's strip scrolls sideways by design, and Left and Right step the pad through every moment",
            c => c.FindAncestorOfType<TileStrip<EmuSen.Mistress.Views.ReelMoment>>() is not null);

        public static IReadOnlyList<FitAudit.Allowance> For(Window window) => window switch
        {
            ActiveCheatsWindow => [FitAudit.TextBoxesScroll, CheatCells],
            ScrapeStatusWindow => [FitAudit.TextBoxesScroll, RecentGames],
            RewindReelWindow => [FitAudit.TextBoxesScroll, ReelTiles],
            _ => [FitAudit.TextBoxesScroll],
        };

        // Every allowance, for the record of what the audit accepts.
        public static IReadOnlyList<FitAudit.Allowance> All => [FitAudit.TextBoxesScroll, CheatCells, RecentGames, ReelTiles];
    }
}
