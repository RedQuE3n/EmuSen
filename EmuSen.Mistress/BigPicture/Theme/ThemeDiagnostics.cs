using System.Collections.Generic;

namespace EmuSen.Mistress.BigPicture.Theme
{
    // THEMES.md's three outcomes: an error unthemes the system, a warning resets a value, a debug note is silent - see EmuSen_BigPicture.md §12.3.
    public enum ThemeSeverity { Debug, Warning, Error }

    public enum ThemeDiagnosticCode
    {
        CapabilitiesMissing,
        NoThemeFile,
        MalformedXml,
        WrongRoot,
        MisplacedTag,
        UnknownTag,
        UnknownElement,
        UnknownProperty,
        UnknownView,
        MissingName,
        LegacySyntax,
        NoValue,
        BadFormat,
        InvalidValue,
        InvalidImageType,
        UndefinedVariable,
        IncludeMissing,
        IncludeSkipped,
        IncludeLoop,
        PathMissing,
        Undeclared,
        Duplicate,
        Reserved,
        ViewRestricted,
        SecondPrimary,
        LanguageWithoutEnglish,
        UnusedAttribute,
    }

    public sealed record ThemeDiagnostic(ThemeSeverity Severity, ThemeDiagnosticCode Code, string File, int Line, string Message)
    {
        public override string ToString() => $"{Severity} {Code} {System.IO.Path.GetFileName(File)}:{Line}: {Message}";
    }

    // Collects diagnostics in the order they happen, and says whether any unthemes the system.
    public sealed class ThemeDiagnostics
    {
        private readonly List<ThemeDiagnostic> _items = [];

        public IReadOnlyList<ThemeDiagnostic> Items => _items;
        public bool HasErrors { get; private set; }

        public void Add(ThemeSeverity severity, ThemeDiagnosticCode code, string file, int line, string message)
        {
            _items.Add(new ThemeDiagnostic(severity, code, file, line, message));
            if (severity == ThemeSeverity.Error) HasErrors = true;
        }

        public void AddRange(IEnumerable<ThemeDiagnostic> diagnostics)
        {
            foreach (ThemeDiagnostic d in diagnostics) Add(d.Severity, d.Code, d.File, d.Line, d.Message);
        }
    }
}
