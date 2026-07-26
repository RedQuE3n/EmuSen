using System.Text;
using static EmuSen.Debug.Commands.DebugCommandHelpers;

namespace EmuSen.Debug.Commands
{
    // Generalizes the old DebugTools.DecodeTileAscii (which only ever
    // worked against a raw VRAM byte[]) into something that works
    // against any IDebugMemorySpace - so it's not tied to VRAM, or to
    // SNES, or to one hardcoded address the way CoinTileDumpLogging's
    // Program.cs call site was. Also extends bpp support to 8 (not
    // needed when DecodeTileAscii was written, but real now that Mode
    // 3/4's 8bpp BG1 and Direct Color are implemented) - same
    // bitplane-pair-every-16-bytes layout as SampleBgPixel in
    // Renderer.Backgrounds.cs.
    public class TileCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "tile";
        public string Usage => "  tile <space> <addr> <bpp>     ASCII-decode one 8x8 tile (bpp: 2, 4, or 8)";

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            if (parts.Length < 4) return "Usage: tile <space> <addr> <bpp>";
            IDebugMemorySpace space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            int bpp = ParseHex(parts[3]);
            if (bpp != 2 && bpp != 4 && bpp != 8) return "bpp must be 2, 4, or 8";

            var sb = new StringBuilder();
            sb.AppendLine($"{space.Name} @ 0x{addr:X}, {bpp}bpp tile:");
            for (int row = 0; row < 8; row++)
            {
                byte p0 = space.Read(addr + row * 2);
                byte p1 = space.Read(addr + row * 2 + 1);
                byte p2 = 0, p3 = 0, p4 = 0, p5 = 0, p6 = 0, p7 = 0;
                if (bpp >= 4)
                {
                    p2 = space.Read(addr + 16 + row * 2);
                    p3 = space.Read(addr + 16 + row * 2 + 1);
                }
                if (bpp == 8)
                {
                    p4 = space.Read(addr + 32 + row * 2);
                    p5 = space.Read(addr + 32 + row * 2 + 1);
                    p6 = space.Read(addr + 48 + row * 2);
                    p7 = space.Read(addr + 48 + row * 2 + 1);
                }

                sb.Append("  ");
                for (int col = 0; col < 8; col++)
                {
                    int bit = 7 - col;
                    int val = ((p0 >> bit) & 1) | (((p1 >> bit) & 1) << 1);
                    if (bpp >= 4) val |= (((p2 >> bit) & 1) << 2) | (((p3 >> bit) & 1) << 3);
                    if (bpp == 8) val |= (((p4 >> bit) & 1) << 4) | (((p5 >> bit) & 1) << 5) | (((p6 >> bit) & 1) << 6) | (((p7 >> bit) & 1) << 7);
                    sb.Append(val == 0 ? (bpp == 8 ? ". " : ".") : val.ToString(bpp == 8 ? "X2" : "X1"));
                    sb.Append(' ');
                }
                sb.AppendLine();
            }
            return sb.ToString().TrimEnd();
        }
    }
}
