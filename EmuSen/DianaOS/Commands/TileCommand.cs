using System.Text;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // ASCII-decodes one 8x8 tile - generalizes the old DebugTools.
    // DecodeTileAscii (which only ever worked against a raw VRAM byte[])
    // into something that works against any IDebugMemorySpace, so it's not
    // tied to VRAM, or to one hardcoded address the way CoinTileDumpLogging's
    // Program.cs call site was.
    //
    // Fully core-agnostic: the actual bitplane/pixel decoding lives in
    // IDebugTarget.DecodeTilePixels - see that method's own comment for why
    // (and SnesDebugTarget's implementation, for the SNES-specific planar
    // bitplane layout that used to be hardcoded directly in this file).
    // This command only walks the resulting pixel grid and renders it as
    // ASCII; it has no idea what "bpp" even means for a given core's tile
    // format.
    public class TileCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "tile";
        public string Usage => "  tile <space> <addr> <bpp>     ASCII-decode one 8x8 tile (bpp meaning is core-specific - 2/4/8 on SNES)";

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
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
