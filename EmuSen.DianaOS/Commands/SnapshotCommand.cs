using System.Linq;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // Captures a named, non-mutating baseline of a memory space's full
    // current contents - the other half of the "what changed" workflow
    // SearchCommand's changed/unchanged/increased/decreased covers for a
    // fixed set of candidate addresses. DiffCommand (separate class,
    // sharing this one's SnapshotStore) is the general case: no need to
    // already know which addresses might be interesting, just "what's
    // different from before" over an entire space.
    public class SnapshotCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "snapshot";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  snapshot <space> <name>       capture <space>'s full current contents under <name>",
            "  snapshot list                 list saved snapshots (name, space, size)",
            "  snapshot remove <name>        delete a saved snapshot",
        });

        private readonly SnapshotStore _store;

        public SnapshotCommand(SnapshotStore store)
        {
            _store = store;
        }

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2)
            {
                return "Usage: snapshot <space> <name> | snapshot list | snapshot remove <name>";
            }

            string first = parts[1].ToLowerInvariant();

            if (first == "list")
            {
                if (_store.Snapshots.Count == 0) return "No saved snapshots.";
                return string.Join('\n', _store.Snapshots.Select(kv => $"  {kv.Key}: {kv.Value.SpaceName} ({kv.Value.Data.Length} bytes)"));
            }

            if (first == "remove")
            {
                if (parts.Length < 3) return "Usage: snapshot remove <name>";
                bool removed = _store.Snapshots.Remove(parts[2]);
                return removed ? $"Snapshot '{parts[2]}' removed." : $"No snapshot named '{parts[2]}'.";
            }

            // Otherwise: parts[1] is a space name, parts[2] is the name to
            // save this capture under.
            if (parts.Length < 3) return "Usage: snapshot <space> <name>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);

            // Same HasSideEffects guard as SearchCommand, same reasoning
            // - a full-space read-every-address capture of a live-
            // hardware-routed space (SNES's CpuBus) could disturb real
            // emulation state.
            if (space.HasSideEffects)
            {
                return $"{space.Name} can have real side effects on read (live hardware registers) - refusing a bulk snapshot there. Try WRAM (or another plain-memory space) instead.";
            }

            var data = new byte[space.Size];
            for (int addr = 0; addr < space.Size; addr++) data[addr] = space.Read(addr);

            string name = parts[2];
            _store.Snapshots[name] = (space.Name, data);
            return $"Snapshot '{name}' saved: {space.Name}, {data.Length} bytes.";
        }
    }
}
