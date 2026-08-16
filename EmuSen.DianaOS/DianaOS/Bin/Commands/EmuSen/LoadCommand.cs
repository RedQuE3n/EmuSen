using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // The write-side counterpart to `dump` - see §3.11.
    public class LoadCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "load";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  load <space> <addr> <file>    write home/Logs/<CoreName>/<file>'s raw bytes into <space> starting at <addr>",
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

            string path = Path.Combine(DianaOSSandbox.LogsDirectory, target.CoreName, file);
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
