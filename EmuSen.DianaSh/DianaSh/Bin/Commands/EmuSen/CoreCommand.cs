using System;
using System.Collections.Generic;
using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;
using EmuSen.Crystal.Systems;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // For reloading an already-running session, which needs HostAction.LoadCore - see §3.3b.
    public class CoreCommand : IDianaShCommand
    {
        private readonly IReadOnlyDictionary<string, CoreDescriptor> _registry;
        private readonly Func<string, bool> _startsEmpty;

        // startsEmpty says whether a console can be started with nothing in it; the host knows, from its engines.
        public CoreCommand(IReadOnlyDictionary<string, CoreDescriptor> registry, Func<string, bool>? startsEmpty = null)
        {
            _registry = registry;
            _startsEmpty = startsEmpty ?? (_ => false);
        }

        public string Name => "core";
        public bool IsReadOnly => false;
        public string Usage => "  core <corename> <path>        load or swap the running ROM (validated against the known core registry)\n" +
                               "  core <corename> --console     start that core's console with nothing in it, where it can";

        public const string ConsoleFlag = "--console";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            if (args.Length != 3)
            {
                return DianaShResult.Fail("Usage: core <corename> <path-to-rom> | core <corename> --console");
            }

            string coreName = args[1];
            string romPath = args[2];
            if (romPath == ConsoleFlag)
            {
                if (ConsoleRefusal(_registry, _startsEmpty, coreName, out string console) is { } no) return no;
                return DianaShResult.Ok($"[CORE] Starting the {console} with nothing in it", new HostAction.StartConsole(coreName, console));
            }
            if (Refusal(_registry, coreName, romPath, out string resolved) is { } refused) return refused;
            CoreDescriptor descriptor = _registry[coreName];
            return DianaShResult.Ok($"[CORE] Loading {descriptor.DisplayName}: {romPath}", new HostAction.LoadCore(coreName, resolved));
        }

        // What `core <name> --console` refuses, shared with the launch prompts; null when console may start empty - see `man core`.
        public static DianaShResult? ConsoleRefusal(IReadOnlyDictionary<string, CoreDescriptor> registry, Func<string, bool> startsEmpty, string coreName, out string console)
        {
            console = "";
            if (!registry.TryGetValue(coreName, out CoreDescriptor? descriptor))
                return DianaShResult.Fail($"core: unknown core '{coreName}'. Supported: {string.Join(", ", new SortedSet<string>(registry.Keys, StringComparer.OrdinalIgnoreCase))}");
            console = descriptor.Console;
            if (!startsEmpty(console)) return DianaShResult.Fail($"core: no core in this build starts the {console} with nothing in it");
            return null;
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
