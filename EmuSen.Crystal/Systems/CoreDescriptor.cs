using System;
using System.Collections.Generic;
using System.Linq;

namespace EmuSen.Crystal.Systems
{
    // A display name plus an extension allowlist, both core-agnostic - see §3.
    public sealed record CoreDescriptor(string DisplayName, string[] Extensions, string[]? CheatSystems = null,
        string? ConsoleName = null, string Manufacturer = "", int ReleaseYear = 0, double CoverAspect = CoreDescriptor.DefaultCoverAspect,
        string[]? OpenVgdbSystems = null, Func<byte[], byte[]>? OpenVgdbBytes = null)
    {
        // OpenVGDB's systemShortName for each system this core plays, and the bytes of a file it hashed - see EmuSen_Settings_Reference.md §4.39.
        public IReadOnlyList<string> OpenVgdbSystemNames => OpenVgdbSystems ?? Array.Empty<string>();

        // Height over width of the console's North American box, OpenEmu's fallback when unknown - see EmuSen_Settings_Reference.md §4.33.
        public const double DefaultCoverAspect = 1.365385;

        // The bare console, with no codename: a tab header, and the key a per-console config file is stored under.
        public string Console => ConsoleName ?? DisplayName;

        public bool SupportsExtension(string extension) =>
            Array.Exists(Extensions, e => e.Equals(extension, StringComparison.OrdinalIgnoreCase));

        public IReadOnlyList<string> CheatSystemNames => CheatSystems ?? Array.Empty<string>();

        // A core's row made from the systems it runs, never written by hand: the first system's console and facts, every system's extensions and folders - see EmuSen_Backend.md §12.
        public static CoreDescriptor Of(string displayName, params SystemEntry[] systems) =>
            new(displayName, systems.SelectMany(s => s.Extensions).Distinct(StringComparer.OrdinalIgnoreCase).ToArray(), systems.SelectMany(s => s.CheatSystems).Distinct().ToArray(),
                systems[0].Console, systems[0].Manufacturer, systems[0].ReleaseYear, CoverAspect: systems[0].CoverAspect,
                OpenVgdbSystems: systems.SelectMany(s => s.OpenVgdbSystems).Distinct().ToArray(), OpenVgdbBytes: systems[0].OpenVgdbBytes);

        // One core under several aliases must not count its folders twice.
        public static IReadOnlyCollection<string> SupportedCheatSystems(IEnumerable<CoreDescriptor> registry)
        {
            var systems = new HashSet<string>(StringComparer.OrdinalIgnoreCase);
            foreach (CoreDescriptor descriptor in registry)
            {
                foreach (string system in descriptor.CheatSystemNames) systems.Add(system);
            }
            return systems;
        }
    }
}
