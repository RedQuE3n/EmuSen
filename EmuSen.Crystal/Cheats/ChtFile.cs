using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Crystal.Cheats
{
    // Two mechanisms era devices used: a write to RAM each frame, or a byte of the cartridge answered otherwise - see EmuSen_Cheats.md.
    public enum CheatKind
    {
        RamPoke,
        RomPatch,
    }

    // One cheat read out of a .cht file.
    public readonly struct ChtCheat
    {
        public string Description { get; init; }
        public bool Enabled { get; init; }
        public IReadOnlyList<CheatWrite> Writes { get; init; }

        // A code the patch codec claims is a ROM patch; everything else a RAM poke.
        public CheatKind Kind { get; init; }
    }

    public readonly struct ChtParseResult
    {
        public IReadOnlyList<ChtCheat> Cheats { get; init; }

        // Entries present but unusable - an undecodable code, type 0, a malformed number.
        public int Skipped { get; init; }
    }

    // RetroArch's format, emusen-cheats' cht module; only the code string is core-specific, decoded by the pack's codecs - see `man cheat`, EmuSen_Backend.md §15.3.
    public static class ChtFile
    {
        // A codec must be a pack's, which the library names; null reads no code.
        public static ChtParseResult Parse(string text, ICheatCodeCodec? codeDecoder, string spaceName, ICheatCodeCodec? patchCodec = null)
        {
            ArgumentNullException.ThrowIfNull(text);
            ArgumentNullException.ThrowIfNull(spaceName);
            return CheatsNative.Parse(text, codeDecoder, spaceName, patchCodec);
        }

        // RomPatch has no equivalent in RetroArch's model; filtering is the caller's - see `man cheat`.
        public static string Write(IReadOnlyList<ChtCheat> cheats)
        {
            if (cheats.Any(c => c.Description is null || c.Writes is null)) throw new ArgumentException("a cheat to write needs its description and its writes", nameof(cheats));
            return CheatsNative.Write(cheats);
        }
    }
}
