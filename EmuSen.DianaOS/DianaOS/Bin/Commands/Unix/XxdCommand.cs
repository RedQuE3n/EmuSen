using System.IO;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.Unix
{
    // Raw bytes, in `mem`'s dump layout so every hexdump here looks alike - see `man xxd`.
    public class XxdCommand : IDianaOSCommand
    {
        public string Name => "xxd";
        public bool IsReadOnly => true;
        public string Usage => "  xxd | hexdump <path>          hexdump a real file's raw bytes, or piped stdin (UTF8-encoded first)";

        public DianaOSResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            byte[] data;

            if (stdin != null)
            {
                data = Encoding.UTF8.GetBytes(stdin);
            }
            else
            {
                if (args.Length < 2) return DianaOSResult.Fail("xxd: usage: xxd <path>");

                if (!DianaOSSandbox.TryResolve(args[1], out string resolved))
                {
                    return DianaOSResult.Fail($"xxd: '{args[1]}' is outside the project sandbox ({DianaOSSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaOSResult.Fail($"xxd: no such file: {args[1]}");
                }
                data = File.ReadAllBytes(resolved);
            }

            if (data.Length == 0) return "";

            var sb = new StringBuilder();
            for (int row = 0; row < data.Length; row += 16)
            {
                sb.Append($"  {row:X6}: ");
                var ascii = new StringBuilder();
                for (int col = 0; col < 16; col++)
                {
                    if (row + col < data.Length)
                    {
                        byte b = data[row + col];
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
