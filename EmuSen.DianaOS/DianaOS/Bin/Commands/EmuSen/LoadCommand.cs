using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // The write-side counterpart to DumpCommand - takes a raw byte file
    // (typically one `dump` just produced, hand-edited in a hex editor,
    // or captured some other way) and pokes it back into a memory space
    // starting at a given address. Useful for reproducing a state that's
    // easier to describe as "this file's bytes at this address" than as
    // a sequence of individual `write` commands, or for restoring a
    // `dump`ped range after poking around with `write`.
    public class LoadCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "load";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  load <space> <addr> <file>    write var/log/<CoreName>/<file>'s raw bytes into <space> starting at <addr>",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 4) return "Usage: load <space> <addr> <file>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            string file = parts[3];

            if (!space.IsWritable)
            {
                return $"{space.Name} is read-only - refusing to load into it.";
            }

            string path = Path.Combine(DianaOSSandbox.RootDirectory, "var", "log", target.CoreName, file);
            if (!File.Exists(path))
            {
                return $"No file at {path}.";
            }

            byte[] data = File.ReadAllBytes(path);
            for (int i = 0; i < data.Length; i++) space.Write(addr + i, data[i]);
            return $"Loaded {data.Length} bytes from {path} into {space.Name} starting at 0x{addr:X}.";
        }
    }
}
