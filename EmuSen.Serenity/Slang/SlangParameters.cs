using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using EmuSen.Serenity.Native;

namespace EmuSen.Serenity.Slang
{
    // A preset's parameters as its passes declare them, first declaration winning, each with the preset's own value as its default where it gives one - see EmuSen_Serenity.md §7.6.
    public static class SlangParameters
    {
        public static IReadOnlyList<SlangParameter> Merge(IEnumerable<SlangSource> sources, IReadOnlyDictionary<string, float> overrides) =>
            SerenityNative.Active ? MergeNative(sources, overrides) : Managed.Merge(sources, overrides);

        // The library's merge: which parameters, in what order, with what value; each answer is the C# object first declared under its id, so what else it carries stays.
        internal static IReadOnlyList<SlangParameter> MergeNative(IEnumerable<SlangSource> sources, IReadOnlyDictionary<string, float> overrides) =>
            MergeNativeLists(sources.Select(source => source.Parameters).ToList(), overrides);

        // The same over the lists themselves, each source read once before any is merged.
        internal static unsafe IReadOnlyList<SlangParameter> MergeNativeLists(IReadOnlyList<IReadOnlyList<SlangParameter>> declared, IReadOnlyDictionary<string, float> overrides)
        {
            if (declared.Any(list => list.Any(p => !SerenityNative.Crosses(p.Id) || !SerenityNative.Crosses(p.Description))) || overrides.Keys.Any(key => !SerenityNative.Crosses(key)))
                return Managed.Merge(declared, overrides);

            byte[] input = JsonSerializer.SerializeToUtf8Bytes(new
            {
                sources = declared.Select(list => list.Select(ToJson)),
                overrides = overrides.Select(pair => new object[] { pair.Key, SerenityNative.Bits(pair.Value) }),
            });
            using JsonDocument document = SerenityNative.Document((buffer, capacity) =>
            {
                fixed (byte* i = input) return SerenityNative.ParametersMerge(i, (nuint)input.Length, buffer, capacity);
            });
            var first = new Dictionary<string, SlangParameter>(StringComparer.Ordinal);
            foreach (SlangParameter parameter in declared.SelectMany(list => list)) first.TryAdd(parameter.Id, parameter);
            var merged = new List<SlangParameter>();
            foreach (JsonElement element in document.RootElement.EnumerateArray())
            {
                SlangParameter original = first[element.GetProperty("id").GetString()!];
                float initial = SerenityNative.Float(element.GetProperty("initial"));
                merged.Add(SerenityNative.Bits(initial) == SerenityNative.Bits(original.Initial) && !overrides.ContainsKey(original.Id) ? original : original with { Initial = initial });
            }
            return merged;
        }

        private static object ToJson(SlangParameter p) => new
        {
            id = p.Id,
            description = p.Description,
            initial = SerenityNative.Bits(p.Initial),
            minimum = SerenityNative.Bits(p.Minimum),
            maximum = SerenityNative.Bits(p.Maximum),
            step = SerenityNative.Bits(p.Step),
        };

        // The parameters of a document the library answered with.
        internal static IReadOnlyList<SlangParameter> FromJson(JsonElement list)
        {
            var parameters = new List<SlangParameter>();
            foreach (JsonElement p in list.EnumerateArray())
                parameters.Add(new SlangParameter(p.GetProperty("id").GetString()!, p.GetProperty("description").GetString()!, SerenityNative.Float(p.GetProperty("initial")),
                    SerenityNative.Float(p.GetProperty("minimum")), SerenityNative.Float(p.GetProperty("maximum")), SerenityNative.Float(p.GetProperty("step"))));
            return parameters;
        }

        // Reads every pass's source without compiling anything, for a settings window that has no device.
        public static IReadOnlyList<SlangParameter> Read(SlangPreset preset) =>
            Merge(preset.Passes.Select(pass => SlangSource.Load(pass.ShaderPath)), preset.Parameters);

        // A parameter whose range has no width can only be a heading, which is how many presets label their groups.
        public static bool IsHeading(SlangParameter parameter) => parameter.Maximum <= parameter.Minimum;

        // The C# merge: the default, and what the library's is held to - see EmuSen_RustPlatform.md §16.
        internal static class Managed
        {
            public static IReadOnlyList<SlangParameter> Merge(IEnumerable<SlangSource> sources, IReadOnlyDictionary<string, float> overrides) =>
                Merge(sources.Select(source => source.Parameters), overrides);

            public static IReadOnlyList<SlangParameter> Merge(IEnumerable<IReadOnlyList<SlangParameter>> sources, IReadOnlyDictionary<string, float> overrides)
            {
                var merged = new List<SlangParameter>();
                var seen = new HashSet<string>(StringComparer.Ordinal);
                foreach (IReadOnlyList<SlangParameter> source in sources)
                    foreach (SlangParameter parameter in source)
                        if (seen.Add(parameter.Id)) merged.Add(overrides.TryGetValue(parameter.Id, out float value) ? parameter with { Initial = value } : parameter);
                return merged;
            }
        }
    }
}
