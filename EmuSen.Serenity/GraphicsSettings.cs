using System;

namespace EmuSen.Graphics
{
    // Display options; none of it affects emulation - see EmuSen_Settings_Reference.md §3.
    public static class GraphicsSettings
    {
        public static int WindowWidth = 1060;
        public static int WindowHeight = 580;

        public static string WindowTitle = "EmuSen";

        // Wired to nothing; Avalonia's compositor paces frames - see EmuSen_Settings_Reference.md §3.
        public static int TargetFps = 60;

        public static bool VSyncEnabled = true;
        public static bool WindowResizable = true;
        public static bool BilinearFiltering = true;

        // Applies etc/EmuSen/graphics.json, seeding it on first run - see EmuSen_Config_Reference.md §3.3.
        public static void LoadFromDisk()
        {
            bool seed = !Galaxia.Models.GraphicsConfig.Exists;
            var config = Galaxia.Models.GraphicsConfig.Load();

            WindowWidth = Math.Clamp(config.WindowWidth, 256, 16384);
            WindowHeight = Math.Clamp(config.WindowHeight, 224, 16384);
            TargetFps = Math.Clamp(config.TargetFps, 1, 1000);
            VSyncEnabled = config.VSyncEnabled;
            WindowResizable = config.WindowResizable;
            BilinearFiltering = config.BilinearFiltering;

            if (seed) SaveToDisk();
        }

        public static bool SaveToDisk() => new Galaxia.Models.GraphicsConfig
        {
            WindowWidth = WindowWidth,
            WindowHeight = WindowHeight,
            TargetFps = TargetFps,
            VSyncEnabled = VSyncEnabled,
            WindowResizable = WindowResizable,
            BilinearFiltering = BilinearFiltering,
        }.Save();
    }
}
