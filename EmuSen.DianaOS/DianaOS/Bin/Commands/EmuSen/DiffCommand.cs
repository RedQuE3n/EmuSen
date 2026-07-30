using System.Collections.Generic;
using System.Linq;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Compares a saved snapshot (see SnapshotCommand, which shares this
    // command's SnapshotStore) against that same space's CURRENT
    // contents and reports every address that's different now - doesn't
    // touch or replace the saved snapshot, so the same baseline can be
    // diffed again later against a further-along state.
    public class DiffCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
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

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
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
