using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform;
using EmuSen.LunaP.Settings;

namespace EmuSen.LunaP.Windowing
{
    // Where a remembered window was, as plain data - see EmuSen_LunaP.md §8.1.
    public sealed class WindowPlacement
    {
        public int X { get; set; }
        public int Y { get; set; }
        public double Width { get; set; }
        public double Height { get; set; }
        public bool Maximized { get; set; }
    }

    // One windows.json keyed by ToolWindow.WindowKey; opt-in, so a window without a key is never remembered.
    public static class WindowPlacementStore
    {
        public const string FileName = "windows.json";

        // Read through LunaSettings.Store rather than a captured file object: a host may replace the store after this type is first touched.
        private static Dictionary<string, WindowPlacement> All() =>
            LunaSettings.Store.Load<Dictionary<string, WindowPlacement>>(null, FileName) ?? new Dictionary<string, WindowPlacement>();

        public static WindowPlacement? Load(string key) =>
            All().TryGetValue(key, out WindowPlacement? p) ? p : null;

        public static void Save(string key, WindowPlacement placement)
        {
            Dictionary<string, WindowPlacement> all = All();
            all[key] = placement;
            LunaSettings.Store.Save(null, FileName, all);
        }

        // A monitor that is no longer attached would otherwise restore a window off every screen, where it cannot be dragged back.
        public static bool IsOnAScreen(Screens? screens, PixelRect bounds)
        {
            if (screens is null) return true;

            var all = new List<PixelRect>(screens.All.Count);
            foreach (Screen screen in screens.All) all.Add(screen.Bounds);
            return IsOnAScreen(all, bounds);
        }

        // Split out from the Screens overload so the rule itself is testable without a display - see EmuSen_LunaP.md §8.1.
        public static bool IsOnAScreen(IReadOnlyList<PixelRect> screenBounds, PixelRect bounds)
        {
            // Nothing to check against is not the same as "off screen"; refusing here would strand the window at the default position instead.
            if (screenBounds.Count == 0) return true;

            foreach (PixelRect screen in screenBounds)
            {
                if (screen.Intersects(bounds)) return true;
            }

            return false;
        }
    }
}
