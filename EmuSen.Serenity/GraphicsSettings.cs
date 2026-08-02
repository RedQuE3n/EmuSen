using System;

namespace EmuSen.Graphics
{
    // Central hub for display/presentation options - see
    // EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Settings_Reference.md §3. None of this affects
    // emulation correctness - that's DebugSettings (EmuSen/Settings/).
    // Lives in EmuSen.Serenity, not EmuSen/Settings/, alongside
    // FramePresenter/BuiltInShaders - these are all presentation-layer
    // concerns any frontend can share, unlike DebugSettings/AudioSettings
    // which stay next to the emulation core they configure.
    //
    // ShowDebugPanels and the two Raylib_cs.Color fields (PanelBackground/
    // LetterboxColor) that used to live here were dropped along with the
    // on-window Raylib debug overlay itself (see git history and
    // EmuSen.DianaOS/DianaOS/Usr/Home/Documents/EmuSen Manual/EmuSen_Debugging_Tools_Reference_v5.md's own revision
    // note) - nothing reads them anymore.
    public static class GraphicsSettings
    {
        public static int WindowWidth = 1060;
        public static int WindowHeight = 580;

        public static string WindowTitle = "EmuSen";

        // Avalonia's own compositor drives frame pacing (typically tied
        // to the display's refresh rate) rather than an explicit target-
        // FPS cap the way Raylib's SetTargetFPS worked - kept as a data
        // field for parity/future use, not currently wired to anything.
        public static int TargetFps = 60;

        public static bool VSyncEnabled = true;
        public static bool WindowResizable = true;
        public static bool BilinearFiltering = true;

        // Applies etc/EmuSen/graphics.json, seeding it on first run so there
        // is something to hand-edit - see EmuSen_Config_Reference.md §3.3.
        // Clamped for the same reason AudioSettings.LoadFromDisk is.
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
