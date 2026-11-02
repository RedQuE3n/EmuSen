using System.Runtime.CompilerServices;

namespace EmuSen.WiseMan.Mistress
{
    // Every window the suite opens reads no tool counts and asks for them in place, so the Home card's lines are the same on every run; a test that wants counts answers them - see EmuSen_BigPicture.md §49.4.
    internal static class NoToolCounts
    {
        [ModuleInitializer]
        internal static void Quiet() => Fixtures.ToolCounts.Quiet();
    }
}
