using System.Text;

namespace EmuSen.Debug.Commands
{
    public class SpacesCommand : IDebugCommand
    {
        public string Name => "spaces";
        public string Usage => "  spaces                        list available memory spaces";

        public string Execute(IDebugTarget target, string[] parts)
        {
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
