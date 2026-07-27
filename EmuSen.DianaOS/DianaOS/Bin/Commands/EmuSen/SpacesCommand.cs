using System.Text;
using EmuSen.DianaOS.DianaOS.Bin;
using EmuSen.DianaOS.DianaOS.Etc;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS.DianaOS.Var;
using EmuSen.DianaOS.DianaOS.Dev;

namespace EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen
{
    public class SpacesCommand : global::EmuSen.DianaOS.DianaOS.Lib.IDianaOSCommand
    {
        public string Name => "spaces";
        public bool IsReadOnly => true;
        public string Usage => "  spaces                        list available memory spaces";

        public global::EmuSen.DianaOS.DianaOS.Lib.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.DianaOS.Bin.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            var sb = new StringBuilder();
            sb.AppendLine($"{target.CoreName} memory spaces:");
            foreach (var s in target.GetMemorySpaces())
            {
                sb.AppendLine($"  {s.Name,-10} {s.Size,8} bytes  {(s.IsWritable ? "R/W" : "R/O")}");
            }
            return sb.ToString().TrimEnd();
        }
    }
}
