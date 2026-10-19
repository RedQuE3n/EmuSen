using EmuSen.Crystal.Debugger;

namespace EmuSen.WiseMan.Crystal
{
    // The scripts' verbs over a machine - see EmuSen_Backend.md §29.30.
    public sealed partial class CrystalDebugScript : IDisposable
    {
        private string? MachineAnswer(string verb, string rest, string[] words) => null;

        // The logging switch is the process's, so a script leaves it off.
        public void Dispose() => DebugLogging.MasterEnabled = false;
    }
}
