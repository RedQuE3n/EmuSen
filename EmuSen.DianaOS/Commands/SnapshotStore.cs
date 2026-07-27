using System.Collections.Generic;

namespace EmuSen.DianaOS.Commands
{
    // The shared state SnapshotCommand and DiffCommand both need -
    // separated into its own tiny class rather than living on either
    // command directly, since neither command "owns" saved snapshots
    // more than the other does (snapshot creates/lists/removes them,
    // diff only reads them). Both commands hold a reference to the same
    // instance, constructed once by DianaOSInterpreter - same
    // approach WatchRegistry already uses to let multiple things share
    // one piece of state without either owning the other.
    public class SnapshotStore
    {
        public readonly Dictionary<string, (string SpaceName, byte[] Data)> Snapshots = new();
    }
}
