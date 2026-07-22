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
                    "tile" => CmdTile(parts),
                    "disasm" => CmdDisasm(parts),
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
                "  tile <space> <addr> <bpp>     ASCII-decode one 8x8 tile (bpp: 2, 4, or 8)",
                "  disasm <space> <addr> [<n>]   disassemble <n> instructions (default 10)",
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
    }
}
