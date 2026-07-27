using System.Collections.Generic;
using System.Linq;
using System.Text;
using static EmuSen.DianaOS.Commands.DebugCommandHelpers;

namespace EmuSen.DianaOS.Commands
{
    public class PalCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "pal";
        public bool IsReadOnly => true;
        public string Usage => "  pal [<index>]                 one palette, or all of them if omitted";

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
            var palettes = target.Palettes.Current;
            var sb = new StringBuilder();

            IEnumerable<DebugPaletteInfo> toShow = palettes;
            if (parts.Length >= 2)
            {
                int idx = ParseHex(parts[1]);
                toShow = palettes.Where(p => p.Index == idx);
            }

            foreach (var p in toShow)
            {
                sb.Append($"pal {p.Index,2}: ");
                sb.AppendLine(string.Join(' ', p.Colors.Select(c => $"{c.r:X2}{c.g:X2}{c.b:X2}")));
            }
            return sb.ToString().TrimEnd();
        }
    }
}
