using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // One view's element definitions as parsed, merged by type and name, last value winning - see EmuSen_BigPicture.md §12.2.
    internal sealed class ViewBuilder
    {
        private readonly List<ElementBuilder> _elements = [];
        private readonly Dictionary<(string Type, string Name), ElementBuilder> _byKey = [];

        public ViewBuilder(string name) => Name = name;

        public string Name { get; }

        public void Merge(ThemeElementSpec spec, string name, ref int order, List<(string Name, RawProperty Value, string? Key)> properties)
        {
            if (!_byKey.TryGetValue((spec.Type, name), out ElementBuilder? element))
            {
                element = new ElementBuilder(spec, name, order++);
                _byKey[(spec.Type, name)] = element;
                _elements.Add(element);
            }
            foreach ((string property, RawProperty value, string? key) in properties)
            {
                if (key is null) element.Properties[property] = value;
                else
                {
                    if (!element.Keyed.TryGetValue(property, out Dictionary<string, RawProperty>? byKey))
                        element.Keyed[property] = byKey = new Dictionary<string, RawProperty>(StringComparer.Ordinal);
                    byKey[key] = value;
                }
            }
        }

        public ResolvedView Resolve(ParseRun run, ThemeDiagnostics diagnostics)
        {
            var resolved = new List<ResolvedElement>();
            ResolvedElement? primary = null;
            foreach (ElementBuilder element in _elements)
            {
                if (!Allows(element.Spec))
                {
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.ViewRestricted, element.FirstFile, element.FirstLine,
                        $"<{element.Spec.Type} name=\"{element.Name}\"> cannot be used in the {Name} view and is ignored");
                    continue;
                }
                ResolvedElement? r = element.Resolve(Name, run, diagnostics);
                if (r is null) continue;
                if (r.Spec.Group == ThemeElementGroup.Primary)
                {
                    if (primary is not null)
                    {
                        diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.SecondPrimary, element.FirstFile, element.FirstLine,
                            $"{r} is a second primary element in the {Name} view after {primary}, and is ignored");
                        continue;
                    }
                    primary = r;
                }
                resolved.Add(r);
            }
            List<ResolvedElement> ordered = resolved
                .OrderBy(e => e.ZIndex ?? float.PositiveInfinity)
                .ThenBy(e => e.Order)
                .ToList();
            return new ResolvedView(Name, ordered);
        }

        private bool Allows(ThemeElementSpec spec) => Name switch
        {
            "system" => spec.Views.HasFlag(ThemeElementViews.System),
            "gamelist" => spec.Views.HasFlag(ThemeElementViews.Gamelist),
            _ => spec.Views.HasFlag(ThemeElementViews.All),
        };
    }

    internal sealed class ElementBuilder
    {
        public ElementBuilder(ThemeElementSpec spec, string name, int order)
        {
            Spec = spec;
            Name = name;
            Order = order;
        }

        public ThemeElementSpec Spec { get; }
        public string Name { get; }
        public int Order { get; }
        public Dictionary<string, RawProperty> Properties { get; } = new(StringComparer.Ordinal);
        public Dictionary<string, Dictionary<string, RawProperty>> Keyed { get; } = new(StringComparer.Ordinal);

        public string FirstFile => Properties.Values.Concat(Keyed.Values.SelectMany(k => k.Values)).FirstOrDefault()?.File ?? "";
        public int FirstLine => Properties.Values.Concat(Keyed.Values.SelectMany(k => k.Values)).FirstOrDefault()?.Line ?? 0;

        // Types each value, applies view limits and ranges, then fills documented defaults; null when the element must not render.
        public ResolvedElement? Resolve(string view, ParseRun run, ThemeDiagnostics diagnostics)
        {
            var explicitValues = new Dictionary<string, ThemeValue>(StringComparer.Ordinal);
            bool render = true;

            foreach ((string name, RawProperty raw) in Properties)
            {
                ThemePropertySpec spec = Spec.Find(name)!;
                if (!InView(spec, view))
                {
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.ViewRestricted, raw.File, raw.Line,
                        $"<{name}> of {Spec.Type} \"{Name}\" can only be used in the {spec.OnlyIn.ToString().ToLowerInvariant()} view and is ignored");
                    continue;
                }
                ThemeValue? value = Parse(spec, raw, diagnostics, ref render);
                if (value is not null) explicitValues[name] = value;
            }

            foreach ((string name, Dictionary<string, RawProperty> byKey) in Keyed)
            {
                var paths = new Dictionary<string, ThemePath>(StringComparer.Ordinal);
                foreach ((string key, RawProperty raw) in byKey)
                    paths[key] = ResolvePath(raw, diagnostics);
                explicitValues[name] = new KeyedPathsValue(paths);
            }

            if (!render) return null;

            var effective = new Dictionary<string, ThemeValue>(explicitValues, StringComparer.Ordinal);
            foreach (ThemePropertySpec spec in Spec.Properties)
                if (!effective.ContainsKey(spec.Name) && InView(spec, view) && Default(spec, effective, run) is { } d)
                    effective[spec.Name] = d;

            float? zIndex = effective.TryGetValue("zIndex", out ThemeValue? z) && z is FloatValue f ? f.Value : null;
            return new ResolvedElement(Spec, Name, Order, zIndex, explicitValues, effective, Bindings(view, effective));
        }

        private static bool InView(ThemePropertySpec spec, string view) => spec.OnlyIn switch
        {
            ThemeViewScope.System => view == "system",
            ThemeViewScope.Gamelist => view == "gamelist",
            _ => true,
        };

        private ThemeValue? Parse(ThemePropertySpec spec, RawProperty raw, ThemeDiagnostics diagnostics, ref bool render)
        {
            string text = raw.Text.Trim();
            void BadFormat(string expected) => diagnostics.Add(ThemeSeverity.Error, ThemeDiagnosticCode.BadFormat, raw.File, raw.Line,
                $"<{spec.Name}> of {Spec.Type} \"{Name}\" is \"{text}\", not {expected}");

            switch (spec.Type)
            {
                case ThemePropertyType.NormalizedPair:
                    if (!ThemeValueParser.TryParsePair(text, out NormalizedPair pair)) { BadFormat("a normalised pair"); return null; }
                    return new PairValue(ThemeValueParser.ClampPair(pair, spec));
                case ThemePropertyType.Path:
                    return new PathValue(ResolvePath(raw, diagnostics));
                case ThemePropertyType.Boolean:
                    if (!ThemeValueParser.TryParseBool(text, out bool b)) { BadFormat("a boolean"); return null; }
                    return new BoolValue(b);
                case ThemePropertyType.Color:
                    if (!ThemeColor.TryParse(text, out ThemeColor color)) { BadFormat("a 6 or 8 digit colour"); return null; }
                    return new ColorValue(color);
                case ThemePropertyType.UnsignedInteger:
                    if (!uint.TryParse(text, NumberStyles.None, CultureInfo.InvariantCulture, out uint u)) { BadFormat("an unsigned integer"); return null; }
                    return new UIntValue((uint)ThemeValueParser.Clamp(u, spec.Min, spec.Max));
                case ThemePropertyType.Float:
                    if (!ThemeValueParser.TryParseFloat(text, out float f)) { BadFormat("a number"); return null; }
                    return new FloatValue(ThemeValueParser.Clamp(f, spec.Min, spec.Max));
            }

            switch (spec.Shape)
            {
                case ThemeValueShape.Enum:
                    if (spec.Values.Contains(text)) return new StringValue(text);
                    diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, raw.File, raw.Line,
                        $"<{spec.Name}> of {Spec.Type} \"{Name}\" is \"{text}\", not one of {string.Join(", ", spec.Values)}; the default applies");
                    return null;
                case ThemeValueShape.List:
                    return ParseList(spec, raw, text, diagnostics, ref render);
                default:
                    return new StringValue(raw.Text);
            }
        }

        // imageType is strict: an unknown type stops the element rendering, a repeated one drops the property - see EmuSen_BigPicture.md §12.3.
        private ThemeValue? ParseList(ThemePropertySpec spec, RawProperty raw, string text, ThemeDiagnostics diagnostics, ref bool render)
        {
            IReadOnlyList<string> items = ThemeLists.Split(text);
            bool imageType = spec.Name == "imageType";
            List<string> invalid = items.Where(i => !spec.Values.Contains(i)).ToList();
            if (invalid.Count > 0)
            {
                if (imageType)
                {
                    diagnostics.Add(ThemeSeverity.Error, ThemeDiagnosticCode.InvalidImageType, raw.File, raw.Line,
                        $"<imageType> of {Spec.Type} \"{Name}\" names {string.Join(", ", invalid)}; the element is not rendered");
                    render = false;
                    return null;
                }
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, raw.File, raw.Line,
                    $"<{spec.Name}> of {Spec.Type} \"{Name}\" names unknown {string.Join(", ", invalid)}, which are ignored");
                items = items.Where(spec.Values.Contains).ToArray();
            }

            if (imageType && items.Distinct().Count() != items.Count)
            {
                diagnostics.Add(ThemeSeverity.Warning, ThemeDiagnosticCode.InvalidValue, raw.File, raw.Line,
                    $"<imageType> of {Spec.Type} \"{Name}\" repeats a type and is ignored");
                return null;
            }
            if (spec.MaxItems > 0 && items.Count > spec.MaxItems)
            {
                diagnostics.Add(ThemeSeverity.Debug, ThemeDiagnosticCode.InvalidValue, raw.File, raw.Line,
                    $"<{spec.Name}> of {Spec.Type} \"{Name}\" lists {items.Count} entries; only the first {spec.MaxItems} are used");
                items = items.Take(spec.MaxItems).ToArray();
            }
            return new ListValue(items);
        }

        // A missing file is a warning when written out and a debug note when built from a variable.
        private static ThemePath ResolvePath(RawProperty raw, ThemeDiagnostics diagnostics)
        {
            string absolute = ThemeValueParser.ResolvePath(raw.Text, raw.File);
            bool exists = File.Exists(absolute) || Directory.Exists(absolute);
            if (!exists)
                diagnostics.Add(raw.FromVariable ? ThemeSeverity.Debug : ThemeSeverity.Warning, ThemeDiagnosticCode.PathMissing, raw.File, raw.Line,
                    $"\"{raw.Text.Trim()}\" not found (resolved to {absolute})");
            return new ThemePath(raw.Text.Trim(), absolute, raw.FromVariable, exists);
        }

        private ThemeValue? Default(ThemePropertySpec spec, Dictionary<string, ThemeValue> effective, ParseRun run)
        {
            switch (spec.Computed)
            {
                case ComputedDefault.SystemFullName:
                    return new StringValue(run.System.FullName);
                case ComputedDefault.ElementWidth:
                    return Value("size", effective, run) is PairValue size ? new FloatValue(size.Value.X) : null;
                case ComputedDefault.OneAndAHalfFontSize:
                    return Value("fontSize", effective, run) is FloatValue font ? new FloatValue(font.Value * 1.5f) : null;
                case ComputedDefault.HelpPosByOrientation:
                    return new PairValue(run.Selection.Vertical ? new NormalizedPair(0.012f, 0.975f) : new NormalizedPair(0.012f, 0.9515f));
                case ComputedDefault.HelpFontSizeByOrientation:
                    return new FloatValue(run.Selection.Vertical ? 0.025f : 0.035f);
                case ComputedDefault.InterpolationByRotation:
                    float rotation = Value("rotation", effective, run) is FloatValue r ? r.Value : 0;
                    return new StringValue(((rotation % 90) + 90) % 90 == 0 ? "nearest" : "linear");
                case ComputedDefault.ContainerForDescription:
                    return new BoolValue(Value("metadata", effective, run) is StringValue { Value: "description" });
                case ComputedDefault.DisplayRelativeForLastPlayed:
                    return new BoolValue(Value("metadata", effective, run) is StringValue { Value: "lastplayed" });
                case ComputedDefault.ContainerStartDelayByType:
                    return new FloatValue(Value("containerType", effective, run) is StringValue { Value: "horizontal" } ? 1.5f : 4.5f);
            }

            if (spec.DefaultSameAs is { } other) return Value(other, effective, run);
            if (spec.Default is null) return null;
            var none = new ThemeDiagnostics();
            bool render = true;
            return Parse(spec, new RawProperty(spec.Default, "", 0, false), none, ref render);
        }

        // A property's effective value, filling and caching its own default first when the theme did not set it.
        private ThemeValue? Value(string name, Dictionary<string, ThemeValue> effective, ParseRun run)
        {
            if (effective.TryGetValue(name, out ThemeValue? value)) return value;
            if (Spec.Find(name) is not { } spec) return null;
            ThemeValue? d = Default(spec, effective, run);
            if (d is not null) effective[name] = d;
            return d;
        }

        // What the element will show that the theme does not supply - see EmuSen_BigPicture.md §12.2.
        private IReadOnlyList<ElementBinding> Bindings(string view, Dictionary<string, ThemeValue> effective)
        {
            var bindings = new List<ElementBinding>();
            IReadOnlyList<string> List(string name) => effective.GetValueOrDefault(name) is ListValue l ? l.Items : [];
            string? Text(string name) => effective.GetValueOrDefault(name) is StringValue s ? s.Value : null;
            IReadOnlyList<string> Media(IReadOnlyList<string> types) => types.SelectMany(t => t == "image" ? ThemeValueParser.ImageShortcut : [t]).Distinct().ToArray();

            switch (Spec.Type)
            {
                case "image":
                    if (List("imageType") is { Count: > 0 } imageTypes) bindings.Add(new ElementBinding("media", Media(imageTypes)));
                    else if (effective.ContainsKey("gameOverridePath")) bindings.Add(new ElementBinding("gameOverride", []));
                    break;
                case "video":
                    bindings.Add(new ElementBinding("videoFallbackImage", Media(List("imageType").Where(t => t != "none").ToArray())));
                    break;
                case "carousel":
                case "grid":
                    bindings.Add(new ElementBinding(view == "system" ? "systems" : "games", []));
                    if (view == "gamelist") bindings.Add(new ElementBinding("media", List("imageType").Where(t => t != "none").ToArray()));
                    break;
                case "textlist":
                    bindings.Add(new ElementBinding(view == "system" ? "systems" : "games", []));
                    break;
                case "text":
                    if (Text("metadata") is { } metadata) bindings.Add(new ElementBinding("metadata", [metadata]));
                    else if (Text("systemdata") is { } systemdata) bindings.Add(new ElementBinding("systemdata", [systemdata]));
                    break;
                case "datetime":
                    if (Text("metadata") is { } date) bindings.Add(new ElementBinding("metadata", [date]));
                    break;
                case "rating":
                    bindings.Add(new ElementBinding("metadata", ["rating"]));
                    break;
                case "badges":
                    bindings.Add(new ElementBinding("badges", List("slots")));
                    break;
                case "gamelistinfo":
                    bindings.Add(new ElementBinding("gamelistinfo", []));
                    break;
                case "helpsystem":
                    bindings.Add(new ElementBinding("help", List("entries")));
                    break;
                case "clock":
                    bindings.Add(new ElementBinding("clock", []));
                    break;
                case "systemstatus":
                    bindings.Add(new ElementBinding("systemstatus", List("entries")));
                    break;
                case "gameselector":
                    bindings.Add(new ElementBinding("gameselector", Text("selection") is { } s ? [s] : []));
                    break;
            }
            return bindings;
        }
    }
}
