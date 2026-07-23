using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;

namespace EmuSen.Debug
{
    // Takes a line of text, returns a line of text. Deliberately knows
    // nothing about where that text came from or where it's going - same
    // way `xxd` doesn't care if its output lands in a terminal or a pipe.
    // That's what makes this reusable from a console-mode prompt today
    // (see Program.cs's F4 hotkey) and, later, from a GUI debug window's
    // command box or a future standalone headless CLI tool, without
    // rewriting anything here.
    //
    // Each command is small and single-purpose (list spaces, hexdump a
    // region, show registers, ...) rather than one big dump - the same
    // "small composable tools" shape as a Linux toolchain (ls/xxd/objdump)
    // instead of a single command that prints everything. Building blocks
    // for scripting later (piping one command's address output into
    // another) if that's ever useful, rather than a monolith.
    //
    // Deliberately NOT interactive in the sense of pausing emulation - see
    // IDebugTarget's comment on why breakpoints/stepping aren't here yet
    // (the execution loop can't pause mid-frame). Each call to Execute is
    // one-shot: runs against whatever the current state is, returns
    // immediately, execution continues exactly as it would without this
    // existing at all.
    public class DebugCommandProcessor
    {
        private readonly IDebugTarget _target;

        // `search` command state - a single active search session, the
        // same "first scan, then narrow with next scan" workflow classic
        // memory-search tools use. Deliberately just plain fields, not
        // its own class: one session at a time is exactly what the
        // command's own UX implies (starting a new `search <space> ...`
        // replaces whatever was active), so there's nothing a richer
        // structure would buy here.
        private string? _searchSpace;
        private int _searchWidth = 1;
        private List<int>? _searchCandidates;
        private Dictionary<int, long>? _searchLastValues;

        // `snapshot`/`diff` state - named, non-mutating baselines. Unlike
        // `search`'s single active session, there's real value in keeping
        // more than one around at once (e.g. "before Yoshi's block opens"
        // and "right after", compared independently later), so this is a
        // dictionary instead of a single set of fields.
        private readonly Dictionary<string, (string SpaceName, byte[] Data)> _snapshots = new();

        public DebugCommandProcessor(IDebugTarget target)
        {
            _target = target;
        }

        public string Execute(string commandLine)
        {
            string[] parts = (commandLine ?? "").Trim().Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries);
            if (parts.Length == 0) return "";

            string cmd = parts[0].ToLowerInvariant();
            try
            {
                return cmd switch
                {
                    "help" => Help(),
                    "spaces" => CmdSpaces(),
                    "mem" => CmdMem(parts),
                    "write" => CmdWrite(parts),
                    "regs" => CmdRegs(),
                    "sprites" => CmdSprites(),
                    "pal" => CmdPal(parts),
                    "watch" => CmdWatch(parts),
                    "search" => CmdSearch(parts),
                    "snapshot" => CmdSnapshot(parts),
                    "diff" => CmdDiff(parts),
                    "tile" => CmdTile(parts),
                    "disasm" => CmdDisasm(parts),
                    "trace" => CmdTrace(parts),
                    "summary" => _target.GetSummaryText(),
                    _ => $"Unknown command '{cmd}'. Type 'help' for a list.",
                };
            }
            catch (Exception ex)
            {
                // A malformed command (bad address, out-of-range space,
                // etc.) should produce a readable error and nothing else -
                // never take down whatever's running this prompt.
                return $"Error: {ex.Message}";
            }
        }

        private static string Help()
        {
            return string.Join('\n', new[]
            {
                "Available commands:",
                "  help                          this text",
                "  spaces                        list available memory spaces",
                "  mem <space> <addr> [<len>]    hexdump <len> bytes (default 16) from <space> at <addr>",
                "  write <space> <addr> <value>  write one byte (only if <space> is writable)",
                "  regs                          CPU + video registers",
                "  sprites                       active sprite/OBJ table",
                "  pal [<index>]                 one palette, or all of them if omitted",
                "  watch add <space> <addr> <len> register a watch, prints matching writes live + stores them",
                "  watch list                    list active watches with their IDs",
                "  watch log <id> [<count>]      show a watch's recorded events (default 20)",
                "  watch clear <id>              clear a watch's stored events (doesn't remove the watch)",
                "  watch remove <id>             remove a watch entirely",
                "  search <space> <val> [<w>]    start a new search: find every address currently equal to <val>",
                "                                (width <w> in bytes: 1, 2, or 4 - default 1, little-endian)",
                "  search refine <val>           narrow the current search to addresses now equal to <val>",
                "  search changed|unchanged      narrow to addresses whose value did/didn't change since last search/refine",
                "  search increased|decreased    narrow to addresses whose value went up/down since last search/refine",
                "  search list [<count>]         list current candidate addresses + values (default 20)",
                "  search reset                  clear the current search",
                "  snapshot <space> <name>       capture <space>'s full current contents under <name>",
                "  snapshot list                 list saved snapshots (name, space, size)",
                "  snapshot remove <name>        delete a saved snapshot",
                "  diff <name> [<count>]         compare snapshot <name> against that space's CURRENT contents,",
                "                                print addresses that changed (default 20 shown)",
                "  tile <space> <addr> <bpp>     ASCII-decode one 8x8 tile (bpp: 2, 4, or 8)",
                "  disasm <space> <addr> [<n>]   disassemble <n> instructions (default 10)",
                "  trace <count>                 arm a live CPU instruction trace for the next <count> instructions",
                "  trace off                     cancel an in-progress trace early",
                "  summary                       free-text state dump (whatever isn't structured above yet)",
                "",
                "Addresses/values are hex; an optional 0x or $ prefix is fine either way.",
            });
        }

        private static int ParseHex(string s)
        {
            s = s.Trim();
            if (s.StartsWith("0x", StringComparison.OrdinalIgnoreCase)) s = s.Substring(2);
            else if (s.StartsWith("$")) s = s.Substring(1);
            return Convert.ToInt32(s, 16);
        }

        private IDebugMemorySpace FindSpace(string name)
        {
            var spaces = _target.GetMemorySpaces();
            var match = spaces.FirstOrDefault(s => string.Equals(s.Name, name, StringComparison.OrdinalIgnoreCase));
            if (match == null)
            {
                string available = string.Join(", ", spaces.Select(s => s.Name));
                throw new ArgumentException($"No memory space named '{name}'. Available: {available}");
            }
            return match;
        }

        private string CmdSpaces()
        {
            var sb = new StringBuilder();
            sb.AppendLine($"{_target.CoreName} memory spaces:");
            foreach (var s in _target.GetMemorySpaces())
            {
                sb.AppendLine($"  {s.Name,-10} {s.Size,8} bytes  {(s.IsWritable ? "R/W" : "R/O")}");
            }
            return sb.ToString().TrimEnd();
        }

        private string CmdMem(string[] parts)
        {
            if (parts.Length < 3) return "Usage: mem <space> <addr> [<len>]";
            IDebugMemorySpace space = FindSpace(parts[1]);
            int addr = ParseHex(parts[2]);
            int len = parts.Length >= 4 ? ParseHex(parts[3]) : 16;

            var sb = new StringBuilder();
            sb.AppendLine($"{space.Name} @ 0x{addr:X} ({len} bytes):");
            for (int row = 0; row < len; row += 16)
            {
                sb.Append($"  {addr + row:X6}: ");
                var ascii = new StringBuilder();
                for (int col = 0; col < 16; col++)
                {
                    if (row + col < len)
                    {
                        byte b = space.Read(addr + row + col);
                        sb.Append($"{b:X2} ");
                        ascii.Append(b >= 0x20 && b < 0x7F ? (char)b : '.');
                    }
                    else
                    {
                        sb.Append("   ");
                    }
                }
                sb.Append(' ').Append(ascii);
                sb.AppendLine();
            }
            return sb.ToString().TrimEnd();
        }

        private string CmdWrite(string[] parts)
        {
            if (parts.Length < 4) return "Usage: write <space> <addr> <value>";
            IDebugMemorySpace space = FindSpace(parts[1]);
            if (!space.IsWritable) return $"{space.Name} is read-only.";
            int addr = ParseHex(parts[2]);
            byte value = (byte)ParseHex(parts[3]);
            space.Write(addr, value);
            return $"{space.Name}[0x{addr:X}] = 0x{value:X2}";
        }

        private string CmdRegs()
        {
            var sb = new StringBuilder();
            sb.AppendLine($"{_target.CoreName} CPU registers:");
            foreach (var r in _target.GetCpuRegisters())
            {
                int digits = Math.Max(1, r.BitWidth / 4);
                sb.AppendLine($"  {r.Name,-4} = 0x{r.Value.ToString("X" + digits)}");
            }
            sb.AppendLine("Video registers:");
            foreach (var r in _target.GetVideoRegisters())
            {
                sb.AppendLine($"  {r.Name,-8} = 0x{r.Value:X2}");
            }
            return sb.ToString().TrimEnd();
        }

        private string CmdSprites()
        {
            var sprites = _target.GetSprites();
            var sb = new StringBuilder();
            sb.AppendLine($"{sprites.Count} active sprite(s):");
            foreach (var s in sprites)
            {
                sb.AppendLine($"  #{s.Index}: x={s.X} y={s.Y} {s.Width}x{s.Height} tile=0x{s.TileIndex:X3} pal={s.PaletteIndex} pri={s.Priority} flipX={s.FlipX} flipY={s.FlipY}");
            }
            return sb.ToString().TrimEnd();
        }

        private string CmdPal(string[] parts)
        {
            var palettes = _target.GetPalettes();
            var sb = new StringBuilder();

            IEnumerable<DebugPaletteInfo> toShow = palettes;
            if (parts.Length >= 2)
            {
                int idx = ParseHex(parts[1]);
                toShow = palettes.Where(p => p.Index == idx);
            }

            foreach (var p in toShow)
            {
                sb.Append($"pal {p.Index,2}: ");
                sb.AppendLine(string.Join(' ', p.Colors.Select(c => $"{c.r:X2}{c.g:X2}{c.b:X2}")));
            }
            return sb.ToString().TrimEnd();
        }

        private string CmdWatch(string[] parts)
        {
            if (parts.Length < 2) return "Usage: watch add|list|log|clear|remove ...";
            string sub = parts[1].ToLowerInvariant();
            var watches = _target.Watches;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 5) return "Usage: watch add <space> <addr> <len>";
                    FindSpace(parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    int len = ParseHex(parts[4]);
                    int id = watches.AddWatch(parts[2], addr, len);
                    return $"Watch #{id} added: {parts[2]} 0x{addr:X}-0x{addr + len - 1:X}";
                }
                case "list":
                {
                    var list = watches.GetWatches();
                    if (list.Count == 0) return "No active watches.";
                    return string.Join('\n', list.Select(w => $"  #{w.Id}: {w.SpaceName} 0x{w.StartAddress:X}-0x{w.StartAddress + w.Length - 1:X}"));
                }
                case "log":
                {
                    if (parts.Length < 3) return "Usage: watch log <id> [<count>]";
                    int id = ParseHex(parts[2]);
                    int count = parts.Length >= 4 ? ParseHex(parts[3]) : 20;
                    var events = watches.GetEvents(id, count);
                    if (events.Count == 0) return $"No events recorded for watch #{id} (or it doesn't exist).";
                    return string.Join('\n', events.Select(e => $"  [{e.Sequence}] 0x{e.Address:X} = 0x{e.Value:X2} ({e.Context})"));
                }
                case "clear":
                {
                    if (parts.Length < 3) return "Usage: watch clear <id>";
                    watches.ClearEvents(ParseHex(parts[2]));
                    return $"Cleared events for watch #{parts[2]}.";
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: watch remove <id>";
                    bool removed = watches.RemoveWatch(ParseHex(parts[2]));
                    return removed ? $"Watch #{parts[2]} removed." : $"No watch #{parts[2]} found.";
                }
                default:
                    return $"Unknown 'watch' subcommand '{sub}'. Try add/list/log/clear/remove.";
            }
        }

        // Little-endian multi-byte read, matching the 65816/SNES
        // convention every other multi-byte value in this codebase uses
        // (see Snes65816Disassembler.FormatOperand's absolute-address
        // formatting for the same low-byte-first pattern). A future
        // core's memory spaces would read the same way for its own
        // little-endian values, or a core-specific command could read
        // multi-byte values its own way if it genuinely needed to -
        // nothing about `search` itself assumes little-endian beyond
        // this one helper.
        private static long ReadValue(IDebugMemorySpace space, int addr, int width)
        {
            long v = 0;
            for (int i = 0; i < width; i++) v |= (long)space.Read(addr + i) << (8 * i);
            return v;
        }

        private void UpdateSearchLastValues(IDebugMemorySpace space)
        {
            _searchLastValues = _searchCandidates!.ToDictionary(a => a, a => ReadValue(space, a, _searchWidth));
        }

        // Classic "first scan, then narrow" memory search - find where a
        // game stores something without already knowing the address
        // (score, lives, a flag), the one common reverse-engineering
        // workflow the rest of this toolchain (mem/write/watch/tile)
        // doesn't cover on its own. Pure IDebugMemorySpace.Read() scans -
        // no SNES-specific logic, works the same for any future core's
        // memory spaces.
        private string CmdSearch(string[] parts)
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
                IDebugMemorySpace listSpace = FindSpace(_searchSpace!);
                var shown = _searchCandidates.Take(listCount)
                    .Select(a => $"  0x{a:X} = 0x{ReadValue(listSpace, a, _searchWidth):X}");
                return $"{_searchCandidates.Count} candidate(s) in {_searchSpace} (width {_searchWidth}), showing up to {listCount}:\n" + string.Join('\n', shown);
            }

            if (first == "refine")
            {
                if (_searchCandidates == null) return "No active search - run 'search <space> <value>' first.";
                if (parts.Length < 3) return "Usage: search refine <value>";
                IDebugMemorySpace refineSpace = FindSpace(_searchSpace!);
                // & 0xFFFFFFFFL: ParseHex returns a signed int, so a
                // width-4 value with the high bit set (e.g. FFFFFFFF)
                // would otherwise parse as a negative number and never
                // match ReadValue's always-non-negative accumulation.
                long target = ParseHex(parts[2]) & 0xFFFFFFFFL;
                _searchCandidates = _searchCandidates.Where(a => ReadValue(refineSpace, a, _searchWidth) == target).ToList();
                UpdateSearchLastValues(refineSpace);
                return $"{_searchCandidates.Count} candidate(s) remain.";
            }

            if (first == "changed" || first == "unchanged" || first == "increased" || first == "decreased")
            {
                if (_searchCandidates == null) return "No active search - run 'search <space> <value>' first.";
                IDebugMemorySpace compareSpace = FindSpace(_searchSpace!);
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
            IDebugMemorySpace newSpace = FindSpace(parts[1]);

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

        // Captures a named, non-mutating baseline of a memory space's
        // full current contents - the other half of the "what changed"
        // workflow `search`'s changed/unchanged/increased/decreased
        // covers for a fixed set of candidate addresses. `diff` (below)
        // is the general case: no need to already know which addresses
        // might be interesting, just "what's different from before" over
        // an entire space. Same HasSideEffects guard as `search`, for
        // the same reason - a full-space read-every-address capture of a
        // live-hardware-routed space (SNES's CpuBus) could disturb real
        // emulation state.
        private string CmdSnapshot(string[] parts)
        {
            if (parts.Length < 2)
            {
                return "Usage: snapshot <space> <name> | snapshot list | snapshot remove <name>";
            }

            string first = parts[1].ToLowerInvariant();

            if (first == "list")
            {
                if (_snapshots.Count == 0) return "No saved snapshots.";
                return string.Join('\n', _snapshots.Select(kv => $"  {kv.Key}: {kv.Value.SpaceName} ({kv.Value.Data.Length} bytes)"));
            }

            if (first == "remove")
            {
                if (parts.Length < 3) return "Usage: snapshot remove <name>";
                bool removed = _snapshots.Remove(parts[2]);
                return removed ? $"Snapshot '{parts[2]}' removed." : $"No snapshot named '{parts[2]}'.";
            }

            // Otherwise: parts[1] is a space name, parts[2] is the name to
            // save this capture under.
            if (parts.Length < 3) return "Usage: snapshot <space> <name>";
            IDebugMemorySpace space = FindSpace(parts[1]);

            if (space.HasSideEffects)
            {
                return $"{space.Name} can have real side effects on read (live hardware registers) - refusing a bulk snapshot there. Try WRAM (or another plain-memory space) instead.";
            }

            var data = new byte[space.Size];
            for (int addr = 0; addr < space.Size; addr++) data[addr] = space.Read(addr);

            string name = parts[2];
            _snapshots[name] = (space.Name, data);
            return $"Snapshot '{name}' saved: {space.Name}, {data.Length} bytes.";
        }

        // Compares a saved snapshot against that same space's CURRENT
        // contents and reports every address that's different now -
        // doesn't touch or replace the saved snapshot, so the same
        // baseline can be diffed again later against a further-along
        // state (unlike `search`'s changed/unchanged, which narrows the
        // candidate set and moves its own baseline forward each time).
        private string CmdDiff(string[] parts)
        {
            if (parts.Length < 2) return "Usage: diff <name> [<count>]";
            if (!_snapshots.TryGetValue(parts[1], out var snap))
            {
                return $"No snapshot named '{parts[1]}'. Try 'snapshot list'.";
            }

            IDebugMemorySpace space = FindSpace(snap.SpaceName);
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

        // Generalizes DebugTools.DecodeTileAscii (which only ever worked
        // against a raw VRAM byte[]) into something that works against any
        // IDebugMemorySpace - so it's not tied to VRAM, or to SNES, or to
        // one hardcoded address the way CoinTileDumpLogging's Program.cs
        // call site was. Also extends bpp support to 8 (not needed when
        // DecodeTileAscii was written, but real now that Mode 3/4's 8bpp
        // BG1 and Direct Color are implemented) - same bitplane-pair-every-
        // 16-bytes layout as SampleBgPixel in Renderer.Backgrounds.cs.
        private string CmdTile(string[] parts)
        {
            if (parts.Length < 4) return "Usage: tile <space> <addr> <bpp>";
            IDebugMemorySpace space = FindSpace(parts[1]);
            int addr = ParseHex(parts[2]);
            int bpp = ParseHex(parts[3]);
            if (bpp != 2 && bpp != 4 && bpp != 8) return "bpp must be 2, 4, or 8";

            var sb = new StringBuilder();
            sb.AppendLine($"{space.Name} @ 0x{addr:X}, {bpp}bpp tile:");
            for (int row = 0; row < 8; row++)
            {
                byte p0 = space.Read(addr + row * 2);
                byte p1 = space.Read(addr + row * 2 + 1);
                byte p2 = 0, p3 = 0, p4 = 0, p5 = 0, p6 = 0, p7 = 0;
                if (bpp >= 4)
                {
                    p2 = space.Read(addr + 16 + row * 2);
                    p3 = space.Read(addr + 16 + row * 2 + 1);
                }
                if (bpp == 8)
                {
                    p4 = space.Read(addr + 32 + row * 2);
                    p5 = space.Read(addr + 32 + row * 2 + 1);
                    p6 = space.Read(addr + 48 + row * 2);
                    p7 = space.Read(addr + 48 + row * 2 + 1);
                }

                sb.Append("  ");
                for (int col = 0; col < 8; col++)
                {
                    int bit = 7 - col;
                    int val = ((p0 >> bit) & 1) | (((p1 >> bit) & 1) << 1);
                    if (bpp >= 4) val |= (((p2 >> bit) & 1) << 2) | (((p3 >> bit) & 1) << 3);
                    if (bpp == 8) val |= (((p4 >> bit) & 1) << 4) | (((p5 >> bit) & 1) << 5) | (((p6 >> bit) & 1) << 6) | (((p7 >> bit) & 1) << 7);
                    sb.Append(val == 0 ? (bpp == 8 ? ". " : ".") : val.ToString(bpp == 8 ? "X2" : "X1"));
                    sb.Append(' ');
                }
                sb.AppendLine();
            }
            return sb.ToString().TrimEnd();
        }

        private string CmdDisasm(string[] parts)
        {
            if (parts.Length < 3) return "Usage: disasm <space> <addr> [<count>]";
            FindSpace(parts[1]); // validates the space name, or throws a helpful error
            int addr = ParseHex(parts[2]);
            int count = parts.Length >= 4 ? ParseHex(parts[3]) : 10;

            var instructions = _target.Disassemble(parts[1], addr, count);
            if (instructions.Count == 0) return $"{_target.CoreName} target has no disassembler, or nothing was returned.";

            var sb = new StringBuilder();
            foreach (var instr in instructions)
            {
                string bytesHex = string.Join(' ', instr.Bytes.Select(b => b.ToString("X2")));
                sb.AppendLine($"  {instr.Address:X6}: {bytesHex,-9} {instr.Mnemonic} {instr.OperandText}".TrimEnd());
            }
            return sb.ToString().TrimEnd();
        }

        // Arms/disarms DebugSettings.CpuVerboseLogging live from the F4
        // prompt, instead of requiring a rebuild and a boot-time flag - the
        // countdown is instruction-count-based and starts from whenever
        // this command runs, not from power-on, so it can be aimed at a
        // specific moment in a play session (e.g. "right before the thing
        // I want to see happens") rather than burning its whole budget
        // during the boot/reset routine.
        private string CmdTrace(string[] parts)
        {
            if (parts.Length < 2) return "Usage: trace <count> | trace off";

            if (parts[1].Equals("off", StringComparison.OrdinalIgnoreCase))
            {
                DebugSettings.CpuVerboseLogging = false;
                DebugSettings.CpuTraceCountdown = 0;
                return "Trace disabled.";
            }

            int count = ParseHex(parts[1]);
            DebugSettings.CpuTraceCountdown = count;
            DebugSettings.CpuVerboseLogging = true;
            return $"Trace armed for the next {count} instructions.";
        }
    }
}
