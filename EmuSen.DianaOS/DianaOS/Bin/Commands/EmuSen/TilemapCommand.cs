using System.Linq;
using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;
using static EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    // Formats entries the core decoded; the caller supplies the base address - see §3.1a.
    public class TilemapCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "tilemap";
        public bool IsReadOnly => true;
        public string Usage => string.Join('\n', new[]
        {
            "  tilemap <space> <addr> <cols> <rows>",
            "                                decode <cols>x<rows> tilemap entries starting at <addr>",
            "                                as a text grid, one already-formatted label per cell -",
            "                                see IDebugTarget.DecodeTilemapEntry for what a label",
            "                                means on the current core. Caller supplies the exact base",
            "                                address (see BG1SC/etc via `regs`) - this doesn't resolve",
            "                                BG-layer-to-tilemap-address mapping itself.",
        });

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            if (parts.Length < 5) return Usage;
            var space = FindSpace(target, parts[1]);
            int addr = ParseHex(parts[2]);
            int cols = ParseHex(parts[3]);
            int rows = ParseHex(parts[4]);
            int stride = target.TilemapEntryStride;

            var sb = new StringBuilder();
            for (int row = 0; row < rows; row++)
            {
                var cells = Enumerable.Range(0, cols)
                    .Select(col => target.DecodeTilemapEntry(space, addr + (row * cols + col) * stride));
                sb.AppendLine(string.Join(' ', cells));
            }
            return sb.ToString().TrimEnd('\n');
        }
    }
}
