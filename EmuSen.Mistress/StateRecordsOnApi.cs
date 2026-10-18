using System;
using System.Globalization;
using System.Text.Json;
using EmuSen.Crystal.Client;
using EmuSen.Galaxia.Library;

namespace EmuSen.Mistress
{
    // A state's record as the runtime keeps it in states.db, read with no session - see EmuSen_Backend.md §14.12.
    public static class StateRecordsOnApi
    {
        // Null where the runtime holds none for the file as it is now.
        public static StateRecord? Read(CrystalRuntime runtime, string statePath)
        {
            string? json;
            try { json = runtime.StateRecord(statePath); }
            catch (CrystalException) { return null; }
            if (json is null) return null;
            using JsonDocument document = JsonDocument.Parse(json);
            JsonElement r = document.RootElement;
            return new StateRecord
            {
                Console = r.GetProperty("console").GetString() ?? "",
                Core = r.GetProperty("core").GetString() ?? "",
                StateVersion = r.GetProperty("state_version").GetInt32(),
                Build = r.GetProperty("build").GetString() ?? "",
                SavedAt = DateTime.Parse(r.GetProperty("saved_at").GetString() ?? "", CultureInfo.InvariantCulture, DateTimeStyles.RoundtripKind),
                RomFile = r.GetProperty("rom_file").GetString() ?? "",
                RomMd5 = r.GetProperty("rom_md5").ValueKind == JsonValueKind.String ? r.GetProperty("rom_md5").GetString() : null,
                RomBytes = r.GetProperty("rom_bytes").GetInt64(),
            };
        }
    }
}
