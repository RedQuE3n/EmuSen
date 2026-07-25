using System.Linq;
using EmuSen.Debug.Cheats;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    // `cheat` sub-commands - every cheat mechanism this project supports,
    // unified under one command since a player thinks of both as just
    // "my cheats" (see CheatRegistry's own comment for the full
    // explanation of each):
    //   - RAM pokes (Pro Action Replay/Game Wizard style, decoded by
    //     ActionReplayCodec) - `add`/`poke`.
    //   - ROM patches (Game Genie style, decoded by GameGenieCodec) -
    //     `gg`/`rompatch`.
    public class CheatCommand : IDebugCommand
    {
        public string Name => "cheat";
        public string Usage => string.Join('\n', new[]
        {
            "  cheat add <code> [description]    decode a Pro Action Replay/Game Wizard",
            "                                     code (8 hex digits) and add it, enabled",
            "  cheat poke <space> <addr> <value> [description]",
            "                                     add a raw RAM-poke cheat directly, bypassing",
            "                                     code decoding (e.g. an address you already",
            "                                     found with `search`)",
            "  cheat gg <code> [description]     decode a real SNES Game Genie code (8",
            "                                     letters) and add it as a ROM patch, enabled",
            "  cheat rompatch <addr> <value> [<compare>|-] [description]",
            "                                     add a Game Genie-style ROM-read intercept",
            "                                     directly, bypassing code decoding: substitute",
            "                                     <value> for reads of cartridge address <addr>.",
            "                                     <compare> gates the patch to only apply while",
            "                                     the real byte there equals it (pass - to skip,",
            "                                     an unconditional patch - what every real SNES",
            "                                     Game Genie code decodes to; unlike NES/Genesis,",
            "                                     SNES Game Genie codes never carry a compare)",
            "  cheat list                         list every cheat with its ID, kind, and state",
            "  cheat enable <id>                  turn a cheat back on",
            "  cheat disable <id>                 turn a cheat off without removing it",
            "  cheat remove <id>                  remove a cheat entirely",
            "  cheat clear                        remove every cheat",
        });

        // Decoded Pro Action Replay/Game Wizard codes always target this
        // space, not a raw WRAM array offset - see ActionReplayCodec's own
        // comment on why a CPU-bus address is the correct target
        // (mirroring resolves the same way real hardware's does).
        private const string DecodedCodeSpace = "CpuBus";

        public string Execute(IDebugTarget target, string[] parts)
        {
            if (parts.Length < 2) return "Usage: cheat add|poke|gg|rompatch|list|enable|disable|remove|clear ...";
            string sub = parts[1].ToLowerInvariant();
            var cheats = target.Cheats;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 3) return "Usage: cheat add <code> [description]";
                    (int address, byte value) = ActionReplayCodec.Decode(parts[2]);
                    string description = parts.Length > 3 ? string.Join(' ', parts.Skip(3)) : parts[2];
                    int id = cheats.AddRamPoke(DecodedCodeSpace, address, value, description);
                    return $"Cheat #{id} added: {DecodedCodeSpace} 0x{address:X6} = 0x{value:X2} ({description})";
                }
                case "poke":
                {
                    if (parts.Length < 5) return "Usage: cheat poke <space> <addr> <value> [description]";
                    FindSpace(target, parts[2]); // validates the space name exists, or throws a helpful error
                    int addr = ParseHex(parts[3]);
                    byte value = (byte)ParseHex(parts[4]);
                    string description = parts.Length > 5 ? string.Join(' ', parts.Skip(5)) : $"{parts[2]} 0x{addr:X}";
                    int id = cheats.AddRamPoke(parts[2], addr, value, description);
                    return $"Cheat #{id} added: {parts[2]} 0x{addr:X} = 0x{value:X2} ({description})";
                }
                case "gg":
                {
                    if (parts.Length < 3) return "Usage: cheat gg <code> [description]";
                    (int ggAddress, byte ggValue) = GameGenieCodec.Decode(parts[2]);
                    string ggDescription = parts.Length > 3 ? string.Join(' ', parts.Skip(3)) : parts[2];
                    int ggId = cheats.AddRomPatch(ggAddress, ggValue, null, ggDescription);
                    return $"Cheat #{ggId} added: ROM 0x{ggAddress:X6} = 0x{ggValue:X2} ({ggDescription})";
                }
                case "rompatch":
                {
                    if (parts.Length < 4) return "Usage: cheat rompatch <addr> <value> [<compare>|-] [description]";
                    int addr = ParseHex(parts[2]);
                    byte value = (byte)ParseHex(parts[3]);
                    byte? compare = null;
                    int descStart = 4;
                    if (parts.Length > 4)
                    {
                        if (parts[4] != "-") compare = (byte)ParseHex(parts[4]);
                        descStart = 5;
                    }
                    string description = parts.Length > descStart ? string.Join(' ', parts.Skip(descStart)) : $"ROM 0x{addr:X6}";
                    int id = cheats.AddRomPatch(addr, value, compare, description);
                    string compareText = compare.HasValue ? $" if==0x{compare.Value:X2}" : "";
                    return $"Cheat #{id} added: ROM 0x{addr:X6} = 0x{value:X2}{compareText} ({description})";
                }
                case "list":
                {
                    var list = cheats.GetCheats();
                    if (list.Count == 0) return "No cheats added.";
                    return string.Join('\n', list.Select(c =>
                    {
                        string state = c.Enabled ? "on " : "off";
                        if (c.Kind == CheatKind.RamPoke)
                        {
                            return $"  #{c.Id}: [{state}] RAM  {c.SpaceName} 0x{c.Address:X} = 0x{c.Value:X2}  {c.Description}";
                        }
                        string compareText = c.Compare.HasValue ? $" if==0x{c.Compare.Value:X2}" : "";
                        return $"  #{c.Id}: [{state}] ROM  0x{c.Address:X6} = 0x{c.Value:X2}{compareText}  {c.Description}";
                    }));
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
                    return $"Unknown 'cheat' subcommand '{sub}'. Try add/poke/gg/rompatch/list/enable/disable/remove/clear.";
            }
        }
    }
}
