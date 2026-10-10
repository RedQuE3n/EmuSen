using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
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

    // For reloading an already-running session, which needs HostAction.LoadCore - see §3.3b.
    public class CoreCommand : IDianaShCommand
    {
        private readonly IReadOnlyDictionary<string, CoreDescriptor> _registry;

        public CoreCommand(IReadOnlyDictionary<string, CoreDescriptor> registry)
        {
            _registry = registry;
        }

        public string Name => "core";
        public bool IsReadOnly => false;
        public string Usage => "  core <corename> <path>        load or swap the running ROM (validated against the known core registry)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length != 3)
            {
                return DianaShResult.Fail("Usage: core <corename> <path-to-rom>");
            }

            string coreName = args[1];
            string romPath = args[2];
            if (Refusal(_registry, coreName, romPath, out string resolved) is { } refused) return refused;
            CoreDescriptor descriptor = _registry[coreName];
            return DianaShResult.Ok($"[CORE] Loading {descriptor.DisplayName}: {romPath}", new HostAction.LoadCore(coreName, resolved));
        }

        // What `core` refuses, shared with Hotaru's launch prompt; null when the ROM may load from resolved - see `man core`.
        public static DianaShResult? Refusal(IReadOnlyDictionary<string, CoreDescriptor> registry, string coreName, string romPath, out string resolved)
        {
            resolved = romPath;
            if (!registry.TryGetValue(coreName, out CoreDescriptor? descriptor))
            {
                return DianaShResult.Fail($"core: unknown core '{coreName}'. Supported: {string.Join(", ", new SortedSet<string>(registry.Keys, StringComparer.OrdinalIgnoreCase))}");
            }

            if (!DianaShSandbox.TryResolveRom(romPath, out resolved))
            {
                string library = DianaShSandbox.RomLibraryDirectory is { } lib ? $" or the ROM library ({lib})" : "";
                return DianaShResult.Fail($"core: '{romPath}' is outside the project sandbox ({DianaShSandbox.RootDirectory}){library}");
            }

            if (!File.Exists(resolved))
            {
                return DianaShResult.Fail($"core: ROM not found: {romPath}");
            }

            string extension = Path.GetExtension(resolved);
            if (!descriptor.SupportsExtension(extension))
            {
                return DianaShResult.Fail($"core: '{(extension.Length > 0 ? extension : "(no extension)")}' is not a supported ROM type for {descriptor.DisplayName} - expected: {string.Join(", ", descriptor.Extensions)}");
            }

            return null;
        }
    }
}
