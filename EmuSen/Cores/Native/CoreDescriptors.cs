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

        // The MD5s of the images the core accepts whatever they are called; empty before 1.12 - see EmuSen_CoreAPI.md §6.32.
        public IReadOnlyList<string> Md5 { get; init; } = Array.Empty<string>();
    }

    // An open replacement's effect in the settings schema's words, exact, accuracy or none, and its cost in plain words - see EmuSen_CoreAPI.md §6.2.
    public sealed record CoreReplacement(string Effect, string? Cost);

    // Which path a machine runs for firmware file Which: file, replacement or absent - see EmuSen_CoreAPI.md §6.4.
    public sealed record CoreFirmwareSource(uint Which, string Source);

    public sealed record CoreSystem(string Id, string Name, IReadOnlyList<string> Extensions, IReadOnlyList<string> Regions, IReadOnlyList<CoreController> Controllers, IReadOnlyList<CoreFirmware> Firmware)
    {
        // The core builds the system but does not run its games yet; absent is false - see EmuSen_CoreAPI.md §27.4.
        public bool Development { get; init; }

        // A machine of the system can be made with no image, the console with nothing in it - see EmuSen_CoreAPI.md §6.31.
        public bool StartsEmpty { get; init; }

        // Every battery file a machine of the system can name, by number and suffix, so they are passed at the first create; null for a core that lists none - see EmuSen_CoreAPI.md §6.3.
        public IReadOnlyList<CoreBatteryFile>? Battery { get; init; }
    }

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
        // Its loads reach the read ranges of a core claiming DEBUG_READS - see EmuSen_CoreAPI.md §6.14.
        public bool ReportsLoads => Flags.Contains("reports_loads");
        // A window onto other spaces' memory, holding none of its own - see EmuSen_CoreAPI.md §29.
        public bool View => Flags.Contains("view");
    }

    public sealed record CoreProcessor(uint Id, string Name, int PcBits, IReadOnlyList<(string Name, int Bits)> Registers, uint? CodeSpace = null);

    // A part that runs no code and holds registers, read through debug_registers by its id - see EmuSen_CoreAPI.md §6.33.
    public sealed record CoreUnit(uint Id, string Name, string Kind, IReadOnlyList<(string Name, int Bits)> Registers);

    // One of a core's own counters, its value read by telemetry in machine info's order - see EmuSen_CoreAPI.md §6.34.
    public sealed record CoreCounter(string Name, string Label, string Unit, string Kind);

    // Whom a battery file belongs with: removable since 1.7, console since 1.8; a kind this host does not know reads as the game's - see EmuSen_CoreAPI.md §6.27, §6.28.
    public enum CoreBatteryKind { Game, Removable, Console }

    // Medium, Label and Port since 1.8, from core info only: the slot a frontend shows - see EmuSen_SaveMedia.md §3.2.
    public sealed record CoreBatteryFile(uint Which, string Suffix, long Length, CoreBatteryKind Kind = CoreBatteryKind.Game, string? Medium = null, string? Label = null, int? Port = null)
    {
        public bool Removable => Kind == CoreBatteryKind.Removable;

        public static CoreBatteryKind KindNamed(string? kind) => kind switch { "removable" => CoreBatteryKind.Removable, "console" => CoreBatteryKind.Console, _ => CoreBatteryKind.Game };
    }

    public sealed record CorePort(uint Port, string? Controller);

    // A drive of a machine claiming MEDIA: its number, words, kind, lid and whether it holds a disc - see EmuSen_CoreAPI.md §6.23.
    public sealed record CoreDrive(uint Drive, string Label, string Kind, bool LidOpen, bool Loaded);

    // One encoding of the picture before the core's palette, machine info's video.raw[n]; Palette is 0xRRGGBB by value, empty for none - see EmuSen_CoreAPI.md §6.30.
    public sealed record CoreRawEncoding(string Encoding, int Element, int MaxWidth, int MaxHeight, IReadOnlyList<int> Palette);

    // A machine's descriptor for its game; every field has the default EmuSen_CoreAPI.md §6.4 states for an older core that omits it.
    public sealed record CoreMachineInfo(string System, string? Region, ulong FrameRateNum, ulong FrameRateDen, int BaseWidth, int BaseHeight, int MaxWidth, int MaxHeight,
        uint AspectNum, uint AspectDen, int AudioRate, IReadOnlyList<string> Channels, IReadOnlyList<CorePort> Ports, IReadOnlyList<CoreSpace> Spaces,
        IReadOnlyList<CoreProcessor> Processors, IReadOnlyList<CoreBatteryFile> Battery, string StateFormat, long StateVersion, IReadOnlyList<uint> StateKinds,
        IReadOnlyList<string> Phases, long? PatchLow, long? PatchHigh, bool SkipRenderingStateNeutral)
    {
        public double FrameRateHz => FrameRateDen == 0 ? 60.0 : (double)FrameRateNum / FrameRateDen;

        public IReadOnlyList<CoreFirmwareSource> Firmware { get; init; } = Array.Empty<CoreFirmwareSource>();

        // None for a core without MEDIA, which an older core is.
        public IReadOnlyList<CoreDrive> Drives { get; init; } = Array.Empty<CoreDrive>();

        // Since 1.5: the frames after a load whose pictures may differ from the saving machine's; 0 for an older core - see EmuSen_CoreAPI.md §6.9.
        public int PictureSettles { get; init; }

        // The state versions the machine reads, as machine info's state.loads_from lists them; empty for a core that says nothing.
        public IReadOnlyList<long> LoadsFrom { get; init; } = Array.Empty<long>();

        // Since 1.10, RAW_PICTURE: the encodings of the picture before the palette, by index; none for an older core - see EmuSen_CoreAPI.md §6.30.
        public IReadOnlyList<CoreRawEncoding> Raw { get; init; } = Array.Empty<CoreRawEncoding>();

        // Since 1.13: the parts that run no code and hold registers; none for a core that lists none.
        public IReadOnlyList<CoreUnit> Units { get; init; } = Array.Empty<CoreUnit>();

        // Since 1.13, TELEMETRY: the core's own counters; none for a core that lists none.
        public IReadOnlyList<CoreCounter> Telemetry { get; init; } = Array.Empty<CoreCounter>();
    }

    // One setting of a v1 core's schema, its words and trade-off kept for presentation - see EmuSen_CoreAPI.md §6.13.
    public sealed record CoreSettingDescriptor(string Key, string Label, string Help, string Kind, string Default, int Min, int Max, IReadOnlyList<string> Choices,
        bool CreateScope, string? Category, string Effect, string? Cost, bool Advanced, bool Hidden, bool Restart)
    {
        // The words each choice is shown by, in the order of Choices; a value is its own label where the schema gives none.
        public IReadOnlyList<string> ChoiceLabels { get; init; } = Array.Empty<string>();

        // As the settings window lists it, an enhancement in a group of its own; a text setting has no row there yet, so null - see EmuSen_CoreAPI.md §6.13.
        public CoreSetting? AsCoreSetting() => Kind switch
        {
            "switch" => new CoreSetting(Key, Label, Hint, CoreSettingKind.Switch, Default, Group: Group),
            "count" => new CoreSetting(Key, Label, Hint, CoreSettingKind.Count, Default, Min, Max, Group: Group),
            "choice" => new CoreSetting(Key, Label, Hint, CoreSettingKind.Choice, Default, Choices: Choices, ChoiceLabels: ChoiceLabels.Count == Choices.Count ? ChoiceLabels : null, Group: Group),
            _ => null,
        };

        public const string EnhancementsGroup = "Enhancements";

        private string? Group => Effect == "enhancement" ? EnhancementsGroup : null;

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
            Md5 = Strs(f, "md5"),
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
                Arr(s, "firmware").Select(Firmware).ToArray())
            {
                Development = Bool(s, "development", false),
                StartsEmpty = Bool(s, "starts_empty", false),
                Battery = s.TryGetProperty("battery", out var battery) && battery.ValueKind == JsonValueKind.Array ? battery.EnumerateArray().Select(b => new CoreBatteryFile((uint)Long(b, "which"), Str(b, "suffix"), 0, CoreBatteryFile.KindNamed(OptStr(b, "kind")), OptStr(b, "medium"), OptStr(b, "label"), b.TryGetProperty("port", out var port) && port.ValueKind == JsonValueKind.Number ? (int)port.GetInt64() : null)).ToArray() : null,
            }).ToArray();
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
                Arr(r, "battery").Select(b => new CoreBatteryFile((uint)Long(b, "which"), Str(b, "suffix", ".srm"), Long(b, "length"), CoreBatteryFile.KindNamed(OptStr(b, "kind")))).ToArray(),
                Str(state, "format"), Long(state, "version"), kinds.Length == 0 ? new uint[] { 0 } : kinds,
                Strs(r, "phases"),
                patches.ValueKind == JsonValueKind.Object ? Long(patches, "low") : null, patches.ValueKind == JsonValueKind.Object ? Long(patches, "high") : null,
                Bool(r, "skip_rendering_state_neutral"))
            {
                Firmware = Arr(r, "firmware").Select(f => new CoreFirmwareSource((uint)Long(f, "which"), Str(f, "source", "absent"))).ToArray(),
                Drives = Arr(r, "drives").Select(d => new CoreDrive((uint)Long(d, "drive"), Str(d, "label"), Str(d, "kind"), Str(d, "lid") == "open", Bool(d, "loaded"))).ToArray(),
                PictureSettles = (int)Long(state, "picture_settles"),
                LoadsFrom = Arr(state, "loads_from").Select(v => v.GetInt64()).ToArray(),
                Raw = Arr(video, "raw").Select(e => new CoreRawEncoding(Str(e, "encoding"), (int)Long(e, "element"), (int)Long(e, "max_width"), (int)Long(e, "max_height"),
                    Arr(e, "palette").Select(c => (int)c.GetInt64()).ToArray())).ToArray(),
                Units = Arr(r, "units").Select(u => new CoreUnit((uint)Long(u, "id"), Str(u, "name"), Str(u, "kind"), Arr(u, "registers").Select(g => (Str(g, "name"), (int)Long(g, "bits"))).ToArray())).ToArray(),
                Telemetry = Arr(r, "telemetry").Select(k => new CoreCounter(Str(k, "name"), Str(k, "label"), Str(k, "unit"), Str(k, "kind"))).ToArray(),
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
