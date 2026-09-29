using System;
using System.IO;
using EmuSen.Galaxia.Library;

namespace EmuSen.Cores
{
    // A cartridge's battery save: its one place in the Saves folder, the --nobattery latch, the atomic write and the flush period - see EmuSen_Settings_Reference.md §4.85.6.
    public sealed class BatterySave
    {
        // How often a core writes its save without being asked, about five seconds at sixty frames.
        public const int FlushEveryNFrames = 300;

        public static readonly BatterySave None = new(null);

        // Null when nothing is read or written: --nobattery, no battery-backed RAM, or no ROM path.
        public string? Path { get; }

        private BatterySave(string? path) => Path = path;

        public static bool IsFlushFrame(long frame) => frame % FlushEveryNFrames == 0;

        // disabled null defers to CoreOptions.BatteryRamDisabled, read here and latched; migrate copies a save a build before 2026-09-28 wrote beside the ROM.
        public static BatterySave Open(string? romPath, bool hasRam = true, bool? disabled = null, string extension = SaveLibrary.SramExtension, bool migrate = false)
        {
            if ((disabled ?? CoreOptions.BatteryRamDisabled) || !hasRam || string.IsNullOrEmpty(romPath)) return None;
            string path = System.IO.Path.ChangeExtension(SaveLibrary.SramPathFor(romPath), extension);
            if (migrate) CopyFromBesideRom(romPath, path, extension);
            return new BatterySave(path);
        }

        // Copied, never moved: the file beside the ROM is left exactly as it was, and a save already in Saves wins.
        private static void CopyFromBesideRom(string romPath, string path, string extension)
        {
            string beside = System.IO.Path.GetFullPath(System.IO.Path.ChangeExtension(romPath, extension));
            if (File.Exists(path) || string.Equals(beside, System.IO.Path.GetFullPath(path), StringComparison.Ordinal)) return;
            if (AtomicFile.TryRead(beside) is not { } saved) return;
            if (AtomicFile.Write(path, saved)) Console.WriteLine($"[BatterySave] Copied {beside} to {path}, where this game's battery save now lives; the original is untouched.");
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
