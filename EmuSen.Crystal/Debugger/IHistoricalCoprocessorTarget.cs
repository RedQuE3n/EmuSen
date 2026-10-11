using System.Collections.Generic;
using EmuSen.Crystal.Telemetry;

namespace EmuSen.Crystal.Debugger
{
    // Separate from IDebugTarget so a core without history carries no ring buffer - see §3.1a.
    public interface IHistoricalCoprocessorTarget
    {
        HistoryProvider<IReadOnlyList<DebugRegisterValue>> CoprocessorHistory { get; }
    }
}
