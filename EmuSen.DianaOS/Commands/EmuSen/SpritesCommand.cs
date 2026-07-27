using System.Text;

namespace EmuSen.DianaOS.Commands.EmuSen
{
    public class SpritesCommand : global::EmuSen.DianaOS.IDianaOSCommand
    {
        public string Name => "sprites";
        public bool IsReadOnly => true;
        public string Usage => "  sprites                       active sprite/OBJ table";

        public global::EmuSen.DianaOS.DianaOSResult Execute(IDebugTarget? target, string[] parts, string? stdin)
        {
            target = global::EmuSen.DianaOS.Commands.EmuSen.DebugCommandHelpers.RequireTarget(target);
            var sprites = target.Sprites.Current;
            var sb = new StringBuilder();
            sb.AppendLine($"{sprites.Count} active sprite(s):");
            foreach (var s in sprites)
            {
                sb.AppendLine($"  #{s.Index}: x={s.X} y={s.Y} {s.Width}x{s.Height} tile=0x{s.TileIndex:X3} pal={s.PaletteIndex} pri={s.Priority} flipX={s.FlipX} flipY={s.FlipY}");
            }
            return sb.ToString().TrimEnd();
        }
    }
}
