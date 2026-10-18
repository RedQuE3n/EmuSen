using System;
using System.Collections.Generic;
using EmuSen.Serenity.Slang;

namespace EmuSen.Serenity.Shaders
{
    // How big a pass draws: the game's own picture, or the rectangle it is shown in - see EmuSen_Serenity.md §3.2.
    public enum PassScale { Source, Viewport }

    // What one axis of a pass is measured against: the frame, the rectangle shown, or a count of its own - see EmuSen_Serenity.md §3.9.
    public enum PassAxis { Source, Viewport, Fixed }

    // One axis of a pass's size: the frame's or the rectangle's times a factor, or a fixed count - see EmuSen_Serenity.md §3.9.
    public readonly record struct PassSize(PassAxis Axis, float Factor = 1f)
    {
        public static PassSize Fixed(int count) => new(PassAxis.Fixed, count);

        public int Of(int source, int viewport) => Math.Max(1, (int)MathF.Round(Axis switch
        {
            PassAxis.Source => source * Factor,
            PassAxis.Viewport => viewport * Factor,
            _ => Factor,
        }));
    }

    // One pass in Vulkan GLSL; History is how many earlier frames it reads, as history1 to historyN - see EmuSen_DevicePicture.md §20.
    public sealed record FilterPass(string Glsl, PassScale Scale, bool LinearSource = false, int History = 0)
    {
        // Either axis sized apart from Scale; null keeps Scale's.
        public PassSize? Width { get; init; }
        public PassSize? Height { get; init; }

        // Drawn into a half-float surface, which keeps values outside 0 to 1 and all four channels as written.
        public bool Float { get; init; }

        // Reads its own output of the game's previous frame as the child "feedback".
        public bool Feedback { get; init; }

        // The child name later passes read this pass's output by, with a uniform of that name plus "Size".
        public string? Name { get; init; }

        // Children besides "source" that are sampled linearly.
        public IReadOnlyList<string>? Linear { get; init; }
    }

    // A screen filter: its passes, the consoles it suits (null for every one), whose work it rests on, and the uniforms a player may set - see EmuSen_Serenity.md §3.3, §3.7 and §3.9.
    public sealed record ScreenFilter(string Name, IReadOnlyList<FilterPass> Passes, IReadOnlyList<string>? Consoles, string Credit, IReadOnlyList<SlangParameter>? Parameters = null)
    {
        // Passes made from the parameters' values, in place of Passes; made again when one of Structural changes.
        public Func<IReadOnlyDictionary<string, float>, IReadOnlyList<FilterPass>>? Build { get; init; }

        // The parameters whose value decides which passes exist, and so cannot be a uniform.
        public IReadOnlyList<string>? Structural { get; init; }

        // The encoding of the picture before the palette that a pass declaring the child "raw" reads, or null - see EmuSen_CRT.md §22.1.
        public string? RawEncoding { get; init; }

        // A parameter's default on one console, where it differs from the parameter's own.
        public IReadOnlyDictionary<string, IReadOnlyDictionary<string, float>>? ConsoleDefaults { get; init; }

        // Width over height of the rectangle the picture is shown in; null is the frame's own pixel count.
        public double? Aspect { get; init; }

        // Places the part of the picture kept itself, from the uniform "crop" (left, top, width and height as shares of the frame) - see EmuSen_Serenity.md §2.10.
        public bool CropsItself { get; init; }

        // The frame's rows drawn once each, where a core hands them over to be repeated; a filter that models the screen counts scanlines - see EmuSen_Serenity.md §3.9.
        public bool RowsOnce { get; init; }

        // How many earlier frames any variant of the filter may read.
        public int HistoryDepth { get; init; }

        public int History
        {
            get
            {
                int most = HistoryDepth;
                foreach (FilterPass pass in Passes) if (pass.History > most) most = pass.History;
                return most;
            }
        }

        // The entry of ConsoleDefaults that any console without its own takes.
        public const string AnyConsole = "";

        // A console's values: its own entry, or the one for any console, or none.
        public IReadOnlyDictionary<string, float>? DefaultsFor(string? console) =>
            ConsoleDefaults is null ? null
            : console is not null && ConsoleDefaults.TryGetValue(console, out var own) ? own
            : ConsoleDefaults.TryGetValue(AnyConsole, out var any) ? any : null;

        // Every value the passes read on a console: each parameter's given value or its default there, then the console's constants.
        public Dictionary<string, float> ValuesFor(string? console, IReadOnlyDictionary<string, float>? given)
        {
            var values = new Dictionary<string, float>(StringComparer.Ordinal);
            foreach (SlangParameter parameter in Parameters ?? Array.Empty<SlangParameter>())
                values[parameter.Id] = given is not null && given.TryGetValue(parameter.Id, out float value) ? value : DefaultFor(parameter, console);
            if (DefaultsFor(console) is { } constants) foreach (var (id, value) in constants) values.TryAdd(id, value);
            return values;
        }

        // Whether a pass made for this console and these values reads RawEncoding, so a frontend asks its core for that encoding.
        public bool ReadsRaw(string? console, IReadOnlyDictionary<string, float>? given) =>
            RawEncoding is not null && (Build?.Invoke(ValuesFor(console, given)) ?? Passes).Any(pass => pass.Glsl.Contains("uniform sampler2D raw;", StringComparison.Ordinal));

        // What a parameter starts at on a console: the console's value where it has one, the parameter's own otherwise.
        public float DefaultFor(SlangParameter parameter, string? console) =>
            DefaultsFor(console) is { } values && values.TryGetValue(parameter.Id, out float value) ? value : parameter.Initial;
    }
}
