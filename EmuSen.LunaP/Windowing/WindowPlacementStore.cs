using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform;
using EmuSen.Galaxia;

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
        private static readonly ConfigFile<Dictionary<string, WindowPlacement>> File = new("windows.json");

        public static WindowPlacement? Load(string key) =>
            File.Load(() => new Dictionary<string, WindowPlacement>()).TryGetValue(key, out WindowPlacement? p) ? p : null;

        public static void Save(string key, WindowPlacement placement)
        {
            Dictionary<string, WindowPlacement> all = File.Load(() => new Dictionary<string, WindowPlacement>());
            all[key] = placement;
            File.Save(all);
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
