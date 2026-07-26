using System.Collections.Generic;
using System.Linq;
using System.Text;
using static EmuSen.Shell.Commands.DebugCommandHelpers;

namespace EmuSen.Shell.Commands
{
    public class PalCommand : EmuSen.Shell.IShellCommand
    {
        public string Name => "pal";
        public string Usage => "  pal [<index>]                 one palette, or all of them if omitted";

        public EmuSen.Shell.ShellResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.Shell.Commands.DebugCommandHelpers.RequireTarget(target);
            var palettes = target.GetPalettes();
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
