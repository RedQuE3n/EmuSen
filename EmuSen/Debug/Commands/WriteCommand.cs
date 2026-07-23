using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    public class WriteCommand : IDebugCommand
    {
        public string Name => "write";
        public string Usage => "  write <space> <addr> <value>  write one byte (only if <space> is writable)";

        public string Execute(IDebugTarget target, string[] parts)
        {
            if (parts.Length < 4) return "Usage: write <space> <addr> <value>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            if (!space.IsWritable) return $"{space.Name} is read-only.";
            int addr = ParseHex(parts[2]);
            byte value = (byte)ParseHex(parts[3]);
            space.Write(addr, value);
            return $"{space.Name}[0x{addr:X}] = 0x{value:X2}";
        }
    }
}
