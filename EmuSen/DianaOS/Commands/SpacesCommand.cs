using System.Text;

namespace EmuSen.DianaOS.Commands
{
    public class SpacesCommand : EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "spaces";
        public string Usage => "  spaces                        list available memory spaces";

        public EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = EmuSen.DianaOS.Commands.DebugCommandHelpers.RequireTarget(target);
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
