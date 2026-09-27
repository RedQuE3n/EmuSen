using System;
using System.IO;
using System.Text.Json;

namespace EmuSen.Galaxia.Library
{
    // Who wrote a save state, so a frontend can refuse or warn before a core reads it; Mistress keeps these in records.db - see EmuSen_Galaxia.md §5.3.
    public sealed record StateRecord
    {
        public required string Console { get; init; }
        public required string Core { get; init; }
        public required int StateVersion { get; init; }
        public required string Build { get; init; }
        public required DateTime SavedAt { get; init; }
        public required string RomFile { get; init; }
        public string? RomMd5 { get; init; }
        public long RomBytes { get; init; }

        // The JSON file builds before 2026-09-26 wrote beside each state; read for migration, never written - see EmuSen_Galaxia.md §5.3a.
        public static string SidecarPathFor(string statePath) => Path.ChangeExtension(statePath, ".json");

        // Null for a state with no sidecar, or one that will not parse; either way the state itself is untouched.
        public static StateRecord? ReadSidecar(string statePath)
        {
            try
            {
                string path = SidecarPathFor(statePath);
                return File.Exists(path) ? JsonSerializer.Deserialize<StateRecord>(File.ReadAllText(path), ConfigJson.Options) : null;
            }
            catch (Exception ex) when (ex is IOException or JsonException or UnauthorizedAccessException)
            {
                return null;
            }
        }
    }
}
