using System;
using System.IO;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores
{
    // A cartridge's battery save: its console's folder in Saves, the --nobattery latch, the atomic write and the flush period - see EmuSen_Settings_Reference.md §4.85.6 and §4.85.11.
    public sealed class BatterySave
    {
        // How often a core writes its save without being asked, about five seconds at sixty frames.
        public const int FlushEveryNFrames = 300;

        public static readonly BatterySave None = new(null);

        // Null when nothing is read or written: --nobattery, no battery-backed RAM, or no ROM path.
        public string? Path { get; }

        private BatterySave(string? path) => Path = path;

        public static bool IsFlushFrame(long frame) => frame % FlushEveryNFrames == 0;

        // The save folders, one per console as the ROM library names them.
        public const string Nes = "NES", Snes = "SNES", N64 = "N64", GameBoy = "GB", GameBoyColor = "GBC";

        // By the file's extension alone, as the library files them: a setting or a header never moves a Game Boy save.
        public static string GameBoyFolder(string romPath) =>
            string.Equals(System.IO.Path.GetExtension(romPath), ".gbc", StringComparison.OrdinalIgnoreCase) ? GameBoyColor : GameBoy;

        // disabled null defers to CoreOptions.BatteryRamDisabled, read here and latched; a save made before 2026-09-28 is copied in at the first load.
        public static BatterySave Open(string? romPath, string console, bool hasRam = true, bool? disabled = null, string extension = SaveLibrary.SramExtension)
        {
            if ((disabled ?? CoreOptions.BatteryRamDisabled) || !hasRam || string.IsNullOrEmpty(romPath)) return None;
            string path = System.IO.Path.ChangeExtension(SaveLibrary.SramPathFor(romPath, console), extension);
            if (!File.Exists(path) && PreviousHome(romPath, console, extension) is { } previous) CopyIn(previous, path);
            return new BatterySave(path);
        }

        // The one place each console's saves were kept before: the flat Saves folder for the SNES and N64, beside the ROM for the NES and Game Boy.
        public static string? PreviousHome(string romPath, string console, string extension = SaveLibrary.SramExtension) => console switch
        {
            Snes or N64 => System.IO.Path.ChangeExtension(SaveLibrary.FlatSramPathFor(romPath), extension),
            Nes or GameBoy or GameBoyColor => System.IO.Path.ChangeExtension(romPath, extension),
            _ => null,
        };

        // Copied, never moved: the old file is left exactly as it was.
        private static void CopyIn(string previous, string path)
        {
            if (string.Equals(System.IO.Path.GetFullPath(previous), System.IO.Path.GetFullPath(path), StringComparison.Ordinal)) return;
            if (AtomicFile.TryRead(previous) is not { } saved) return;
            if (AtomicFile.Write(path, saved)) Console.WriteLine($"[BatterySave] Copied {previous} to {path}, where this game's battery save now lives; the original is untouched.");
        }

        public byte[]? Read() => Path is null ? null : AtomicFile.TryRead(Path);

        // The save's bytes into ram, as many as both hold; false when there was nothing to read.
        public bool ReadInto(Span<byte> ram)
        {
            if (Read() is not { } saved) return false;
            saved.AsSpan(0, Math.Min(saved.Length, ram.Length)).CopyTo(ram);
            return true;
        }

        public bool Write(byte[] contents) => Path is not null && AtomicFile.Write(Path, contents);
    }
}
