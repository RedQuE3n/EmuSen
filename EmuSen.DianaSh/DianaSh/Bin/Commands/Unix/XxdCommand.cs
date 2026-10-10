using System.IO;
using System.Text;
using EmuSen.DianaSh.DianaSh.Bin;
using EmuSen.DianaSh.DianaSh.Etc;
using EmuSen.DianaSh.DianaSh.Lib;
using EmuSen.DianaSh.DianaSh.Var;
using EmuSen.DianaSh.DianaSh.Dev;

namespace EmuSen.DianaSh.DianaSh.Bin.Commands.Unix
{
    // Raw bytes, in `mem`'s dump layout so every hexdump here looks alike - see `man xxd`.
    public class XxdCommand : IDianaShCommand
    {
        public string Name => "xxd";
        public bool IsReadOnly => true;
        public string Usage => "  xxd | hexdump <path>          hexdump a real file's raw bytes, or piped stdin (UTF8-encoded first)";

        public DianaShResult Execute(IDebugTarget? target, string[] args, string? stdin)
        {
            byte[] data;

            if (stdin != null)
            {
                data = Encoding.UTF8.GetBytes(stdin);
            }
            else
            {
                if (args.Length < 2) return DianaShResult.Fail("xxd: usage: xxd <path>");

                if (!DianaShSandbox.TryResolve(args[1], out string resolved))
                {
                    return DianaShResult.Fail($"xxd: '{args[1]}' is outside the project sandbox ({DianaShSandbox.RootDirectory})");
                }
                if (!File.Exists(resolved))
                {
                    return DianaShResult.Fail($"xxd: no such file: {args[1]}");
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
