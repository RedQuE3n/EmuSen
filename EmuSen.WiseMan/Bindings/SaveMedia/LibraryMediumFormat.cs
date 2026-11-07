using System;
using EmuSen.WiseMan.Bindings.Systems;

namespace EmuSen.WiseMan.Bindings.SaveMedia
{
    // A medium's file system answered by emusen-systems' module for the medium's id, every method pure - see EmuSen_SaveMedia.md §6.2, EmuSen_Backend.md §13.1, §40.1.
    public sealed class LibraryMediumFormat(string medium) : ISaveMediumFormat
    {
        public string Medium => medium;
        public int Length => SystemCode.Module(medium).Length;
        public int Capacity => SystemCode.Module(medium).Capacity;
        public string Unit => SystemCode.Module(medium).Unit;

        public MediumListing Read(ReadOnlySpan<byte> image) => SystemCode.MediaRead(medium, image);

        public MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to) => SystemCode.MediaCopy(medium, from, save, to);

        public MediumEdit Delete(ReadOnlySpan<byte> image, string save) => SystemCode.MediaDelete(medium, image, save);

        public byte[]? Format() => SystemCode.MediaFormat(medium);
    }
}
