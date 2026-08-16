using System.IO;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Raw bytes with no header, so a hex editor opens the output directly - see §3.11.
    public class DumpCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "dump";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  dump <space> <addr> <len> <file> write raw bytes to home/Logs/<CoreName>/<file>",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
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

            string coreLogDir = Path.Combine(DianaOSSandbox.LogsDirectory, target.CoreName);
            Directory.CreateDirectory(coreLogDir);
            string path = Path.Combine(coreLogDir, file);
            File.WriteAllBytes(path, data);
            return $"Dumped {space.Name} 0x{addr:X}-0x{addr + len - 1:X} ({len} bytes) to {path}";
        }
    }
}
