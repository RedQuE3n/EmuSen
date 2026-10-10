using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // The write-side counterpart to `dump` - see §3.11.
    public class LoadCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "load";
        public bool IsReadOnly => false;
        public string Usage => string.Join('\n', new[]
        {
            "  load <space> <addr> <file>    write home/Logs/<CoreName>/<file>'s raw bytes into <space> starting at <addr>",
        });

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 4) return "Usage: load <space> <addr> <file>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            string file = parts[3];

            if (!space.IsWritable)
            {
                return $"{space.Name} is read-only - refusing to load into it.";
            }

            if (!DianaShSandbox.TryResolveFrom(Path.Combine(DianaShSandbox.LogsDirectory, target.CoreName), file, out string path))
                return DianaShResult.Fail($"load: '{file}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
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
