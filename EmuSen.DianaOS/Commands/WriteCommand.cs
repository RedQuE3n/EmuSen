using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    public class WriteCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "write";
        public bool IsReadOnly => false;
        public string Usage => "  write <space> <addr> <value>  write one byte (only if <space> is writable)";

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
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
