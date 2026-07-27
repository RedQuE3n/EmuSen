using System.Collections.Generic;
using System.Linq;
using static EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    // Classic "first scan, then narrow" memory search - find where a
    // game stores something without already knowing the address (score,
    // lives, a flag), the one common reverse-engineering workflow the
    // rest of this toolchain (mem/write/watch/tile) doesn't cover on its
    // own. Pure IDebugMemorySpace.Read() scans - no SNES-specific logic,
    // works the same for any future core's memory spaces.
    public class SearchCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "search";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  search <space> <val> [<w>]    start a new search: find every address currently equal to <val>",
            "                                (width <w> in bytes: 1, 2, or 4 - default 1, little-endian)",
            "  search refine <val>           narrow the current search to addresses now equal to <val>",
            "  search changed|unchanged      narrow to addresses whose value did/didn't change since last search/refine",
            "  search increased|decreased    narrow to addresses whose value went up/down since last search/refine",
            "  search list [<count>]         list current candidate addresses + values (default 20)",
            "  search reset                  clear the current search",
        });

        // A single active search session, the same "first scan, then
        // narrow with next scan" workflow classic memory-search tools
        // use. Deliberately just plain fields, not its own class: one
        // session at a time is exactly what the command's own UX
        // implies (starting a new `search <space> ...` replaces
        // whatever was active), so there's nothing a richer structure
        // would buy here. DianaOSInterpreter constructs this command
        // once and reuses the same instance for every call, so this
        // state persists across F4 prompt invocations exactly the way
        // it did as private fields directly on the processor before.
        private string? _searchSpace;
        private int _searchWidth = 1;
        private List<int>? _searchCandidates;
        private Dictionary<int, long>? _searchLastValues;

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 2)
            {
                return "Usage: search <space> <value> [<width>] | search refine <value> | search changed|unchanged|increased|decreased | search list [<count>] | search reset";
            }

            string first = parts[1].ToLowerInvariant();

            if (first == "reset")
            {
                _searchSpace = null;
                _searchCandidates = null;
                _searchLastValues = null;
                return "Search cleared.";
            }

            if (first == "list")
            {
                if (_searchCandidates == null) return "No active search - run 'search <space> <value>' first.";
                int listCount = parts.Length >= 3 ? ParseHex(parts[2]) : 20;
                IDebugMemorySpace listSpace = FindSpace(target, _searchSpace!);
                var shown = _searchCandidates.Take(listCount)
                    .Select(a => $"  0x{a:X} = 0x{ReadValue(listSpace, a, _searchWidth):X}");
                return $"{_searchCandidates.Count} candidate(s) in {_searchSpace} (width {_searchWidth}), showing up to {listCount}:\n" + string.Join('\n', shown);
            }

            if (first == "refine")
            {
                if (_searchCandidates == null) return "No active search - run 'search <space> <value>' first.";
                if (parts.Length < 3) return "Usage: search refine <value>";
                IDebugMemorySpace refineSpace = FindSpace(target, _searchSpace!);
                // & 0xFFFFFFFFL: ParseHex returns a signed int, so a
                // width-4 value with the high bit set (e.g. FFFFFFFF)
                // would otherwise parse as a negative number and never
                // match ReadValue's always-non-negative accumulation.
                long targetValue = ParseHex(parts[2]) & 0xFFFFFFFFL;
                _searchCandidates = _searchCandidates.Where(a => ReadValue(refineSpace, a, _searchWidth) == targetValue).ToList();
                UpdateSearchLastValues(refineSpace);
                return $"{_searchCandidates.Count} candidate(s) remain.";
            }

            if (first == "changed" || first == "unchanged" || first == "increased" || first == "decreased")
            {
                if (_searchCandidates == null) return "No active search - run 'search <space> <value>' first.";
                IDebugMemorySpace compareSpace = FindSpace(target, _searchSpace!);
                _searchCandidates = _searchCandidates.Where(a =>
                {
                    long now = ReadValue(compareSpace, a, _searchWidth);
                    long then = _searchLastValues![a];
                    return first switch
                    {
                        "changed" => now != then,
                        "unchanged" => now == then,
                        "increased" => now > then,
                        "decreased" => now < then,
                        _ => false,
                    };
                }).ToList();
                UpdateSearchLastValues(compareSpace);
                return $"{_searchCandidates.Count} candidate(s) remain.";
            }

            // Otherwise: parts[1] is a memory space name - start a brand
            // new search, replacing whatever was active before.
            if (parts.Length < 3) return "Usage: search <space> <value> [<width>]";
            IDebugMemorySpace newSpace = FindSpace(target, parts[1]);

            // A bulk, read-every-address scan is exactly the case
            // IDebugMemorySpace.HasSideEffects exists for - a space
            // routed through live hardware (the SNES's CpuBus) can have
            // registers that change real emulation state just by being
            // read (RDNMI clearing the pending-NMI flag, OPHCT/OPVCT
            // toggling a latch, the manual joypad port shifting on every
            // read). Refuse outright rather than silently corrupting a
            // running session - there's no legitimate reason to bulk-
            // search live registers anyway; real game state (scores,
            // flags, counters) lives in WRAM/SRAM, not there.
            if (newSpace.HasSideEffects)
            {
                return $"{newSpace.Name} can have real side effects on read (live hardware registers) - refusing a bulk search there. Try WRAM (or another plain-memory space) instead.";
            }

            // Same unsigned-mask reasoning as the 'refine' branch above.
            long value = ParseHex(parts[2]) & 0xFFFFFFFFL;
            int width = parts.Length >= 4 ? ParseHex(parts[3]) : 1;
            if (width != 1 && width != 2 && width != 4) return "width must be 1, 2, or 4";

            _searchSpace = newSpace.Name;
            _searchWidth = width;
            _searchCandidates = new List<int>();
            for (int addr = 0; addr <= newSpace.Size - width; addr++)
            {
                if (ReadValue(newSpace, addr, width) == value) _searchCandidates.Add(addr);
            }
            UpdateSearchLastValues(newSpace);
            return $"{_searchCandidates.Count} candidate(s) found in {newSpace.Name} matching 0x{value:X} (width {width}).";
        }

        private void UpdateSearchLastValues(IDebugMemorySpace space)
        {
            _searchLastValues = _searchCandidates!.ToDictionary(a => a, a => ReadValue(space, a, _searchWidth));
        }
    }
}
