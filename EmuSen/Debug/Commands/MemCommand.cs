using System.Text;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    public class MemCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "mem";
        public string Usage => "  mem <space> <addr> [<len>]    hexdump <len> bytes (default 16) from <space> at <addr>";

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 3) return "Usage: mem <space> <addr> [<len>]";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            int len = parts.Length >= 4 ? ParseHex(parts[3]) : 16;

            var sb = new StringBuilder();
            sb.AppendLine($"{space.Name} @ 0x{addr:X} ({len} bytes):");
            for (int row = 0; row < len; row += 16)
            {
                sb.Append($"  {addr + row:X6}: ");
                var ascii = new StringBuilder();
                for (int col = 0; col < 16; col++)
                {
                    if (row + col < len)
                    {
                        byte b = space.Read(addr + row + col);
                        sb.Append($"{b:X2} ");
                        ascii.Append(b >= 0x20 && b < 0x7F ? (char)b : '.');
                    }
                    else
                    {
                        sb.Append("   ");
                    }
                }
                sb.Append(' ').Append(ascii);
                sb.AppendLine();
            }
            return sb.ToString().TrimEnd();
        }
    }
}
