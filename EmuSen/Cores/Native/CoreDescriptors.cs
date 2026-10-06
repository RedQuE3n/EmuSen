using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Native
{
    // A pad bit of a v1 core's controller and the canonical button it stands for, null for an extra - see EmuSen_CoreAPI.md §6.3.
    public sealed record CoreButton(uint Bit, PadButton? Control, string Label);

    // An axis as set_axis receives it; Trigger runs 0 to 1, a stick -1 to 1 - see EmuSen_CoreAPI.md §6.3.
    public sealed record CoreAxis(uint Axis, PadAxis? Control, bool Trigger, string Label);

    public sealed record CoreController(string Id, string Label, IReadOnlyList<uint> Ports, IReadOnlyList<CoreButton> Buttons, IReadOnlyList<CoreAxis> Axes);

    // A firmware file, passed as file Which; Parts are its split forms, Replacement what runs without it - see EmuSen_CoreAPI.md §6.2.
    public sealed record CoreFirmware(uint Which, string Name, string Label, long Size, bool Required, IReadOnlyList<IReadOnlyList<string>> Parts)
    {
        public CoreReplacement? Replacement { get; init; }
    }

    // An open replacement's effect in the settings schema's words, exact, accuracy or none, and its cost in plain words - see EmuSen_CoreAPI.md §6.2.
    public sealed record CoreReplacement(string Effect, string? Cost);

    // Which path a machine runs for firmware file Which: file, replacement or absent - see EmuSen_CoreAPI.md §6.4.
    public sealed record CoreFirmwareSource(uint Which, string Source);

    public sealed record CoreSystem(string Id, string Name, IReadOnlyList<string> Extensions, IReadOnlyList<string> Regions, IReadOnlyList<CoreController> Controllers, IReadOnlyList<CoreFirmware> Firmware);

    // A v1 core's info document, with Text the JSON it was read from - see EmuSen_CoreAPI.md §6.3.
    public sealed record CoreInfo(string Abi, string Id, string Name, string DisplayName, string Version, string License, IReadOnlyList<string> Authors,
        string? Description, IReadOnlyList<CoreSystem> Systems, IReadOnlyList<string> Capabilities, IReadOnlyList<string> HostRequires, bool Deterministic, string Text)
    {
        public bool Claims(string extension) => Systems.Any(s => s.Extensions.Contains(extension, StringComparer.OrdinalIgnoreCase));

        public CoreSystem? SystemFor(string extension) => Systems.FirstOrDefault(s => s.Extensions.Contains(extension, StringComparer.OrdinalIgnoreCase));
    }

    public sealed record CoreSpace(uint Id, string Name, long Size, IReadOnlyList<string> Flags)
    {
        public bool ReadOnly => Flags.Contains("read_only");
        public bool SideEffects => Flags.Contains("side_effects");
        public bool ReportsStores => Flags.Contains("reports_stores");
    }

    public sealed record CoreProcessor(uint Id, string Name, int PcBits, IReadOnlyList<(string Name, int Bits)> Registers, uint? CodeSpace = null);

    public sealed record CoreBatteryFile(uint Which, string Suffix, long Length);

    public sealed record CorePort(uint Port, string? Controller);

    // A machine's descriptor for its game; every field has the default EmuSen_CoreAPI.md §6.4 states for an older core that omits it.
    public sealed record CoreMachineInfo(string System, string? Region, ulong FrameRateNum, ulong FrameRateDen, int BaseWidth, int BaseHeight, int MaxWidth, int MaxHeight,
        uint AspectNum, uint AspectDen, int AudioRate, IReadOnlyList<string> Channels, IReadOnlyList<CorePort> Ports, IReadOnlyList<CoreSpace> Spaces,
        IReadOnlyList<CoreProcessor> Processors, IReadOnlyList<CoreBatteryFile> Battery, string StateFormat, long StateVersion, IReadOnlyList<uint> StateKinds,
        IReadOnlyList<string> Phases, long? PatchLow, long? PatchHigh, bool SkipRenderingStateNeutral)
    {
        public double FrameRateHz => FrameRateDen == 0 ? 60.0 : (double)FrameRateNum / FrameRateDen;

        public IReadOnlyList<CoreFirmwareSource> Firmware { get; init; } = Array.Empty<CoreFirmwareSource>();
    }

    // One setting of a v1 core's schema, its words and trade-off kept for presentation - see EmuSen_CoreAPI.md §6.13.
    public sealed record CoreSettingDescriptor(string Key, string Label, string Help, string Kind, string Default, int Min, int Max, IReadOnlyList<string> Choices,
        bool CreateScope, string? Category, string Effect, string? Cost, bool Advanced, bool Hidden, bool Restart)
    {
        // The words each choice is shown by, in the order of Choices; a value is its own label where the schema gives none.
        public IReadOnlyList<string> ChoiceLabels { get; init; } = Array.Empty<string>();

        // As the settings window lists it; a text setting has no row there yet, so null.
        public CoreSetting? AsCoreSetting() => Kind switch
        {
            "switch" => new CoreSetting(Key, Label, Hint, CoreSettingKind.Switch, Default),
            "count" => new CoreSetting(Key, Label, Hint, CoreSettingKind.Count, Default, Min, Max),
            "choice" => new CoreSetting(Key, Label, Hint, CoreSettingKind.Choice, Default, Choices: Choices, ChoiceLabels: ChoiceLabels.Count == Choices.Count ? ChoiceLabels : null),
            _ => null,
        };

        private string Hint => Cost is null ? Help : $"{Help} {Cost}";
    }

    // The JSON of the v1 descriptors read as records; a field this host does not know is ignored, one it needs and is absent takes its default.
    public static class CoreDescriptorReader
    {
        private static string Str(JsonElement e, string name, string fallback = "") =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.String ? v.GetString() ?? fallback : fallback;

        private static string? OptStr(JsonElement e, string name) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;

        private static long Long(JsonElement e, string name, long fallback = 0) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.Number && v.TryGetInt64(out long n) ? n : fallback;

        private static bool Bool(JsonElement e, string name, bool fallback = false) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind is JsonValueKind.True or JsonValueKind.False ? v.GetBoolean() : fallback;

        private static IEnumerable<JsonElement> Arr(JsonElement e, string name) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.Array ? v.EnumerateArray() : Enumerable.Empty<JsonElement>();

        private static IReadOnlyList<string> Strs(JsonElement e, string name) =>
            Arr(e, name).Where(x => x.ValueKind == JsonValueKind.String).Select(x => x.GetString()!).ToArray();

        private static JsonElement Obj(JsonElement e, string name) =>
            e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.Object ? v : default;

        private static T? EnumNamed<T>(string? name) where T : struct, Enum => name is not null && Enum.TryParse(name, false, out T v) && Enum.IsDefined(v) ? v : null;

        private static CoreFirmware Firmware(JsonElement f) => new((uint)Long(f, "which"), Str(f, "name"), Str(f, "label"), Long(f, "size"), Bool(f, "required"),
            Arr(f, "parts").Select(p => (IReadOnlyList<string>)p.EnumerateArray().Where(x => x.ValueKind == JsonValueKind.String).Select(x => x.GetString()!).ToArray()).ToArray())
        {
            Replacement = Obj(f, "replacement") is { ValueKind: JsonValueKind.Object } r ? new CoreReplacement(Str(r, "effect", "none"), OptStr(r, "cost")) : null,
        };

        public static CoreInfo Info(string json)
        {
            using var doc = JsonDocument.Parse(json);
            var r = doc.RootElement;
            if (r.ValueKind != JsonValueKind.Object) throw new JsonException("core info is not an object");
            var systems = Arr(r, "systems").Select(s => new CoreSystem(Str(s, "id"), Str(s, "name"), Strs(s, "extensions").Select(x => x.ToLowerInvariant()).ToArray(), Strs(s, "regions"),
                Arr(s, "controllers").Select(c => new CoreController(Str(c, "id"), Str(c, "label"), Arr(c, "ports").Select(p => (uint)p.GetInt64()).ToArray(),
                    Arr(c, "buttons").Select(b => new CoreButton((uint)Long(b, "bit"), EnumNamed<PadButton>(OptStr(b, "control")), Str(b, "label"))).ToArray(),
                    Arr(c, "axes").Select(a => new CoreAxis((uint)Long(a, "axis"), EnumNamed<PadAxis>(OptStr(a, "control")), Str(a, "kind") == "trigger", Str(a, "label"))).ToArray())).ToArray(),
                Arr(s, "firmware").Select(Firmware).ToArray())).ToArray();
            string name = Str(r, "name");
            return new CoreInfo(Str(r, "abi"), Str(r, "id"), name, OptStr(r, "display_name") ?? name, Str(r, "version"), Str(r, "license"), Strs(r, "authors"),
                OptStr(r, "description"), systems, Strs(r, "capabilities"), Strs(r, "host_requires"), Bool(r, "deterministic", true), json);
        }

        public static IReadOnlyList<CoreFirmware> FirmwareList(string json)
        {
            using var doc = JsonDocument.Parse(json);
            return doc.RootElement.ValueKind == JsonValueKind.Array ? doc.RootElement.EnumerateArray().Select(Firmware).ToArray() : Array.Empty<CoreFirmware>();
        }

        // audioRate stands in for an absent audio.rate: the export's answer at the time of reading.
        public static CoreMachineInfo MachineInfo(string json, int audioRate)
        {
            using var doc = JsonDocument.Parse(json);
            var r = doc.RootElement;
            var rate = Obj(r, "frame_rate");
            var video = Obj(r, "video");
            var aspect = Obj(video, "aspect");
            var audio = Obj(r, "audio");
            var state = Obj(r, "state");
            var patches = Obj(r, "patches");
            var kinds = Arr(state, "kinds").Select(k => (uint)k.GetInt64()).ToArray();
            return new CoreMachineInfo(Str(r, "system"), OptStr(r, "region"),
                (ulong)Long(rate, "num", 60), (ulong)Long(rate, "den", 1),
                (int)Long(video, "base_width"), (int)Long(video, "base_height"), (int)Long(video, "max_width"), (int)Long(video, "max_height"),
                (uint)Long(aspect, "num"), (uint)Long(aspect, "den"),
                (int)Long(audio, "rate", audioRate), Strs(audio, "channels"),
                Arr(r, "ports").Select(p => new CorePort((uint)Long(p, "port"), OptStr(p, "controller"))).ToArray(),
                Arr(r, "spaces").Select(s => new CoreSpace((uint)Long(s, "id"), Str(s, "name"), Long(s, "size"), Strs(s, "flags"))).ToArray(),
                Arr(r, "processors").Select(p => new CoreProcessor((uint)Long(p, "id"), Str(p, "name"), (int)Long(p, "pc_bits"),
                    Arr(p, "registers").Select(g => (Str(g, "name"), (int)Long(g, "bits"))).ToArray(),
                    p.TryGetProperty("code_space", out var code) && code.ValueKind == JsonValueKind.Number ? (uint)code.GetInt64() : null)).ToArray(),
                Arr(r, "battery").Select(b => new CoreBatteryFile((uint)Long(b, "which"), Str(b, "suffix", ".srm"), Long(b, "length"))).ToArray(),
                Str(state, "format"), Long(state, "version"), kinds.Length == 0 ? new uint[] { 0 } : kinds,
                Strs(r, "phases"),
                patches.ValueKind == JsonValueKind.Object ? Long(patches, "low") : null, patches.ValueKind == JsonValueKind.Object ? Long(patches, "high") : null,
                Bool(r, "skip_rendering_state_neutral"))
            {
                Firmware = Arr(r, "firmware").Select(f => new CoreFirmwareSource((uint)Long(f, "which"), Str(f, "source", "absent"))).ToArray(),
            };
        }

        public static IReadOnlyList<CoreSettingDescriptor> Settings(string json)
        {
            using var doc = JsonDocument.Parse(json);
            if (doc.RootElement.ValueKind != JsonValueKind.Array) return Array.Empty<CoreSettingDescriptor>();
            return doc.RootElement.EnumerateArray().Select(s => new CoreSettingDescriptor(Str(s, "key"), Str(s, "label"), Str(s, "help"), Str(s, "kind"), Str(s, "default"),
                (int)Long(s, "min"), (int)Long(s, "max"), Arr(s, "choices").Select(c => Str(c, "value")).ToArray(), Str(s, "scope") == "create", OptStr(s, "category"),
                Str(s, "effect", "none"), OptStr(s, "cost"), Bool(s, "advanced"), Bool(s, "hidden"), Bool(s, "restart"))
            { ChoiceLabels = Arr(s, "choices").Select(c => Str(c, "label", Str(c, "value"))).ToArray() }).ToArray();
        }

        // key -> sentence, from setting_notes.
        public static IReadOnlyDictionary<string, string> Notes(string json)
        {
            using var doc = JsonDocument.Parse(json);
            var notes = new Dictionary<string, string>();
            if (doc.RootElement.ValueKind == JsonValueKind.Object)
                foreach (var p in doc.RootElement.EnumerateObject())
                    if (p.Value.ValueKind == JsonValueKind.String) notes[p.Name] = p.Value.GetString()!;
            return notes;
        }
    }
}
