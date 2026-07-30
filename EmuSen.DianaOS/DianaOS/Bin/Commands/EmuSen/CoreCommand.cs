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
    // Which console a ROM belongs to, and what file extensions it's
    // expected to have - a display name plus an extension allowlist, both
    // genuinely core-agnostic (same reasoning IDebugTarget itself stays
    // agnostic) despite living alongside DianaOS's other, mostly SNES-
    // touching commands. Trying to load, say, a `.nes` file against
    // `venus` is a clear user error worth catching here rather than
    // handing bytes that aren't really an SNES ROM to `Cartridge`, which
    // has no format validation of its own at all.
    public sealed record CoreDescriptor(string DisplayName, string[] Extensions)
    {
        public bool SupportsExtension(string extension) =>
            Array.Exists(Extensions, e => e.Equals(extension, StringComparison.OrdinalIgnoreCase));
    }

    // `core <corename> <path>` - swaps (or, on a frontend that has no
    // other way to load one at all, loads) the running ROM. Validates
    // exactly like EmuSen.Hotaru/Program.cs's own TryResolveCoreCommand,
    // which backs RunStandaloneShell's pre-window `core` handling and
    // deliberately stays a separate, untouched code path (see that
    // method's own comment) - RunStandaloneShell runs before any
    // VenusCore/window exists at all, so it has nothing to swap yet and
    // needs to hand a validated path back up to Main instead of emitting
    // a HostAction. This class is for the case that method can't cover:
    // reloading an ALREADY-RUNNING session, which needs to reach back out
    // through DianaOSResult's own HostAction.LoadCore escape hatch (see
    // that type's own comment) since a command can't otherwise change
    // its caller's control flow.
    //
    // The registry is constructor-injected (Mechanism A, same shape as
    // CoretopCommand's own openWindow delegate) rather than owned here,
    // since which concrete ICore to construct for a given ROM is exactly
    // the kind of frontend-owned decision EmuSen.DianaOS itself stays
    // agnostic about on purpose - this class only ever sees whatever
    // registry its caller hands it.
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
