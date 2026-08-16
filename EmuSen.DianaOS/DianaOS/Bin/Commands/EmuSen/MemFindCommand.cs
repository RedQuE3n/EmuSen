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
    // Finds a byte sequence, which `search`'s scalar match cannot - see §3.25.
    public class MemFindCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "memfind";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  memfind <space> <bytes> [<max>]  find every offset where <bytes> occurs (hex pairs, '??' matches",
            "                                any byte, separators optional) - default at most 20 hits",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 3) return "Usage: memfind <space> <bytes> [<max>]";
            IDebugMemorySpace space = FindSpace(target, parts[1]);

            // The same refusal as `search` and `dump` - see §3.1a.
            if (space.HasSideEffects)
            {
                return $"{space.Name} can have real side effects on read (live hardware registers) - refusing a bulk scan there. Try WRAM (or another plain-memory space) instead.";
            }

            // So "memfind VRAM 00 FF 11" and "memfind VRAM 00FF11" both work.
            int max = 20;
            var patternWords = new List<string>(parts.Skip(2));
            if (patternWords.Count >= 2 && int.TryParse(patternWords[^1], out int parsedMax) && parsedMax > 0)
            {
                max = parsedMax;
                patternWords.RemoveAt(patternWords.Count - 1);
            }

            int[]? pattern = ParsePattern(string.Concat(patternWords));
            if (pattern == null) return "Pattern must be whole hex byte pairs, using '??' for a wildcard - e.g. 00FF11 or '00 ?? 11'.";
            if (pattern.Length == 0) return "Pattern is empty.";
            if (pattern.Length > space.Size) return $"Pattern is {pattern.Length} byte(s), longer than {space.Name} ({space.Size} bytes).";

            var hits = new List<int>();
            int last = space.Size - pattern.Length;
            for (int addr = 0; addr <= last; addr++)
            {
                bool match = true;
                for (int i = 0; i < pattern.Length; i++)
                {
                    if (pattern[i] < 0) continue;
                    if (space.Read(addr + i) != pattern[i]) { match = false; break; }
                }
                if (match)
                {
                    hits.Add(addr);
                    if (hits.Count > max) break;
                }
            }

            if (hits.Count == 0) return $"No match for {pattern.Length}-byte pattern in {space.Name}.";

            bool truncated = hits.Count > max;
            var shown = hits.Take(max).Select(a => $"  0x{a:X}");
            string header = truncated
                ? $"More than {max} match(es) in {space.Name} - showing the first {max}:"
                : $"{hits.Count} match(es) for the {pattern.Length}-byte pattern in {space.Name}:";
            return header + "\n" + string.Join('\n', shown);
        }

        // Returns one entry per byte: 0-255 for a literal, -1 for '??'.
        private static int[]? ParsePattern(string spec)
        {
            var cleaned = new System.Text.StringBuilder();
            foreach (char c in spec)
            {
                if (c == ',' || c == ':' || c == '-' || c == '_') continue;
                cleaned.Append(c);
            }
            string text = cleaned.ToString();
            if (text.Length % 2 != 0) return null;

            var result = new int[text.Length / 2];
            for (int i = 0; i < result.Length; i++)
            {
                string pair = text.Substring(i * 2, 2);
                if (pair == "??") { result[i] = -1; continue; }
                if (!byte.TryParse(pair, System.Globalization.NumberStyles.HexNumber, null, out byte value)) return null;
                result[i] = value;
            }
            return result;
        }
    }
}
