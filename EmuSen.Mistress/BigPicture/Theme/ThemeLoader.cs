using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text.RegularExpressions;
using System.Xml;
using System.Xml.Linq;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // Loads an ES-DE theme for one system and one set of choices - see EmuSen_BigPicture.md §12.2 and §12.3.
    public static class ThemeLoader
    {
        public const int MaxIncludeDepth = 64;

        public static ResolvedTheme Load(string themeDirectory, ThemeSystem system, ThemeChoices choices, MediaPresence? media = null) =>
            Load(ThemeCapabilitiesReader.Read(themeDirectory), system, choices, media);

        public static ResolvedTheme Load(ThemeCapabilities capabilities, ThemeSystem system, ThemeChoices choices, MediaPresence? media = null)
        {
            var diagnostics = new ThemeDiagnostics();
            diagnostics.AddRange(capabilities.Diagnostics);
            ThemeSelection selection = ThemeSelection.Resolve(capabilities, choices);
            string? gamelistVariant = ThemeSelection.ApplyTriggers(capabilities, selection.Variant, media, choices.VariantTriggers, diagnostics);

            string? entry = EntryFile(capabilities.Directory, system);
            bool capabilitiesFailed = capabilities.Diagnostics.Any(d => d.Severity == ThemeSeverity.Error);
            if (entry is null || capabilitiesFailed)
            {
                if (entry is null)
                    diagnostics.Add(ThemeSeverity.Error, ThemeDiagnosticCode.NoThemeFile, capabilities.Directory, 0,
                        $"neither {system.Theme}/theme.xml nor theme.xml exists");
                var empty = new ResolvedView("system", []);
                return new ResolvedTheme(capabilities, system, selection, gamelistVariant, entry, empty, new ResolvedView("gamelist", []),
                    new ResolvedView("all", []), ResolveTransitions(capabilities, selection, null), diagnostics.Items, []);
            }

            ParseRun forSystem = ParseRun.Run(capabilities, selection, selection.Variant, system, entry);
            ParseRun forGamelist = gamelistVariant == selection.Variant
                ? forSystem
                : ParseRun.Run(capabilities, selection, gamelistVariant, system, entry);

            var seen = new HashSet<ThemeDiagnostic>();
            foreach (ThemeDiagnostic d in forSystem.Diagnostics.Items.Concat(forGamelist.Diagnostics.Items))
                if (seen.Add(d)) diagnostics.Add(d.Severity, d.Code, d.File, d.Line, d.Message);

            ResolvedView systemView = forSystem.Finish("system", diagnostics, seen);
            ResolvedView gamelistView = forGamelist.Finish("gamelist", diagnostics, seen);
            ResolvedView sounds = forSystem.Finish("all", diagnostics, seen);

            return new ResolvedTheme(capabilities, system, selection, gamelistVariant, entry, systemView, gamelistView, sounds,
                ResolveTransitions(capabilities, selection, forSystem.VariantTransitions), diagnostics.Items,
                forSystem.FilesRead.Union(forGamelist.FilesRead).ToArray());
        }

        // A system's own folder's theme.xml, else the theme's root one.
        public static string? EntryFile(string themeDirectory, ThemeSystem system)
        {
            string own = Path.Combine(themeDirectory, system.Theme, "theme.xml");
            if (system.Theme.Length > 0 && File.Exists(own)) return Path.GetFullPath(own);
            string root = Path.Combine(themeDirectory, "theme.xml");
            return File.Exists(root) ? Path.GetFullPath(root) : null;
        }

        // A chosen profile, else Automatic: the variant's, the first declared, then instant - see EmuSen_BigPicture.md §12.2.
        public static TransitionProfile ResolveTransitions(ThemeCapabilities capabilities, ThemeSelection selection, string? variantProfile)
        {
            string? choice = selection.TransitionsChoice;
            if (choice is not null && ThemeCapabilities.BuiltInTransitions.Contains(choice) && !capabilities.SuppressedTransitions.Contains(choice))
                return BuiltIn(choice);
            if (choice is not null && capabilities.Transitions.FirstOrDefault(t => t.Name == choice) is { } chosen)
                return chosen;
            if (variantProfile is not null && capabilities.Transitions.FirstOrDefault(t => t.Name == variantProfile) is { } forVariant)
                return forVariant;
            return capabilities.Transitions.FirstOrDefault() ?? BuiltIn("builtin-instant");
        }

        private static TransitionProfile BuiltIn(string name)
        {
            TransitionAnimation a = name switch
            {
                "builtin-slide" => TransitionAnimation.Slide,
                "builtin-fade" => TransitionAnimation.Fade,
                _ => TransitionAnimation.Instant,
            };
            return new TransitionProfile(name, new Dictionary<string, string>(), true, a, a, a, a, a, a);
        }
    }

    // One pass over a theme's files for one variant, in THEMES.md's parsing order - see EmuSen_BigPicture.md §12.2.
    internal sealed class ParseRun
    {
        private static readonly Regex Reference = new(@"\$\{([^}]*)\}", RegexOptions.Compiled);
        private static readonly string[] ViewNames = ["system", "gamelist", "all"];

        private readonly ThemeCapabilities _capabilities;
        private readonly ThemeSelection _selection;
        private readonly string? _variant;
        private readonly ThemeSystem _system;
        private readonly Dictionary<string, string> _variables;
        private readonly Dictionary<string, ViewBuilder> _views = new(StringComparer.Ordinal);
        private readonly List<string> _includeStack = [];
        private readonly HashSet<string> _warnedNames = new(StringComparer.Ordinal);
        private int _order;

        private ParseRun(ThemeCapabilities capabilities, ThemeSelection selection, string? variant, ThemeSystem system)
        {
            _capabilities = capabilities;
            _selection = selection;
            _variant = variant;
            _system = system;
            _variables = new Dictionary<string, string>(system.Variables(), StringComparer.Ordinal);
            foreach (string view in ViewNames) _views[view] = new ViewBuilder(view);
        }

        public ThemeDiagnostics Diagnostics { get; } = new();
        public List<string> FilesRead { get; } = [];
        public string? VariantTransitions { get; private set; }
        public IReadOnlyDictionary<string, string> Variables => _variables;

        public static ParseRun Run(ThemeCapabilities capabilities, ThemeSelection selection, string? variant, ThemeSystem system, string entry)
        {
            var run = new ParseRun(capabilities, selection, variant, system);
            run.ProcessFile(entry, 0);
            return run;
        }

        private enum Block { Theme, Variant, AspectRatio, Language }

        private void ProcessFile(string file, int line)
        {
            if (_includeStack.Contains(file, StringComparer.Ordinal) || _includeStack.Count >= ThemeLoader.MaxIncludeDepth)
            {
                // ES-DE does not check for loops and hangs; the loader refuses the include instead.
                Error(ThemeDiagnosticCode.IncludeLoop, _includeStack.LastOrDefault() ?? file, line, $"including \"{file}\" again would loop");
                return;
            }

            XDocument document;
            try
            {
                document = XDocument.Load(file, LoadOptions.SetLineInfo);
            }
            catch (XmlException e)
            {
                Error(ThemeDiagnosticCode.MalformedXml, file, e.LineNumber, e.Message);
                return;
            }
            FilesRead.Add(file);

            XElement root = document.Root!;
            if (root.Name.LocalName != "theme")
            {
                Error(ThemeDiagnosticCode.WrongRoot, file, Line(root), $"root is <{root.Name.LocalName}>, not <theme>");
                return;
            }

            _includeStack.Add(file);
            ProcessBlock(root, file, Block.Theme);
            _includeStack.RemoveAt(_includeStack.Count - 1);
        }

        private static readonly Dictionary<Block, string[]> Allowed = new()
        {
            [Block.Theme] = ["variables", "colorScheme", "fontSize", "language", "include", "view", "variant", "aspectRatio"],
            [Block.Variant] = ["transitions", "variables", "colorScheme", "fontSize", "language", "include", "view", "aspectRatio"],
            [Block.AspectRatio] = ["variables", "colorScheme", "fontSize", "language", "include", "view"],
            [Block.Language] = ["variables", "include"],
        };

        // Steps 1-9 of "Configuration parsing order", each in file order; an include runs all nine at its place.
        private void ProcessBlock(XElement block, string file, Block kind)
        {
            List<XElement> children = [];
            foreach (XElement child in block.Elements())
            {
                string tag = child.Name.LocalName;
                if (Allowed[kind].Contains(tag))
                {
                    children.Add(child);
                    continue;
                }
                bool known = Allowed[Block.Theme].Contains(tag) || tag == "transitions";
                Error(known ? ThemeDiagnosticCode.MisplacedTag : ThemeDiagnosticCode.UnknownTag, file, Line(child),
                    known ? $"<{tag}> is not allowed inside <{block.Name.LocalName}>" : $"<{tag}> is not a theme tag");
            }

            foreach (XElement t in children.Where(c => c.Name.LocalName == "transitions")) Transitions(t, file);
            foreach (XElement v in children.Where(c => c.Name.LocalName == "variables")) DefineVariables(v, file);
            foreach (XElement c in children.Where(c => c.Name.LocalName == "colorScheme"))
                if (Selected(c, file, "colour scheme", n => _capabilities.ColorSchemes.Any(s => s.Name == n), _selection.ColorScheme)) VariablesOnly(c, file);
            foreach (XElement f in children.Where(c => c.Name.LocalName == "fontSize"))
                if (Selected(f, file, "font size", n => ThemeCapabilities.FontSizeOrder.Contains(n), _selection.FontSize)) VariablesOnly(f, file);
            foreach (XElement l in children.Where(c => c.Name.LocalName == "language"))
                if (Selected(l, file, null, _ => true, _selection.Language)) ProcessBlock(l, file, Block.Language);
            foreach (XElement i in children.Where(c => c.Name.LocalName == "include")) Include(i, file);
            foreach (XElement v in children.Where(c => c.Name.LocalName == "view")) View(v, file);
            foreach (XElement v in children.Where(c => c.Name.LocalName == "variant"))
                if (VariantApplies(v, file)) ProcessBlock(v, file, Block.Variant);
            foreach (XElement a in children.Where(c => c.Name.LocalName == "aspectRatio"))
                if (Selected(a, file, "aspect ratio", n => ThemeCapabilities.Ratio(n) is not null, _selection.AspectRatio)) ProcessBlock(a, file, Block.AspectRatio);
        }

        private void Transitions(XElement element, string file)
        {
            string name = element.Value.Trim();
            if (_capabilities.Transitions.Any(t => t.Name == name)) VariantTransitions = name;
            else Warn(ThemeDiagnosticCode.Undeclared, file, Line(element), $"transitions profile \"{name}\" is not declared in capabilities.xml");
        }

        // Variables live in one namespace, and a later definition replaces an earlier one.
        private void DefineVariables(XElement variables, string file)
        {
            foreach (XElement variable in variables.Elements())
                _variables[variable.Name.LocalName] = Substitute(variable.Value, out _);
        }

        // Colour schemes and font sizes hold variables only; a bare child is taken as a variable, as THEMES.md's own example writes it.
        private void VariablesOnly(XElement block, string file)
        {
            foreach (XElement child in block.Elements())
            {
                if (child.Name.LocalName == "variables")
                {
                    DefineVariables(child, file);
                    continue;
                }
                Warn(ThemeDiagnosticCode.MisplacedTag, file, Line(child), $"<{child.Name.LocalName}> directly inside <{block.Name.LocalName}> is taken as a variable");
                _variables[child.Name.LocalName] = Substitute(child.Value, out _);
            }
        }

        private bool Selected(XElement block, string file, string? what, Func<string, bool> known, string? chosen)
        {
            IReadOnlyList<string> names = Names(block, file);
            if (what is not null)
            {
                foreach (string name in names.Where(n => !known(n)))
                    if (_warnedNames.Add(what + ":" + name))
                        Warn(ThemeDiagnosticCode.Undeclared, file, Line(block), $"{what} \"{name}\" is not declared");
            }
            return chosen is not null && names.Contains(chosen);
        }

        private bool VariantApplies(XElement block, string file)
        {
            IReadOnlyList<string> names = Names(block, file);
            foreach (string name in names.Where(n => n != "all" && !_capabilities.IsDeclaredVariant(n)))
                if (_warnedNames.Add("variant:" + name))
                    Warn(ThemeDiagnosticCode.Undeclared, file, Line(block), $"variant \"{name}\" is not declared in capabilities.xml");
            return names.Contains("all") || (_variant is not null && names.Contains(_variant));
        }

        private IReadOnlyList<string> Names(XElement block, string file)
        {
            IReadOnlyList<string> names = ThemeLists.Split(block.Attribute("name")?.Value);
            if (names.Count == 0) Error(ThemeDiagnosticCode.MissingName, file, Line(block), $"<{block.Name.LocalName}> has no name attribute");
            return names;
        }

        // Explicit paths that are missing are errors; paths built from variables are skipped silently - see EmuSen_BigPicture.md §12.3.
        private void Include(XElement include, string file)
        {
            string written = include.Value.Trim();
            bool fromVariable = written.Contains("${", StringComparison.Ordinal);
            string text = Substitute(written, out IReadOnlyList<string> undefined);
            if (undefined.Count > 0)
            {
                Debug(ThemeDiagnosticCode.IncludeSkipped, file, Line(include), $"include \"{written}\" skipped: {string.Join(", ", undefined.Select(u => "${" + u + "}"))} undefined");
                return;
            }
            if (text.Length == 0)
            {
                if (fromVariable) Debug(ThemeDiagnosticCode.IncludeSkipped, file, Line(include), $"include \"{written}\" skipped: empty");
                else Error(ThemeDiagnosticCode.NoValue, file, Line(include), "<include> has no path");
                return;
            }

            string resolved = ThemeValueParser.ResolvePath(text, file);
            if (!File.Exists(resolved))
            {
                if (fromVariable) Debug(ThemeDiagnosticCode.IncludeSkipped, file, Line(include), $"include \"{written}\" skipped: {resolved} not found");
                else Error(ThemeDiagnosticCode.IncludeMissing, file, Line(include), $"include \"{written}\" not found (resolved to {resolved})");
                return;
            }
            ProcessFile(resolved, Line(include));
        }

        private void View(XElement view, string file)
        {
            IReadOnlyList<string> names = Names(view, file);
            foreach (string name in names.Where(n => !ViewNames.Contains(n)))
                Error(ThemeDiagnosticCode.UnknownView, file, Line(view), $"view \"{name}\" is not system, gamelist or all");
            List<ViewBuilder> targets = names.Where(ViewNames.Contains).Distinct().Select(n => _views[n]).ToList();

            foreach (XElement element in view.Elements())
            {
                string type = element.Name.LocalName;
                ThemeElementSpec? spec = ThemeCatalog.Find(type);
                if (spec is null)
                {
                    bool block = Allowed[Block.Theme].Contains(type) || type == "transitions";
                    Error(block ? ThemeDiagnosticCode.MisplacedTag : ThemeDiagnosticCode.UnknownElement, file, Line(element),
                        block ? $"<{type}> is not allowed inside <view>" : $"<{type}> is not an element type");
                    continue;
                }
                if (element.Attribute("extra") is not null)
                    Error(ThemeDiagnosticCode.LegacySyntax, file, Line(element), $"<{type}> carries the legacy extra attribute");
                foreach (XAttribute attribute in element.Attributes().Where(a => a.Name.LocalName is not "name" and not "extra"))
                    Warn(ThemeDiagnosticCode.UnusedAttribute, file, Line(element), $"attribute {attribute.Name.LocalName} on <{type}> is ignored");

                IReadOnlyList<string> elementNames = Names(element, file);
                List<(string Name, RawProperty Value, string? Key)> properties = Properties(element, spec, file);
                foreach (ViewBuilder target in targets)
                    foreach (string elementName in elementNames)
                        target.Merge(spec, elementName, ref _order, properties);
            }
        }

        private List<(string, RawProperty, string?)> Properties(XElement element, ThemeElementSpec spec, string file)
        {
            var properties = new List<(string, RawProperty, string?)>();
            foreach (XElement property in element.Elements())
            {
                string name = property.Name.LocalName;
                ThemePropertySpec? propertySpec = spec.Find(name);
                if (propertySpec is null)
                {
                    Error(ThemeDiagnosticCode.UnknownProperty, file, Line(property), $"<{name}> is not a property of <{spec.Type}>");
                    continue;
                }
                if (property.HasElements)
                {
                    Error(ThemeDiagnosticCode.BadFormat, file, Line(property), $"<{name}> holds elements, not a value");
                    continue;
                }

                string? key = null;
                if (propertySpec.Shape == ThemeValueShape.Keyed)
                {
                    key = property.Attribute(propertySpec.KeyAttribute!)?.Value.Trim();
                    if (string.IsNullOrEmpty(key))
                    {
                        Error(ThemeDiagnosticCode.MissingName, file, Line(property), $"<{name}> needs a {propertySpec.KeyAttribute} attribute");
                        continue;
                    }
                    if (!propertySpec.Values.Contains(key))
                    {
                        Warn(ThemeDiagnosticCode.InvalidValue, file, Line(property), $"<{name} {propertySpec.KeyAttribute}=\"{key}\"> names no known {propertySpec.KeyAttribute}");
                        continue;
                    }
                }

                string written = property.Value;
                if (written.Trim().Length == 0)
                {
                    Error(ThemeDiagnosticCode.NoValue, file, Line(property), $"property \"{name}\" for element \"{spec.Type}\" has no value defined");
                    continue;
                }
                string text = Substitute(written, out IReadOnlyList<string> undefined);
                if (undefined.Count > 0)
                {
                    Error(ThemeDiagnosticCode.UndefinedVariable, file, Line(property), $"<{name}> uses undefined {string.Join(", ", undefined.Select(u => "${" + u + "}"))}");
                    continue;
                }
                bool fromVariable = written.Contains("${", StringComparison.Ordinal);
                if (text.Trim().Length == 0)
                {
                    Warn(ThemeDiagnosticCode.NoValue, file, Line(property), $"<{name}> is empty once its variables are substituted, and is ignored");
                    continue;
                }
                properties.Add((name, new RawProperty(text, file, Line(property), fromVariable), key));
            }
            return properties;
        }

        // One pass: each reference becomes the variable's value, which was itself substituted when it was defined.
        private string Substitute(string text, out IReadOnlyList<string> undefined)
        {
            var missing = new List<string>();
            string result = Reference.Replace(text, m =>
            {
                if (_variables.TryGetValue(m.Groups[1].Value, out string? value)) return value;
                missing.Add(m.Groups[1].Value);
                return m.Value;
            });
            undefined = missing;
            return result;
        }

        public ResolvedView Finish(string viewName, ThemeDiagnostics diagnostics, HashSet<ThemeDiagnostic> seen)
        {
            var local = new ThemeDiagnostics();
            ResolvedView view = _views[viewName].Resolve(this, local);
            foreach (ThemeDiagnostic d in local.Items)
                if (seen.Add(d)) diagnostics.Add(d.Severity, d.Code, d.File, d.Line, d.Message);
            return view;
        }

        internal ThemeSystem System => _system;
        internal ThemeSelection Selection => _selection;

        private void Error(ThemeDiagnosticCode code, string file, int line, string message) => Diagnostics.Add(ThemeSeverity.Error, code, file, line, message);
        private void Warn(ThemeDiagnosticCode code, string file, int line, string message) => Diagnostics.Add(ThemeSeverity.Warning, code, file, line, message);
        private void Debug(ThemeDiagnosticCode code, string file, int line, string message) => Diagnostics.Add(ThemeSeverity.Debug, code, file, line, message);

        internal static int Line(XObject node) => ThemeCapabilitiesReader.Line(node);
    }

    internal sealed record RawProperty(string Text, string File, int Line, bool FromVariable);
}
