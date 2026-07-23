using Raylib_cs;

namespace EmuSen.Graphics
{
    // Central hub for display/presentation options - see
    // Man pages/EmuSen_Settings_Reference.md §3. None of this affects
    // emulation correctness - that's DebugSettings.
    public static class GraphicsSettings
    {
        public static int WindowWidth = 1060;
        public static int WindowHeight = 580;

        public static string WindowTitle = "EmuSen";

        public static int TargetFps = 60;

        public static bool VSyncEnabled = true;
        public static bool WindowResizable = true;
        public static bool BilinearFiltering = true;

        public static bool ShowDebugPanels = true;

        public static Color PanelBackgroundColor = new Color(24, 24, 28, 255);
        public static Color LetterboxColor = new Color(0, 0, 0, 255);
    }
}
