using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // Decodes Pro Action Replay / Game Wizard style RAM-poke codes - both
    // Datel devices, and (per published documentation of the format) using
    // the same plain 8-hex-digit wire format: a 24-bit CPU-bus address
    // (bank + 16-bit offset) followed by the one byte to hold there every
    // frame - e.g. "7E01F663" means "poke CPU address $7E:01F6 to 0x63".
    // No cipher or bit-scrambling involved, unlike Game Genie - see that
    // codec once it exists for the very different mechanism it decodes.
    //
    // Targets the "CpuBus" memory space by convention (see CheatCommand),
    // not a raw WRAM array offset - a real Action Replay/Game Wizard pokes
    // the CPU bus address the game itself would use, so mirroring (WRAM's
    // $7E/$7F pair, the $00-$3F low-page mirror, etc.) resolves the same
    // way it would on real hardware.
    public static class ActionReplayCodec
    {
        // True if <code> looks like a Pro Action Replay/Game Wizard code
        // at all - exactly 8 hex digits once separators are stripped.
        // Meant for a future auto-detecting `cheat add` that tries each
        // known codec in turn (see the phased cheat-engine plan).
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        // Throws FormatException with a readable message on anything that
        // isn't exactly 8 hex digits once non-hex separators (spaces,
        // dashes, colons - some published code lists format them
        // "7E01F6:63" or "7E01F6-63") are stripped. Callers (CheatCommand)
        // let this propagate up to ShellInterpreter's existing
        // catch-and-stringify handling, same as every other command's
        // parse errors.
        public static (int Address, byte Value) Decode(string code)
        {
            if (!TryNormalize(code, out string hex))
            {
                throw new FormatException($"'{code}' isn't a valid Pro Action Replay/Game Wizard code - expected 8 hex digits (6-digit address + 2-digit value).");
            }

            int address = Convert.ToInt32(hex.Substring(0, 6), 16);
            byte value = Convert.ToByte(hex.Substring(6, 2), 16);
            return (address, value);
        }

        private static bool TryNormalize(string code, out string hex)
        {
            hex = new string((code ?? "").Where(Uri.IsHexDigit).ToArray());
            return hex.Length == 8;
        }
    }
}
