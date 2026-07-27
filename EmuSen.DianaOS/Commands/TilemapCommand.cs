using System.Linq;
using System.Text;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    // Decodes a grid of raw tilemap/nametable entries as text instead of
    // pixels - complementary to `tile` (decodes ONE tile's actual pixel
    // content). Built specifically so a menu cursor's position, or
    // whether a HUD element changed, can be confirmed by comparing
    // tilemap entries numerically, without ever rendering or eyeballing
    // a screenshot.
    //
    // Core-agnostic on purpose, same split as `disasm`/`regs`: this
    // command only walks a grid and formats whatever string each entry
    // decodes to - it has no idea what a SNES BG screen word's bits mean.
    // All of that lives in IDebugTarget.TilemapEntryStride/
    // DecodeTilemapEntry (see that interface's own comment for why an
    // NES core's nametable+attribute-table split couldn't share a single
    // generic bit-layout here even if this command tried to parse one
    // itself).
    //
    // Same convention as `tile`: the caller supplies the exact base
    // address (from BG1SC/etc, via `regs`) rather than this command
    // resolving BG-layer-to-tilemap-address mapping itself, which would
    // need to duplicate exactly what Renderer.Backgrounds.cs already does
    // for real rendering (per-mode/per-layer 32x32/64x32/32x64/64x64
    // mirroring) - not worth it for a debug dump.
    public class TilemapCommand : EmuSen.DianaOS.IDianaOSCommand
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

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
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
