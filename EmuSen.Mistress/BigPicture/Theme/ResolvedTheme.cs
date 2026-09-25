using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // Data an element shows that is not in the theme: a game's media, its metadata, the clock - see EmuSen_BigPicture.md §12.2.
    public sealed record ElementBinding(string Kind, IReadOnlyList<string> Names)
    {
        public override string ToString() => Names.Count == 0 ? Kind : $"{Kind}:{string.Join(",", Names)}";
    }

    // One element after merging, typing and defaults; immutable.
    public sealed class ResolvedElement
    {
        internal ResolvedElement(ThemeElementSpec spec, string name, int order, float? zIndex,
            IReadOnlyDictionary<string, ThemeValue> explicitValues, IReadOnlyDictionary<string, ThemeValue> effective, IReadOnlyList<ElementBinding> bindings)
        {
            Spec = spec;
            Name = name;
            Order = order;
            ZIndex = zIndex;
            Explicit = explicitValues;
            Effective = effective;
            Bindings = bindings;
        }

        public ThemeElementSpec Spec { get; }
        public string Type => Spec.Type;
        public string Name { get; }

        // The order of the element's first definition, which breaks zIndex ties.
        public int Order { get; }

        // Null for helpsystem, clock and systemstatus, which draw above everything.
        public float? ZIndex { get; }

        // What the theme set, typed.
        public IReadOnlyDictionary<string, ThemeValue> Explicit { get; }

        // What the theme set, plus every documented default that has a value.
        public IReadOnlyDictionary<string, ThemeValue> Effective { get; }

        public IReadOnlyList<ElementBinding> Bindings { get; }

        public ThemeValue? Get(string property) => Effective.GetValueOrDefault(property);
        public float? Float(string property) => Get(property) is FloatValue f ? f.Value : null;
        public uint? UInt(string property) => Get(property) is UIntValue u ? u.Value : null;
        public bool? Bool(string property) => Get(property) is BoolValue b ? b.Value : null;
        public NormalizedPair? Pair(string property) => Get(property) is PairValue p ? p.Value : null;
        public ThemeColor? Color(string property) => Get(property) is ColorValue c ? c.Value : null;
        public ThemePath? Path(string property) => Get(property) is PathValue p ? p.Value : null;
        public string? String(string property) => Get(property) is StringValue s ? s.Value : null;
        public IReadOnlyList<string> List(string property) => Get(property) is ListValue l ? l.Items : [];
        public IReadOnlyDictionary<string, ThemePath> Keyed(string property) => Get(property) is KeyedPathsValue k ? k.Paths : new Dictionary<string, ThemePath>();

        public override string ToString() => $"{Type} \"{Name}\"";
    }

    // One view's elements in drawing order: zIndex low to high, ties by first definition, the unindexed last.
    public sealed class ResolvedView
    {
        internal ResolvedView(string name, IReadOnlyList<ResolvedElement> elements)
        {
            Name = name;
            Elements = elements;
        }

        public string Name { get; }
        public IReadOnlyList<ResolvedElement> Elements { get; }

        public ResolvedElement? Find(string type, string name) => Elements.FirstOrDefault(e => e.Type == type && e.Name == name);

        public IEnumerable<ResolvedElement> OfType(string type) => Elements.Where(e => e.Type == type);

        public ResolvedElement? Primary => Elements.FirstOrDefault(e => e.Spec.Group == ThemeElementGroup.Primary);
    }

    // A theme resolved for one system and one set of choices - see EmuSen_BigPicture.md §12.2.
    public sealed class ResolvedTheme
    {
        internal ResolvedTheme(ThemeCapabilities capabilities, ThemeSystem system, ThemeSelection selection, string? gamelistVariant,
            string? entryFile, ResolvedView systemView, ResolvedView gamelistView, ResolvedView soundView,
            TransitionProfile transitions, IReadOnlyList<ThemeDiagnostic> diagnostics, IReadOnlyList<string> filesRead)
        {
            Capabilities = capabilities;
            System = system;
            Selection = selection;
            GamelistVariant = gamelistVariant;
            EntryFile = entryFile;
            SystemView = systemView;
            GamelistView = gamelistView;
            SoundView = soundView;
            Transitions = transitions;
            Diagnostics = diagnostics;
            FilesRead = filesRead;
        }

        public ThemeCapabilities Capabilities { get; }
        public ThemeSystem System { get; }
        public ThemeSelection Selection { get; }

        // The variant the gamelist view was built with, after the triggers; the system view uses Selection.Variant.
        public string? GamelistVariant { get; }

        public string? EntryFile { get; }
        public ResolvedView SystemView { get; }
        public ResolvedView GamelistView { get; }
        // The all view, which holds only the navigation sounds.
        public ResolvedView SoundView { get; }

        public IReadOnlyDictionary<string, ThemePath> Sounds => SoundView.OfType("sound").Where(s => s.Path("path") is not null).ToDictionary(s => s.Name, s => s.Path("path")!);

        public IEnumerable<ResolvedElement> SoundElements => SoundView.Elements;
        public TransitionProfile Transitions { get; }
        public IReadOnlyList<ThemeDiagnostic> Diagnostics { get; }
        public IReadOnlyList<string> FilesRead { get; }

        // An error unthemes the system; the views stay readable for inspection but must not be drawn.
        public bool IsThemed => Diagnostics.All(d => d.Severity != ThemeSeverity.Error);

        public IEnumerable<ThemeDiagnostic> Errors => Diagnostics.Where(d => d.Severity == ThemeSeverity.Error);

        public ResolvedView View(string name) => name switch
        {
            "system" => SystemView,
            "gamelist" => GamelistView,
            _ => throw new ArgumentOutOfRangeException(nameof(name), name, "a view is system or gamelist"),
        };
    }
}
