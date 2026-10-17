using System;
using System.Runtime.InteropServices;
using Silk.NET.Core.Contexts;
using Silk.NET.Core.Loader;
using Silk.NET.Shaderc;

namespace EmuSen.Serenity.Slang
{
    // Shaderc's binding over the copy the program ships, found by .NET's own native probing rather than Silk.NET's list of distributions - see EmuSen_RustPlatform.md §32.
    internal static class ShadercLibrary
    {
        // The file Silk.NET.Shaderc.Native ships for this system.
        internal static string FileName => OperatingSystem.IsWindows() ? "shaderc_shared.dll" : OperatingSystem.IsMacOS() ? "libshaderc_shared.dylib" : "libshaderc_shared.so";

        internal static Shaderc Load() => new(new DefaultNativeContext(new UnmanagedLibrary(FileName, new AssemblyProbing(), PathResolver.Default)));

        // DllImport's order for this assembly: the host's native directories from the deps file, then the assembly's folder, then the system's.
        private sealed class AssemblyProbing : LibraryLoader
        {
            protected override nint CoreLoadNativeLibrary(string name) =>
                NativeLibrary.TryLoad(name, typeof(ShadercLibrary).Assembly, null, out nint handle) ? handle : 0;

            protected override void CoreFreeNativeLibrary(nint handle)
            {
                if (handle != 0) NativeLibrary.Free(handle);
            }

            protected override nint CoreLoadFunctionPointer(nint handle, string functionName) =>
                NativeLibrary.TryGetExport(handle, functionName, out nint address) ? address : 0;
        }
    }
}
