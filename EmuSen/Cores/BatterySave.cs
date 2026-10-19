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

        // Where the save was kept before, read until Settle copies it in; null when it already lives at Path.
        private readonly string? _previous;

        private BatterySave(string? path, string? previous = null) => (Path, _previous) = (path, previous);

        public static bool IsFlushFrame(long frame) => frame % FlushEveryNFrames == 0;

        // The save folders, one per console as the ROM library names them.
        public const string Nes = "NES", Snes = "SNES", N64 = "N64", GameBoy = "GB", GameBoyColor = "GBC", Genesis = "Genesis", SegaCd = "Sega CD", S32x = "32X";

        // disabled is the core's CoreOptions.BatteryRamDisabled, latched here; a save made before 2026-09-28 is read in place and copied in by Settle, for a file the machine keeps - see EmuSen_Backend.md §17.10.
        public static BatterySave Open(string? romPath, string console, bool hasRam = true, bool disabled = false, string extension = SaveLibrary.SramExtension)
        {
            if (disabled || !hasRam || string.IsNullOrEmpty(romPath)) return None;
            string path = System.IO.Path.ChangeExtension(SaveLibrary.SramPathFor(romPath, console), extension);
            return new BatterySave(path, File.Exists(path) ? null : PreviousHome(romPath, console, extension));
        }

        // The save kept before copied into its own place, once a load knows the machine keeps the file; the original untouched.
        public void Settle(Action<string>? say = null)
        {
            if (Path is not null && _previous is not null && !File.Exists(Path)) CopyIn(_previous, Path, say);
        }

        // A card shared between games, by its own path rather than a game's; --nobattery holds for it as for any save - see Diamond_Cards.md §9.2.
        public static BatterySave Card(string console, string name, string extension, bool disabled = false) => Medium(console, false, name, extension, disabled);

        // A shared or named medium's file of either kind - see EmuSen_SaveMedia.md §3.3.
        public static BatterySave Medium(string console, bool consoleKind, string name, string extension, bool disabled = false) =>
            disabled ? None : new BatterySave(SaveLibrary.MediumPathFor(console, consoleKind, name, extension));

        // The one place each console's saves were kept before: the flat Saves folder for the SNES and N64, beside the ROM for the NES and Game Boy.
        public static string? PreviousHome(string romPath, string console, string extension = SaveLibrary.SramExtension) => console switch
        {
            Snes or N64 => System.IO.Path.ChangeExtension(SaveLibrary.FlatSramPathFor(romPath), extension),
            Nes or GameBoy or GameBoyColor => System.IO.Path.ChangeExtension(romPath, extension),
            // Before 2026-10-06 a development build filed them under the system's id - see EmuSen_Settings_Reference.md §4.91.
            Genesis => System.IO.Path.ChangeExtension(SaveLibrary.SramPathFor(romPath, "MD"), extension),
            SegaCd => System.IO.Path.ChangeExtension(SaveLibrary.SramPathFor(romPath, "MCD"), extension),
            _ => null,
        };

        // Copied, never moved: the old file is left exactly as it was.
        private static void CopyIn(string previous, string path, Action<string>? say)
        {
            if (string.Equals(System.IO.Path.GetFullPath(previous), System.IO.Path.GetFullPath(path), StringComparison.Ordinal)) return;
            if (AtomicFile.TryRead(previous) is not { } saved) return;
            if (AtomicFile.Write(path, saved)) say?.Invoke($"[BatterySave] Copied {previous} to {path}, where this game's battery save now lives; the original is untouched.");
        }

        public byte[]? Read() => Path is null ? null : AtomicFile.TryRead(Path) ?? (_previous is null || File.Exists(Path) ? null : AtomicFile.TryRead(_previous));

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
