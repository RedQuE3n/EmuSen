using System;
using System.Reflection;

namespace EmuSen.Serenity
{
    // The rate Avalonia's sleeping render timer ticks at, which quantises when a handed-over picture is drawn - see EmuSen_Settings_Reference.md §4.87.10.
    public static class RenderTimerRate
    {
        private const BindingFlags Any = BindingFlags.Instance | BindingFlags.Static | BindingFlags.Public | BindingFlags.NonPublic;
        private static object? _timer;
        private static PropertyInfo? _fps;
        private static bool _looked;

        // Reached by reflection, since Avalonia marks the loop and its timer private API; null when the timer is not the sleeping one.
        private static bool Find()
        {
            if (_looked) return _fps is not null;
            _looked = true;
            try
            {
                Assembly avalonia = typeof(Avalonia.AvaloniaObject).Assembly;
                Type? locator = avalonia.GetType("Avalonia.AvaloniaLocator");
                Type? loopType = avalonia.GetType("Avalonia.Rendering.IRenderLoop");
                object? current = locator?.GetProperty("Current", Any)?.GetValue(null);
                object? loop = current is null || loopType is null ? null : current.GetType().GetMethod("GetService", new[] { typeof(Type) })?.Invoke(current, new object[] { loopType });
                object? timer = loop?.GetType().GetProperty("Timer", Any)?.GetValue(loop);
                _fps = RateOf(timer);
                _timer = _fps is null ? null : timer;
            }
            catch (Exception) { _fps = null; }
            return _fps is not null;
        }

        // The rate property of Avalonia's sleeping timer, and of nothing else.
        internal static PropertyInfo? RateOf(object? timer) =>
            timer?.GetType().FullName == "Avalonia.Rendering.SleepLoopRenderTimer" ? timer.GetType().GetProperty("DesiredFps", Any) : null;

        // Sets a timer's rate as Hold does, for a test that makes its own.
        internal static bool Hold(object timer, int fps)
        {
            if (fps < 1 || RateOf(timer) is not { } rate) return false;
            if ((int)rate.GetValue(timer)! != fps) rate.SetValue(timer, fps);
            return true;
        }

        // The rate now, or 0 when there is no sleeping timer.
        public static int Current => Find() ? (int)_fps!.GetValue(_timer)! : 0;

        // Keeps the timer at fps, which the platform resets whenever the screens change; false when there is no such timer.
        public static bool Hold(int fps)
        {
            if (fps < 1 || !Find()) return false;
            if ((int)_fps!.GetValue(_timer)! != fps) _fps.SetValue(_timer, fps);
            return true;
        }
    }
}
