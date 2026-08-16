using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Walks a pixel grid; the bitplane decoding is the core's - see §3.1a.
    public class TileCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "tile";
        public bool IsReadOnly => true;
        public string Usage => "  tile <space> <addr> <bpp>     ASCII-decode one 8x8 tile (bpp meaning is core-specific - 2/4/8 on SNES)";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 4) return "Usage: tile <space> <addr> <bpp>";
            IDebugTarget t = RequireTarget(target);
            IDebugMemorySpace space = FindSpace(t, parts[1]);
            int addr = ParseHex(parts[2]);
            int bpp = ParseHex(parts[3]);

            byte[] pixels = t.DecodeTilePixels(space, addr, bpp);

            var sb = new StringBuilder();
            sb.AppendLine($"{space.Name} @ 0x{addr:X}, {bpp}bpp tile:");
            for (int row = 0; row < 8; row++)
            {
                sb.Append("  ");
                for (int col = 0; col < 8; col++)
                {
                    int val = pixels[row * 8 + col];
                    sb.Append(val == 0 ? (bpp == 8 ? ". " : ".") : val.ToString(bpp == 8 ? "X2" : "X1"));
                    sb.Append(' ');
                }
                sb.AppendLine();
            }
            return sb.ToString().TrimEnd();
        }
    }
}
