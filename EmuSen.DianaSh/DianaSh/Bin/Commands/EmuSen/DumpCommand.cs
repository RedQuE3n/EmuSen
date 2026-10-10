using System.IO;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;
using static EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen
{
    // Raw bytes with no header, so a hex editor opens the output directly - see §3.11.
    public class DumpCommand : global::EmuSen.DianaSh.DianaSh.Lib.IDianaShCommand
    {
        public string Name => "dump";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  dump <space> <addr> <len> <file> write raw bytes to home/Logs/<CoreName>/<file>",
        });

        public global::EmuSen.DianaSh.DianaSh.Lib.DianaShResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaSh.DianaSh.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 5) return "Usage: dump <space> <addr> <len> <file>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            int len = ParseHex(parts[3]);
            string file = parts[4];

            // The same HasSideEffects refusal as search and snapshot - see §3.1a.
            if (space.HasSideEffects)
            {
                return $"{space.Name} can have real side effects on read (live hardware registers) - refusing a bulk dump there. Try WRAM (or another plain-memory space) instead.";
            }

            var data = new byte[len];
            for (int i = 0; i < len; i++) data[i] = space.Read(addr + i);

            string coreLogDir = Path.Combine(DianaShSandbox.LogsDirectory, target.CoreName);
            if (!DianaShSandbox.TryResolveFrom(coreLogDir, file, out string path))
                return DianaShResult.Fail($"dump: '{file}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
            Directory.CreateDirectory(Path.GetDirectoryName(path)!);
            File.WriteAllBytes(path, data);
            return $"Dumped {space.Name} 0x{addr:X}-0x{addr + len - 1:X} ({len} bytes) to {path}";
        }
    }
}
