using System;
using System.IO;
using System.Text;
using EmuSen.Galaxia.Media;

namespace EmuSen.Galaxia.Native
{
    // A packaged game's soft patches applied by emusen-patch through the platform library, which owns the formats; this has no C# rule - see EmuSen_GamePackages.md §5.4 and §15.
    public static unsafe class SoftPatches
    {
        private static readonly Lazy<(nint Apply, nint LastError)> Exports = new(() =>
            (PlatformLibrary.Export("emusen_patch_package_image"), PlatformLibrary.Export("emusen_platform_last_error")));

        // The image with the package's patches applied, held as a mapping; null where the package lists none. IOException with the words where one does not apply or the platform library is not in use.
        public static MappedFile? Apply(string package, MappedFile image)
        {
            if (!Lists(package)) return null;
            var (apply, _) = Exports.Value;
            if (apply == 0) throw new IOException($"{Path.GetFileName(Path.TrimEndingDirectorySeparator(package))} has soft patches, which {PlatformLibrary.FileName} applies, and it is not in use here ({PlatformLibrary.Report})");
            byte[] folder = Encoding.UTF8.GetBytes(package);
            nint handle = 0;
            int status;
            fixed (byte* f = folder) status = ((delegate* unmanaged[Cdecl]<byte*, nuint, byte*, nuint, nint*, int>)apply)(f, (nuint)folder.Length, image.Pointer, (nuint)image.Length, &handle);
            if (status < 0) throw new IOException(LastError());
            return handle == 0 ? null : MappedFile.FromNative(handle);
        }

        // Whether the manifest lists any patch; a manifest that does not read is left to the library to say why.
        private static bool Lists(string package)
        {
            try { return GamePackage.Read(Path.Combine(package, GamePackage.FileName)).Patches > 0; }
            catch (Exception e) when (e is IOException or FormatException or UnauthorizedAccessException) { return true; }
        }

        private static string LastError()
        {
            var call = (delegate* unmanaged[Cdecl]<byte*, nuint, long>)Exports.Value.LastError;
            if (call == null) return "the soft patches did not apply";
            byte[] buffer = new byte[1024];
            long length;
            fixed (byte* b = buffer) length = call(b, (nuint)buffer.Length);
            if (length > buffer.Length)
            {
                buffer = new byte[length];
                fixed (byte* b = buffer) length = call(b, (nuint)buffer.Length);
            }
            return length <= 0 ? "the soft patches did not apply" : Encoding.UTF8.GetString(buffer, 0, (int)Math.Min(length, buffer.Length));
        }
    }
}
