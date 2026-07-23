using System.Text;

namespace EmuSen.Debug.Commands
{
    public class SpritesCommand : IDebugCommand
    {
        public string Name => "sprites";
        public string Usage => "  sprites                       active sprite/OBJ table";

        public string Execute(IDebugTarget target, string[] parts)
        {
            var sprites = target.GetSprites();
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
