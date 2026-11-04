using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using EmuSen.Galaxia.Library;
using EmuSen.Galaxia.Models;

namespace EmuSen.Common.Firmware
{
    // One request found: the folder it was in, and its bytes whole or as the parts of a split form, in order; ByContent when recognised by its MD5, Names then the player's file.
    public sealed record FirmwareFound(string Folder, IReadOnlyList<string> Names, IReadOnlyList<byte[]> Parts, bool ByContent = false)
    {
        public bool Whole => Parts.Count == 1;
    }

    // Where a request's image is, without its bytes.
    public sealed record FirmwareLocated(string Folder, IReadOnlyList<string> Names, bool Whole, bool ByContent = false);

    // home/Firmware, and the player's chosen folder before it, as a core-agnostic store of firmware images, emusen-firmware's - see EmuSen_Firmware.md §2, EmuSen_Backend.md §15.1.
    public static class FirmwareLibrary
    {
        private static string? _directoryOverride;

        // Settable so a test can point at a temp folder instead of the real one; null means the sandbox's own.
        public static string Directory
        {
            get => _directoryOverride ?? DataStore.Firmware;
            set => _directoryOverride = value;
        }

        public static void ResetDirectory() => _directoryOverride = null;

        // The folders searched, first to last: a test's override alone, else the order of EmuSen_Firmware.md §2.3, from emusen-firmware.
        public static IReadOnlyList<string> Folders => _directoryOverride is { } only ? [only] : FirmwareNative.Folders(AppSettings.Load().FirmwareDirectory, Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), DataStore.Firmware);

        // Where Install writes: home/Firmware, never the player's chosen folder.
        public static string PathFor(FirmwareRequest request) => Path.Combine(Directory, request.FileName);

        // The first folder holding the image, whole by any of its names, as a complete split form, or by its content - see EmuSen_Firmware.md §2.3 and §7.
        public static FirmwareFound? Find(FirmwareRequest request) => Find(request, Folders);

        public static FirmwareFound? Find(FirmwareRequest request, IReadOnlyList<string> folders) => FirmwareNative.Find(request, folders);

        // Where Find would take the image, unread, and else the first folder with a file of its name and another size, for the overview's row.
        public static (FirmwareLocated? Found, string? WrongSizeIn) Locate(FirmwareRequest request, IReadOnlyList<string> folders) => FirmwareNative.Locate(request, folders);

        // The image whole where Find takes it whole, else null - see EmuSen_Firmware.md §2.1.
        public static byte[]? TryLoad(FirmwareRequest request) => Find(request) is { Whole: true } found ? found.Parts[0] : null;

        public static bool IsInstalled(FirmwareRequest request) => Locate(request, Folders).Found != null;

        // The parts where Find takes a split form, all from one folder and together the whole's size (any size where the request gives none), else null.
        public static IReadOnlyList<byte[]>? TryLoadParts(FirmwareRequest request) => Find(request) is { Whole: false } found ? found.Parts : null;

        public static IReadOnlyList<FirmwareRequest> MissingFrom(IEnumerable<FirmwareRequest> requests)
        {
            IReadOnlyList<string> folders = Folders;
            return requests.Where(r => Locate(r, folders).Found is null).ToArray();
        }

        // Copies a file the player picked into home/Firmware under its canonical name, so nothing has to ask again.
        public static bool Install(FirmwareRequest request, string sourcePath, Action<string>? say = null)
        {
            try { return FirmwareNative.Install(request, sourcePath, Directory); }
            catch (Crystal.Services.ServiceFault fault) when (fault.Status == Crystal.Services.ServiceFault.Io)
            {
                say?.Invoke($"[Firmware] Could not install {request.FileName}: {fault.Message}");
                return false;
            }
        }
    }
}
