using System.Linq;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // A named baseline for `diff`, which needs no candidate addresses first - see §3.10.
    public class SnapshotCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
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

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
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

            // Otherwise parts[1] is a space and parts[2] the name to save under.
            if (parts.Length < 3) return "Usage: snapshot <space> <name>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);

            // The same HasSideEffects refusal as `search` - see §3.1a.
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
