namespace EmuSen.Galaxia.Models
{
    // On-disk mirror of EmuSen.Graphics.GraphicsSettings - see
    // EmuSen_Config_Reference.md §3.3. WindowTitle is deliberately absent:
    // it is branding, not a preference, and a corrupt config blanking the
    // title is a worse outcome than not being able to change it.
    public class GraphicsConfig
    {
        public int WindowWidth { get; set; } = 1060;
        public int WindowHeight { get; set; } = 580;
        public int TargetFps { get; set; } = 60;
        public bool VSyncEnabled { get; set; } = true;
        public bool WindowResizable { get; set; } = true;
        public bool BilinearFiltering { get; set; } = true;

        private static readonly ConfigFile<GraphicsConfig> File = new("graphics.json");

        public bool Save() => File.Save(this);

        public static GraphicsConfig Load() => File.Load(() => new GraphicsConfig());

        public static bool Exists => File.Exists;
    }
}
