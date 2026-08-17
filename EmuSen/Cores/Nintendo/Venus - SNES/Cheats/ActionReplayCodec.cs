using System;
using System.Linq;

namespace EmuSen.Cores.Nintendo.Venus.Cheats
{
    // Decodes Pro Action Replay / Game Wizard style RAM-poke codes - both Datel devices, and (per.
    public static class ActionReplayCodec
    {
        // True if <code> looks like a Pro Action Replay/Game Wizard code at all - exactly 8 hex digits once.
        public static bool CanDecode(string code) => TryNormalize(code, out _);

        // Throws FormatException with a readable message on anything that isn't exactly 8 hex digits once.
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
