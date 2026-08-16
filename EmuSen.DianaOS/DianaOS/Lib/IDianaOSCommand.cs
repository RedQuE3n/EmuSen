using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Lib
{
    // One interface for debug commands and text filters alike - see EmuSen_Debugging_Tools_Reference_v5.md §3.3.
    public interface IDianaOSCommand
    {
        // Matched case-insensitively by the registry.
        string Name { get; }

        // Per-class, so a mixed read/write sub-verb surface must report false - see §3.3b.
        bool IsReadOnly { get; }

        // Formatted to match the rest of `help`'s output; may be several lines.
        string Usage { get; }

        // A null target is a normal "no ROM loaded" condition; null stdin is not empty stdin - see §3.3b.
        DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin);
    }
}
