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
    // First scan, then narrow - see §3.9.
    public class SearchCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
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

        // One session at a time is what the command's own UX implies - see §3.9.
        private string? _searchSpace;
        private int _searchWidth = 1;
        private List<int>? _searchCandidates;
        private Dictionary<int, long>? _searchLastValues;

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
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
                // Or a width-4 value with the high bit set parses negative and never matches.
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

            // Otherwise parts[1] is a space name, replacing whatever search was active.
            if (parts.Length < 3) return "Usage: search <space> <value> [<width>]";
            IDebugMemorySpace newSpace = FindSpace(target, parts[1]);

            // A bulk read is exactly what HasSideEffects exists to refuse - see §3.1a.
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
