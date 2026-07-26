using System.IO;
using static EmuSen.Shell.Commands.DebugCommandHelpers;

namespace EmuSen.Shell.Commands
{
    // The write-side counterpart to DumpCommand - takes a raw byte file
    // (typically one `dump` just produced, hand-edited in a hex editor,
    // or captured some other way) and pokes it back into a memory space
    // starting at a given address. Useful for reproducing a state that's
    // easier to describe as "this file's bytes at this address" than as
    // a sequence of individual `write` commands, or for restoring a
    // `dump`ped range after poking around with `write`.
    public class LoadCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "load";
        public string Usage => string.Join('\n', new[]
        {
            "  load <space> <addr> <file>    write Logs/<CoreName>/<file>'s raw bytes into <space> starting at <addr>",
        });

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Shell.Commands.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 4) return "Usage: load <space> <addr> <file>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            string file = parts[3];

            if (!space.IsWritable)
            {
                return $"{space.Name} is read-only - refusing to load into it.";
            }

            string path = Path.Combine("Logs", target.CoreName, file);
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
