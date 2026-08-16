using System.Collections.Generic;
using Avalonia.Input;

namespace EmuSen.Hotaru.Input
{
    // What a non-gameplay key does, as data - see EmuSen_Frontend_Driver.md §3e.
    public enum HotaruHotkey
    {
        Summary,
        DumpOam,
        Screenshot,
        DebugPrompt,
        SaveState,
        ToggleRecording,
        MirrorToggle,
        CycleShader,
        LoadState,
        DumpBackdrop,
        StartBgScrollTrace,
        ShowHotkeys,
        FastForward,
        Rewind,
    }

    // One table for dispatch and for the help window - see EmuSen_Frontend_Driver.md §3e.
    public static class HotaruHotkeys
    {
        public sealed record Entry(HotaruHotkey Action, Key Key, string Name, string Held);

        // Held keys are layered rather than edge-triggered - see EmuSen_Rewind_And_FastForward.md §4.
        public static readonly IReadOnlyList<Entry> All = new[]
        {
            new Entry(HotaruHotkey.Summary, Key.F1, "Print a frame summary", ""),
            new Entry(HotaruHotkey.DumpOam, Key.F2, "Dump OAM", ""),
            new Entry(HotaruHotkey.Screenshot, Key.F3, "Save a screenshot", ""),
            new Entry(HotaruHotkey.DebugPrompt, Key.F4, "Open the debug prompt", ""),
            new Entry(HotaruHotkey.SaveState, Key.F5, "Save state", ""),
            new Entry(HotaruHotkey.ToggleRecording, Key.F6, "Start or stop frame recording", ""),
            new Entry(HotaruHotkey.MirrorToggle, Key.F7, "Mirror player 1 to player 2", ""),
            new Entry(HotaruHotkey.CycleShader, Key.F8, "Cycle the shader effect", ""),
            new Entry(HotaruHotkey.LoadState, Key.F9, "Load state", ""),
            new Entry(HotaruHotkey.DumpBackdrop, Key.O, "Dump the backdrop", ""),
            new Entry(HotaruHotkey.StartBgScrollTrace, Key.P, "Start a BG scroll trace", ""),
            new Entry(HotaruHotkey.ShowHotkeys, Key.F12, "Show this list", ""),
            new Entry(HotaruHotkey.FastForward, Key.Tab, "Fast forward", "held"),
            new Entry(HotaruHotkey.Rewind, Key.Back, "Rewind", "held"),
        };

        private static readonly Dictionary<Key, HotaruHotkey> _byKey = Build();

        private static Dictionary<Key, HotaruHotkey> Build()
        {
            var map = new Dictionary<Key, HotaruHotkey>();
            foreach (Entry entry in All) map[entry.Key] = entry.Action;
            return map;
        }

        public static bool TryGetAction(Key key, out HotaruHotkey action) => _byKey.TryGetValue(key, out action);

        public static bool IsHeld(HotaruHotkey action) =>
            action is HotaruHotkey.FastForward or HotaruHotkey.Rewind;
    }
}
