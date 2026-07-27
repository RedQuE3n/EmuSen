using System.Text;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    public class SpacesCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "spaces";
        public bool IsReadOnly => true;
        public string Usage => "  spaces                        list available memory spaces";

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
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
