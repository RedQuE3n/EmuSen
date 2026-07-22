using Raylib_cs;

namespace EmuSen.Graphics
{
    // Central hub for display/presentation options: window size, title, vsync,
    // target frame rate, texture filtering, and whether to show the debug side
    // panels. None of this affects emulation correctness - that's DebugSettings.
    // This is purely "how the output window looks," so Renderer.cs reads these
    // instead of hardcoding them.
    public static class GraphicsSettings
    {
        // Window/canvas size in pixels. The real SNES resolution (256x224) is
        // fixed hardware fact and lives in Renderer.cs as ScreenW/ScreenH, not
        // here - this is just how big the window is on screen, with room for
        // the debug side panels alongside the game view.
        public static int WindowWidth = 1060;
        public static int WindowHeight = 580;

        public static string WindowTitle = "EmuSen";

        // The emulator's own frame pacing target - separate from VSyncEnabled,
        // which asks the OS/driver to sync to the display's refresh rate.
        public static int TargetFps = 60;

        public static bool VSyncEnabled = true;
        public static bool WindowResizable = true;

        // Bilinear smooths the upscaled image; Point keeps hard pixel edges (the
        // classic "sharp pixel" look). Point is generally more authentic for
        // pixel art; Bilinear can look better at non-integer scale factors.
        public static bool BilinearFiltering = true;

        // Show the VRAM tile sheet + CGRAM palette panels alongside the game
        // view. Turn off for a clean, game-only window. Note: turning this off
        // currently just skips drawing the panels - the game view itself stays
        // in its current position/size rather than re-centering to fill the
        // freed space, which would be a separate, bigger layout change.
        public static bool ShowDebugPanels = true;

        // Background behind the debug panels (the dark canvas the game view and
        // side panels sit on top of).
        public static Color PanelBackgroundColor = new Color(24, 24, 28, 255);

        // Color of the letterbox bars around the final scaled output when the
        // window aspect ratio doesn't match the virtual canvas.
        public static Color LetterboxColor = new Color(0, 0, 0, 255);
    }
}
