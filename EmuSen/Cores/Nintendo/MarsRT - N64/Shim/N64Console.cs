using System;
using System.Collections.Generic;
using EmuSen.Galaxia.Input;

namespace EmuSen.Cores.Nintendo.MarsRT
{
    // The N64's facts a frontend reads with no library loaded: its clock, screen, pad, ports and settings - see Mars_Native.md §6.7.
    public static class N64Console
    {
        // The VR4300's pipeline clock, which the machine's cycle count counts - see Mars_Memory.md §3.
        public const long ProcessorClockHz = 93_750_000;

        // A little over the longest field the VI's registers can describe, so it only ends a frame the VI never would - see Mars_Core.md §3.
        public const long CycleCap = 4_100_000;

        // The VI's raster width, and an NTSC picture's height before a game programs it - see Mars_Core.md §2.
        public const int ScreenWidthPixels = 640;
        public const int DefaultScreenHeight = 480;

        public const int Ports = 4;

        // The Controller Pak's own file beside the cartridge's - see Mars_Save.md §7.
        public const string PakExtension = ".mpk";

        // The right stick stands in for the four C buttons, pressed past half its travel - see Mars_Core.md §5.
        public const double CButtonThreshold = 0.5;

        public static IReadOnlyList<PadButton> PadButtons { get; } = new[]
        {
            PadButton.Up, PadButton.Down, PadButton.Left, PadButton.Right,
            PadButton.A, PadButton.B, PadButton.Start, PadButton.L, PadButton.R, PadButton.L2,
        };

        public static IReadOnlyList<PadAxis> PadAxes { get; } = new[] { PadAxis.LeftX, PadAxis.LeftY, PadAxis.RightX, PadAxis.RightY };

        // The picture's lines at the multiple its width says, or zero for a width that is no multiple of the raster's.
        public static int LinesOf(int width, int height) => width >= ScreenWidthPixels && width % ScreenWidthPixels == 0 ? height / (width / ScreenWidthPixels) : 0;

        // The one rule the core and the settings window read: the averaging gives way first when the two pass four - see Mars_Video.md §2.10.
        public static int EffectiveAntialiasingOf(int renderScale, int antialiasing) => Math.Max(1, Math.Min(Math.Clamp(antialiasing, 1, 4), 4 / Math.Clamp(renderScale, 1, 4)));

        // The Antialiasing row's note: what the chosen level becomes at the chosen resolution, when that is less - see Mars_Performance.md §42.5.
        public static string? AntialiasingNote(Func<string, string> value)
        {
            int scale = int.TryParse(value("RenderScale"), out int s) ? s : 1;
            string chosen = value("Antialiasing");
            int wanted = chosen is "2x" or "3x" or "4x" ? chosen[0] - '0' : 1;
            int effective = EffectiveAntialiasingOf(scale, wanted);
            if (effective >= wanted) return null;
            return effective == 1
                ? $"Off at {scale}x resolution: the two together are held to four, so {chosen} has no effect."
                : $"{effective}x at {scale}x resolution: the two together are held to four.";
        }

        // The settings the graphics window offers for the console, with the defaults Mistress used to set by hand - see Mars_Core.md §10.
        public static readonly IReadOnlyList<CoreSetting> VideoSettings = new CoreSetting[]
        {
            new("ThreadedRdp", "Draw on a separate thread", "The display processor runs its lists on a thread of its own, behind marks on the memory it reaches. Exact; faster on any machine with two cores to spare.", CoreSettingKind.Switch, "true"),
            new("RdpWorkers", "Rasteriser threads", "How many processors share each list, each shading every Nth row. Exact at any count; past two, each adds less. One per three cores is the default.", CoreSettingKind.Count, Math.Clamp(Environment.ProcessorCount / 3, 1, 4).ToString(), 1, 8),
            new("DeferredPresentation", "Scan out while the next frame runs", "The picture is finished on another thread while the machine runs the next frame, so it reaches the screen one frame late. Off, the frame waits for its picture.", CoreSettingKind.Switch, "true"),
            new("SkipRepeatedScans", "Skip a scan that repeats the last", "A scan whose registers and bytes match the last walk is not walked again. Exact; the picture is the same either way.", CoreSettingKind.Switch, "true"),
            new("RenderScale", "Internal resolution", "The picture drawn at a multiple of the console's, beside the exact drawing games read back. Each step costs its square in drawing: 2x is four times the pixels, 4x sixteen. Threads help; 2x is what most machines can hold at full speed.", CoreSettingKind.Choice, "1", Choices: new[] { "1", "2", "3", "4" }),
            new("Antialiasing", "Antialiasing", "Each pixel of the picture averaged from a drawing that many times finer each way, which smooths edges and shimmering textures. It multiplies the drawing's cost like the internal resolution, and the two together are held to four: at 2x resolution the most is 2x.", CoreSettingKind.Choice, "Off", Choices: new[] { "Off", "2x", "3x", "4x" }, Note: AntialiasingNote),
            new("Gpu", "Draw the multiple on the graphics card", "The picture at an internal resolution above one is shaded by the graphics card rather than by the processor's own threads, which is where nearly all of that setting's cost is. It needs Vulkan; without it the setting does nothing and the processor draws as before. It changes nothing at 1x, and the drawing the game itself reads back is never the card's.", CoreSettingKind.Switch, "false"),
            new("ExpansionPak", "Expansion Pak", "The memory accessory that doubles the console's 4MB. A few games refuse to start without it and more use it when it is there. Takes effect when a game is next loaded; a save state resumes with the memory it was made with.", CoreSettingKind.Switch, "true"),
        };
    }
}
