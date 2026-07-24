using System.Linq;
using EmuSen.Debug.Cheats;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    // `cheat` sub-commands - RAM-poke cheats (Pro Action Replay/Game
    // Wizard style, see ActionReplayCodec) applied every frame via
    // CheatRegistry. Game Genie's ROM-intercept codes are a different
    // mechanism entirely and aren't handled here yet - see the phased
    // cheat-engine plan for that separate addition.
    public class CheatCommand : IDebugCommand
    {
        public string Name => "cheat";
        public string Usage => string.Join('\n', new[]
        {
            "  cheat add <code> [description]    decode a Pro Action Replay/Game Wizard",
            "                                     code (8 hex digits) and add it, enabled",
            "  cheat poke <space> <addr> <value> [description]",
            "                                     add a raw poke cheat directly, bypassing",
            "                                     code decoding (e.g. an address you already",
            "                                     found with `search`)",
            "  cheat list                         list every cheat with its ID and state",
            "  cheat enable <id>                  turn a cheat back on",
            "  cheat disable <id>                 turn a cheat off without removing it",
            "  cheat remove <id>                  remove a cheat entirely",
            "  cheat clear                        remove every cheat",
        });

        // Decoded codes always target this space, not a raw WRAM array
        // offset - see ActionReplayCodec's own comment on why a CPU-bus
        // address is the correct target (mirroring resolves the same way
        // real hardware's does).
        private const string DecodedCodeSpace = "CpuBus";

        public string Execute(IDebugTarget target, string[] parts)
        {
            if (parts.Length < 2) return "Usage: cheat add|poke|list|enable|disable|remove|clear ...";
            string sub = parts[1].ToLowerInvariant();
            var cheats = target.Cheats;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 3) return "Usage: cheat add <code> [description]";
                    (int address, byte value) = ActionReplayCodec.Decode(parts[2]);
                    string description = parts.Length > 3 ? string.Join(' ', parts.Skip(3)) : parts[2];
                    int id = cheats.AddCheat(DecodedCodeSpace, address, value, description);
                    return $"Cheat #{id} added: {DecodedCodeSpace} 0x{address:X6} = 0x{value:X2} ({description})";
                }
                case "poke":
                {
                    if (parts.Length < 5) return "Usage: cheat poke <space> <addr> <value> [description]";
                    FindSpace(target, parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    byte value = (byte)ParseHex(parts[4]);
                    string description = parts.Length > 5 ? string.Join(' ', parts.Skip(5)) : $"{parts[2]} 0x{addr:X}";
                    int id = cheats.AddCheat(parts[2], addr, value, description);
                    return $"Cheat #{id} added: {parts[2]} 0x{addr:X} = 0x{value:X2} ({description})";
                }
                case "list":
                {
                    var list = cheats.GetCheats();
                    if (list.Count == 0) return "No cheats added.";
                    return string.Join('\n', list.Select(c => $"  #{c.Id}: [{(c.Enabled ? "on " : "off")}] {c.SpaceName} 0x{c.Address:X} = 0x{c.Value:X2}  {c.Description}"));
                }
                case "enable":
                {
                    if (parts.Length < 3) return "Usage: cheat enable <id>";
                    bool ok = cheats.SetEnabled(ParseHex(parts[2]), true);
                    return ok ? $"Cheat #{parts[2]} enabled." : $"No cheat #{parts[2]} found.";
                }
                case "disable":
                {
                    if (parts.Length < 3) return "Usage: cheat disable <id>";
                    bool ok = cheats.SetEnabled(ParseHex(parts[2]), false);
                    return ok ? $"Cheat #{parts[2]} disabled." : $"No cheat #{parts[2]} found.";
                }
                case "remove":
                {
                    if (parts.Length < 3) return "Usage: cheat remove <id>";
                    bool removed = cheats.RemoveCheat(ParseHex(parts[2]));
                    return removed ? $"Cheat #{parts[2]} removed." : $"No cheat #{parts[2]} found.";
                }
                case "clear":
                {
                    cheats.Clear();
                    return "All cheats removed.";
                }
                default:
                    return $"Unknown 'cheat' subcommand '{sub}'. Try add/poke/list/enable/disable/remove/clear.";
            }
        }
    }
}
