using System.Collections.Generic;
using System.Linq;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // Compares a saved snapshot (see SnapshotCommand, which shares this
    // command's SnapshotStore) against that same space's CURRENT
    // contents and reports every address that's different now - doesn't
    // touch or replace the saved snapshot, so the same baseline can be
    // diffed again later against a further-along state.
    public class DiffCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "diff";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  diff <name> [<count>]         compare snapshot <name> against that space's CURRENT contents,",
            "                                print addresses that changed (default 20 shown)",
        });

        private readonly SnapshotStore _store;

        public DiffCommand(SnapshotStore store)
        {
            _store = store;
        }

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2) return "Usage: diff <name> [<count>]";
            if (!_store.Snapshots.TryGetValue(parts[1], out var snap))
            {
                return $"No snapshot named '{parts[1]}'. Try 'snapshot list'.";
            }

            IDebugMemorySpace space = FindSpace(target, snap.SpaceName);
            int count = parts.Length >= 3 ? ParseHex(parts[2]) : 20;

            var changes = new List<(int Addr, byte Old, byte New)>();
            for (int addr = 0; addr < snap.Data.Length; addr++)
            {
                byte now = space.Read(addr);
                if (now != snap.Data[addr]) changes.Add((addr, snap.Data[addr], now));
            }

            if (changes.Count == 0) return $"No differences from snapshot '{parts[1]}'.";

            var shown = changes.Take(count).Select(c => $"  0x{c.Addr:X}: 0x{c.Old:X2} -> 0x{c.New:X2}");
            return $"{changes.Count} byte(s) differ from snapshot '{parts[1]}' ({snap.SpaceName}), showing up to {count}:\n" + string.Join('\n', shown);
        }
    }
}
