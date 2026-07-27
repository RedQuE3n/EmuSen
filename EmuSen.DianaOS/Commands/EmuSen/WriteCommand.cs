using static EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    public class WriteCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "write";
        public bool IsReadOnly => false;
        public string Usage => "  write <space> <addr> <value>  write one byte (only if <space> is writable)";

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
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
