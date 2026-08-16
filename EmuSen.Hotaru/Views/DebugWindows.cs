using System;
using EmuSen.DianaOS.DianaOS.Lib;
using EmuSen.DianaOS;
using EmuSen.Serenity.Dashboards;
using EmuSen.LunaP.Windowing;

namespace EmuSen.Hotaru.Views
{
    // `coretop -w` / `feed -w` window management - see EmuSen_Frontend_Driver.md. WindowSlot owns the at-most-one rule and the thread marshalling.
    internal static class DebugWindows
    {
        private static readonly WindowSlot<CoretopWindow> Coretop = new();
        private static readonly WindowSlot<FeedWindow> Feed = new();
        private static readonly WindowSlot<HotkeyHelpWindow> Hotkeys = new();

        public static void ShowCoretopWindow(IDebugTarget target) =>
            Coretop.Show(null, () => new CoretopWindow(target), refresh: w => w.UpdateTarget(target));

        // Never creates and never activates: a `core <name> <path>` swap should not pop up a dashboard nobody
        // asked for, nor steal focus mid-gameplay - so this is safe to call on every swap. See EmuSen_LunaP.md §8.3.
        public static void UpdateCoretopWindowTargetIfOpen(IDebugTarget target) =>
            Coretop.RefreshIfOpen(w => w.UpdateTarget(target));

        // The only discoverability this frontend has: it owns no menu bar - see §3e.
        public static void ShowHotkeyHelpWindow() => Hotkeys.Show(null, () => new HotkeyHelpWindow());

        public static void ShowFeedWindow(Func<(byte[] Rgba, int Width, int Height)> frameProvider) =>
            Feed.Show(null, () => new FeedWindow(frameProvider));
    }
}
