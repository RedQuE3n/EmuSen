using System.Collections.Generic;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // How big a pass draws: the game's own picture, or the rectangle it is shown in - see EmuSen_Serenity.md §3.2.
    public enum PassScale { Source, Viewport }

    // One SkSL pass; History is how many earlier frames it reads, as history1 to historyN - see EmuSen_Serenity.md §3.2.
    public sealed record FilterPass(string Sksl, PassScale Scale, bool LinearSource = false, int History = 0);

    // A screen filter: its passes, the consoles it suits (null for every one), whose work it rests on, and the uniforms a player may set - see EmuSen_Serenity.md §3.3 and §3.7.
    public sealed record ScreenFilter(string Name, IReadOnlyList<FilterPass> Passes, IReadOnlyList<string>? Consoles, string Credit, IReadOnlyList<SlangParameter>? Parameters = null)
    {
        public int History
        {
            get
            {
                int most = 0;
                foreach (FilterPass pass in Passes) if (pass.History > most) most = pass.History;
                return most;
            }
        }
    }
}
