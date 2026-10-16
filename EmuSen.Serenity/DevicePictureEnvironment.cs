using System;
using System.Runtime.InteropServices;

namespace EmuSen.Serenity
{
    // Mesa's GLX asked to keep the window's drawable when Avalonia binds its pbuffer, as the device path's imports make it do - see EmuSen_DevicePicture.md §18.
    public static class DevicePictureEnvironment
    {
        public const string Variable = "keep_native_window_glx_drawable";

        // Set in the native environment, where Mesa reads it, unless already set; called before Avalonia makes its GLX screen.
        public static void Prepare()
        {
            if (OperatingSystem.IsLinux() || OperatingSystem.IsFreeBSD()) SetEnv(Variable, "true", 0);
        }

        [DllImport("libc", EntryPoint = "setenv")]
        private static extern int SetEnv(string name, string value, int overwrite);
    }
}
