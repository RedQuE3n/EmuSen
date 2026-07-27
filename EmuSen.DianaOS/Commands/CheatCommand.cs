using System;
using System.Linq;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // `cheat` sub-commands - every cheat mechanism this project supports,
    // unified under one command since a player thinks of both as just
    // "my cheats" (see CheatRegistry's own comment for the full
    // explanation of each):
    //   - RAM pokes (Pro Action Replay/Game Wizard style) - `poke`, or
    //     `add` when it guesses that's the format (see
    //     LooksLikeGameGenieFormat).
    //   - ROM patches (Game Genie style) - `rompatch`, or `add`/`gg`
    //     (see below).
    //
    // Genuinely core-agnostic now: `add`/`gg` decode through whichever
    // ICheatCodeCodec instances the host passes in (see
    // DianaOSInterpreter.CreateDefault's cheatAutoDetectCodec/
    // cheatExplicitCodec parameters), rather than calling a specific
    // core's code-format decoders directly - this is the "future
    // auto-detecting cheat add that tries each known codec in turn" the
    // codecs' own CanDecode methods were already written for. Without a
    // codec registered (a future core with no equivalent format, or a
    // standalone launch with no core at all), `add`/`gg` just report
    // there's nothing to decode with - `poke`/`rompatch`/`list`/`enable`/
    // `disable`/`remove`/`clear` keep working regardless, since they only
    // ever touch the generic CheatRegistry (IDebugTarget.Cheats), never a
    // codec.
    public class CheatCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        // `add`'s guessed format (LooksLikeGameGenieFormat == false) and
        // `poke`'s own raw path both land here; `_explicitCodec` backs
        // both `gg` and `add`'s other guess. Named by role, not by any
        // one core's format, since a future core supplies its own
        // instances for both roles.
        private readonly ICheatCodeCodec? _autoDetectCodec;
        private readonly ICheatCodeCodec? _explicitCodec;

        public CheatCommand(ICheatCodeCodec? autoDetectCodec = null, ICheatCodeCodec? explicitCodec = null)
        {
            _autoDetectCodec = autoDetectCodec;
            _explicitCodec = explicitCodec;
        }

        public string Name => "cheat";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  cheat add <code> [description]    decode a code, guessing whether it's Pro",
            "                                     Action Replay/Game Wizard or Game Genie from",
            "                                     its formatting (see `gg` if it guesses wrong -",
            "                                     both formats use the same 8 hex-digit",
            "                                     characters, so this is a best-effort guess,",
            "                                     not a guarantee)",
            "  cheat poke <space> <addr> <value> [description]",
            "                                     add a raw RAM-poke cheat directly, bypassing",
            "                                     code decoding (e.g. an address you already",
            "                                     found with `search`)",
            "  cheat gg <code> [description]     decode a real SNES Game Genie code (8",
            "                                     characters) and add it as a ROM patch, enabled",
            "                                     - use this instead of `add` when its guess is",
            "                                     wrong for a code formatted unconventionally",
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

        // `cheat add`'s format guess. Both codecs decode from the exact
        // same 8 hex-digit character set - SNES Game Genie's own alphabet
        // (see GameGenieCodec) is a scrambled ordering of 0-9A-F, not a
        // distinct letter set the way NES Game Genie's is - so there is no
        // way to tell the two formats apart from their characters alone.
        // Falls back to how each device's codes are conventionally
        // published instead: a separator right after the 4th character
        // ("XXXX-XXXX") matches Game Genie's own convention; anything
        // else (no separator at all, or one after the 6th character, e.g.
        // "AAAAAA-VV"/"AAAAAA:VV") is treated as Pro Action Replay/Game
        // Wizard's. Best-effort only, NOT a guarantee - `cheat gg`/`cheat
        // poke` are the reliable fallback for a code formatted
        // unconventionally (or copied without its original punctuation).
        private static bool LooksLikeGameGenieFormat(string code)
        {
            for (int i = 0; i < code.Length; i++)
            {
                if (!Uri.IsHexDigit(code[i])) return i == 4;
            }
            return false; // no separator at all - assume Pro Action Replay/Game Wizard
        }

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 2) return "Usage: cheat add|poke|gg|rompatch|list|enable|disable|remove|clear ...";
            string sub = parts[1].ToLowerInvariant();
            var cheats = target.Cheats;

            switch (sub)
            {
                case "add":
                {
                    if (parts.Length < 3) return "Usage: cheat add <code> [description]";
                    string code = parts[2];
                    string description = parts.Length > 3 ? string.Join(' ', parts.Skip(3)) : code;

                    if (LooksLikeGameGenieFormat(code))
                    {
                        if (_explicitCodec is null) return "No Game Genie-style cheat codec is registered for this target.";
                        (int ggAddress, byte ggValue) = _explicitCodec.Decode(code);
                        int ggId = cheats.AddRomPatch(ggAddress, ggValue, null, description);
                        return $"Cheat #{ggId} added (detected {_explicitCodec.Name} format): ROM 0x{ggAddress:X6} = 0x{ggValue:X2} ({description})";
                    }

                    if (_autoDetectCodec is null) return "No cheat codec is registered for this target.";
                    (int address, byte value) = _autoDetectCodec.Decode(code);
                    string space = _autoDetectCodec.SpaceName ?? "CpuBus";
                    int id = cheats.AddRamPoke(space, address, value, description);
                    return $"Cheat #{id} added (detected {_autoDetectCodec.Name} format): {space} 0x{address:X6} = 0x{value:X2} ({description})";
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
                    if (_explicitCodec is null) return "No Game Genie-style cheat codec is registered for this target.";
                    (int ggAddress, byte ggValue) = _explicitCodec.Decode(parts[2]);
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
