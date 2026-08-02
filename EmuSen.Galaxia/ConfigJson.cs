using System.Text.Json;
using System.Text.Json.Serialization;

namespace EmuSen.Galaxia
{
    // One set of options for every config file - see EmuSen_Config_Reference.md §2.1.
    public static class ConfigJson
    {
        public static readonly JsonSerializerOptions Options = new()
        {
            WriteIndented = true,
            // These files are meant to be opened and edited by hand from the
            // shell, so tolerate what a person actually types - see §2.1.
            ReadCommentHandling = JsonCommentHandling.Skip,
            AllowTrailingCommas = true,
            PropertyNameCaseInsensitive = true,
            // Enum values as names, not numbers: "DPadUp" over 11 - see §2.1.
            // Reads both, so files written before this still load. Not the
            // stock JsonStringEnumConverter: this one names the bad value and
            // suggests the nearest real one - see §6.2.
            Converters = { new SuggestingEnumConverterFactory() },
        };
    }
}
