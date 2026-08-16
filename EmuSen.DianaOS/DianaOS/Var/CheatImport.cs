using System;
using EmuSen.DianaOS.DianaOS.Lib;

namespace EmuSen.DianaOS.DianaOS.Var
{
    public readonly struct CheatImportResult
    {
        public int Loaded { get; init; }

        // Entries that parsed but could not become writes - see `man cheat`.
        public int Skipped { get; init; }
    }

    // The one .cht-into-a-registry path, shared by both commands and Mistress - see `man cheat`.
    public static class CheatImport
    {
        // Where a decoded RAM poke lands when a codec names no space of its own.
        public const string DefaultSpaceName = "CpuBus";

        public static CheatImportResult FromChtText(CheatRegistry registry, string text, ICheatCodeCodec? codec, bool replace = false)
        {
            ChtParseResult parsed = ChtFile.Parse(text, codec, codec?.SpaceName ?? DefaultSpaceName);

            if (replace) registry.Clear();

            int loaded = 0;
            foreach (ChtCheat cheat in parsed.Cheats)
            {
                // Always disabled, however the file flagged it - see `man cheat`.
                try { registry.AddCheat(CheatKind.RamPoke, cheat.Writes, null, cheat.Description, enabled: false); loaded++; }
                catch (ArgumentException) { }
            }

            return new CheatImportResult { Loaded = loaded, Skipped = parsed.Skipped };
        }

        public static CheatImportResult FromChtFile(CheatRegistry registry, string path, ICheatCodeCodec? codec, bool replace = false) =>
            FromChtText(registry, System.IO.File.ReadAllText(path), codec, replace);
    }
}
