using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Serenity.Slang
{
    // A preset's parameters as its passes declare them, first declaration winning, each with the preset's own value as its default where it gives one - see EmuSen_Serenity.md §7.6.
    public static class SlangParameters
    {
        public static IReadOnlyList<SlangParameter> Merge(IEnumerable<SlangSource> sources, IReadOnlyDictionary<string, float> overrides)
        {
            var merged = new List<SlangParameter>();
            var seen = new HashSet<string>(StringComparer.Ordinal);
            foreach (SlangSource source in sources)
                foreach (SlangParameter parameter in source.Parameters)
                    if (seen.Add(parameter.Id)) merged.Add(overrides.TryGetValue(parameter.Id, out float value) ? parameter with { Initial = value } : parameter);
            return merged;
        }

        // Reads every pass's source without compiling anything, for a settings window that has no device.
        public static IReadOnlyList<SlangParameter> Read(SlangPreset preset) =>
            Merge(preset.Passes.Select(pass => SlangSource.Load(pass.ShaderPath)), preset.Parameters);

        // A parameter whose range has no width can only be a heading, which is how many presets label their groups.
        public static bool IsHeading(SlangParameter parameter) => parameter.Maximum <= parameter.Minimum;
    }
}
