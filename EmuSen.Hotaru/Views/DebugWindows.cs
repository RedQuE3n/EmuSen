using System;
using EmuSen.DianaSh;
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

        // Over the window's session as the API answers it, read at each poll, so a swap needs no update - see EmuSen_Backend.md §14.11.
        public static void ShowCoretopWindow(IHotaruHost host) => Coretop.Show(null, () => CoretopWindow.Over(new Readout(host)));

        // The window's current session read through the binding; a call refused, a closed session's among them, reads as nothing.
        private sealed class Readout(IHotaruHost host) : ISessionReadout
        {
            public string? Telemetry(uint kind) => Read(() => host.Session.Telemetry(kind));
            public string? Info() => Read(() => host.Session.ServiceInfo());
            public string? Summary() => Read(() => host.Session.Summary());
            public byte[]? TileSheet() => Read(() => host.Session.TileSheet());
            public byte[]? PaletteSwatch() => Read(() => host.Session.PaletteSwatch());
            public byte[]? AudioPeek() => Read(() => host.Session.AudioPeek());

            private static T? Read<T>(Func<T> call) where T : class
            {
                try { return call(); }
                catch (EmuSen.Crystal.Client.CrystalException) { return null; }
            }
        }

        // The only discoverability this frontend has: it owns no menu bar - see §3e.
        public static void ShowHotkeyHelpWindow() => Hotkeys.Show(null, () => new HotkeyHelpWindow());

        public static void ShowFeedWindow(Func<(byte[] Rgba, int Width, int Height)> frameProvider) =>
            Feed.Show(null, () => new FeedWindow(frameProvider));
    }
}
