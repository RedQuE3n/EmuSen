using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // A display name plus an extension allowlist, both core-agnostic - see §3.
    public sealed record CoreDescriptor(string DisplayName, string[] Extensions, string[]? CheatSystems = null,
        string? ConsoleName = null, string Manufacturer = "", int ReleaseYear = 0)
    {
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
    public class CoreCommand : IDianaOSCommand
    {
        private readonly IReadOnlyDictionary<string, CoreDescriptor> _registry;

        public CoreCommand(IReadOnlyDictionary<string, CoreDescriptor> registry)
        {
            _registry = registry;
        }

        public string Name => "core";
        public bool IsReadOnly => false;
        public string Usage => "  core <corename> <path>        load or swap the running ROM (validated against the known core registry)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length != 3)
            {
                return DianaOSResult.Fail("Usage: core <corename> <path-to-rom>");
            }

            string coreName = args[1];
            string romPath = args[2];

            if (!_registry.TryGetValue(coreName, out CoreDescriptor? descriptor))
            {
                return DianaOSResult.Fail($"core: unknown core '{coreName}'. Supported: {string.Join(", ", new SortedSet<string>(_registry.Keys, StringComparer.OrdinalIgnoreCase))}");
            }

            if (!File.Exists(romPath))
            {
                return DianaOSResult.Fail($"core: ROM not found: {romPath}");
            }

            string extension = Path.GetExtension(romPath);
            if (!descriptor.SupportsExtension(extension))
            {
                return DianaOSResult.Fail($"core: '{(extension.Length > 0 ? extension : "(no extension)")}' is not a supported ROM type for {descriptor.DisplayName} - expected: {string.Join(", ", descriptor.Extensions)}");
            }

            return DianaOSResult.Ok($"[CORE] Loading {descriptor.DisplayName}: {romPath}", new HostAction.LoadCore(coreName, romPath));
        }
    }
}
