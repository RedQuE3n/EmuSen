using System;
using System.IO;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.Crystal.Debugger;

namespace EmuSen.Hotaru
{
    // Where `hotaru` puts the game's picture; the window is the only one so far - see `man hotaru`.
    public enum HotaruOutput
    {
        Window,
    }

    // `hotaru <rom>`: Hotaru's game window, opened by a host that can open one - see `man hotaru`.
    public sealed class HotaruCommand : IDianaShCommand
    {
        public const string NoGameWindow = "this shell's host does not open Hotaru's game window";

        private readonly Action<string, HotaruOutput>? _open;
        private readonly string _refusal;

        public HotaruCommand(Action<string, HotaruOutput>? open, string? refusal = null)
        {
            _open = open;
            _refusal = refusal ?? NoGameWindow;
        }

        public string Name => "hotaru";
        public bool IsReadOnly => false;
        public string Usage => "  hotaru [--output window] <rom>  play <rom> in Hotaru's game window, with its hotkeys, feed and\n" +
                                "                                debug windows; the shell keeps its prompt";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            HotaruOutput output = HotaruOutput.Window;
            string? rom = null;
            for (int i = 1; i < args.Length; i++)
            {
                if (args[i].Equals("--output", StringComparison.OrdinalIgnoreCase))
                {
                    if (++i >= args.Length) return DianaShResult.Fail("Usage: hotaru [--output window] <rom>");
                    string? named = Array.Find(Enum.GetNames<HotaruOutput>(), n => n.Equals(args[i], StringComparison.OrdinalIgnoreCase));
                    if (named is null) return DianaShResult.Fail($"hotaru: no output named '{args[i]}' - the only one is 'window'.");
                    output = Enum.Parse<HotaruOutput>(named);
                    continue;
                }
                if (rom != null) return DianaShResult.Fail("Usage: hotaru [--output window] <rom>");
                rom = args[i];
            }

            if (rom is null) return DianaShResult.Fail("Usage: hotaru [--output window] <rom>");
            if (_open is null) return DianaShResult.Fail($"hotaru: {_refusal} - see `man hotaru`.");
            if (Refusal(rom, sandboxed: true, out string resolved) is { } refused) return refused;

            // A ROM that will not load lands back at the prompt, not on the shell's host.
            try { _open(resolved, output); }
            catch (Exception ex) { return DianaShResult.Fail($"[CPU HALT] {ex.Message}"); }
            return DianaShResult.Ok($"[ROM] Loading: {rom}");
        }

        // A typed ROM is the sandbox's; one from the program's command line is the launcher's - see `man hotaru`.
        public static DianaShResult? Refusal(string rom, bool sandboxed, out string resolved)
        {
            resolved = rom;
            if (sandboxed && !DianaShSandbox.TryResolveRom(rom, out resolved))
            {
                string library = DianaShSandbox.RomLibraryDirectory is { } lib ? $" or the ROM library ({lib})" : "";
                return DianaShResult.Fail($"hotaru: '{rom}' is outside the project sandbox ({DianaShSandbox.RootDirectory}){library}");
            }
            if (!File.Exists(resolved)) return DianaShResult.Fail($"[ERROR] ROM not found: {rom}");
            return null;
        }
    }
}
