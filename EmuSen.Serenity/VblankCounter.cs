using System;
using System.Runtime.InteropServices;

namespace EmuSen.Serenity
{
    // The display's own vblank count and time, read on the render thread through GLX_OML_sync_control - see EmuSen_Settings_Reference.md §4.87.2.
    public static unsafe class VblankCounter
    {
        private static int _state; // 0 unprobed, 1 usable, -1 absent
        private static delegate* unmanaged<IntPtr, IntPtr, long*, long*, long*, int> _getSyncValues;

        [DllImport("libGL.so.1")] private static extern IntPtr glXGetCurrentDisplay();
        [DllImport("libGL.so.1")] private static extern IntPtr glXGetCurrentDrawable();
        [DllImport("libGL.so.1")] private static extern IntPtr glXGetProcAddressARB([MarshalAs(UnmanagedType.LPStr)] string name);

        // True with the last vblank's UST (microseconds, CLOCK_MONOTONIC) and its count, when a GLX drawable is current on this thread.
        public static bool TrySample(out long ustMicroseconds, out long msc, out long sbc)
        {
            ustMicroseconds = msc = sbc = 0;
            if (_state < 0) return false;
            try
            {
                if (_state == 0)
                {
                    if (!OperatingSystem.IsLinux()) { _state = -1; return false; }
                    IntPtr fn = glXGetProcAddressARB("glXGetSyncValuesOML");
                    if (fn == IntPtr.Zero) { _state = -1; return false; }
                    _getSyncValues = (delegate* unmanaged<IntPtr, IntPtr, long*, long*, long*, int>)fn;
                    _state = 1;
                }
                IntPtr display = glXGetCurrentDisplay(), drawable = glXGetCurrentDrawable();
                if (display == IntPtr.Zero || drawable == IntPtr.Zero) return false;
                long ust, m, s;
                if (_getSyncValues(display, drawable, &ust, &m, &s) == 0) return false;
                ustMicroseconds = ust; msc = m; sbc = s;
                return true;
            }
            catch (Exception ex) when (ex is DllNotFoundException or EntryPointNotFoundException)
            {
                _state = -1;
                return false;
            }
        }
    }
}
