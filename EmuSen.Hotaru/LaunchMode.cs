using System;
using System.Collections.Generic;

namespace EmuSen.Hotaru
{
    // Which shell a launch gets - see EmuSen_Frontend_Driver.md §3c.
    public enum ShellMode
    {
        Console,
        Window,
    }

    // Decides console vs window, and the flags that override it - see EmuSen_Frontend_Driver.md §3c.
    public static class LaunchMode
    {
        public const string ForceConsoleFlag = "--console";
        public const string ForceWindowFlag = "--shell-window";

        // Kept pure so the decision is testable without a display or a terminal.
        public static ShellMode Decide(IReadOnlyList<string> args, bool interactiveTerminal, bool displayAvailable)
        {
            foreach (string arg in args)
            {
                if (arg.Equals(ForceConsoleFlag, StringComparison.OrdinalIgnoreCase)) return ShellMode.Console;
                if (arg.Equals(ForceWindowFlag, StringComparison.OrdinalIgnoreCase)) return ShellMode.Window;
            }

            // No display means no window is possible at all, whatever the terminal looks like.
            if (!displayAvailable) return ShellMode.Console;

            // A terminal launch asked for a terminal; only a launch with nowhere to type needs the window.
            return interactiveTerminal ? ShellMode.Console : ShellMode.Window;
        }

        // The ROM path is the first non-flag argument, so a flag cannot be mistaken for one.
        public static string? RomPathFrom(IReadOnlyList<string> args)
        {
            foreach (string arg in args)
            {
                if (!arg.StartsWith("--", StringComparison.Ordinal)) return arg;
            }
            return null;
        }

        public static bool InteractiveTerminal()
        {
            // Console.IsInputRedirected throws if there is no stdin handle at all.
            try { return !Console.IsInputRedirected; }
            catch (InvalidOperationException) { return false; }
            catch (System.IO.IOException) { return false; }
        }

        // Windows and macOS always have a window server; on Linux it is X11 or Wayland or nothing.
        public static bool DisplayAvailable() =>
            !OperatingSystem.IsLinux()
            || !string.IsNullOrEmpty(Environment.GetEnvironmentVariable("WAYLAND_DISPLAY"))
            || !string.IsNullOrEmpty(Environment.GetEnvironmentVariable("DISPLAY"));
    }
}
