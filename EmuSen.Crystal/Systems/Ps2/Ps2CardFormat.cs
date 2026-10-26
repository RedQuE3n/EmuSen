using System;
using EmuSen.Crystal.SaveMedia;

namespace EmuSen.Crystal.Systems.Ps2
{
    // The PlayStation 2 memory card's file system, emusen-ps2-card through emusen-systems alone: no C# twin, so under EMUSEN_SYSTEMS_NATIVE=0 it reads nothing - see EmuSen_SaveMedia.md §7.4, Sapphire_Native.md §30.16.
    public sealed class Ps2CardFormat : ISaveMediumFormat
    {
        public static readonly Ps2CardFormat Instance = new();

        private Ps2CardFormat() { }

        public const string Id = "ps2.card";
        public const int CardLength = 8_650_752, Kilobytes = 7998;

        private const string Off = "the PlayStation 2 card's module is the platform library's, which is switched off (EMUSEN_SYSTEMS_NATIVE=0)";

        public string Medium => Id;
        public int Length => CardLength;
        public int Capacity => Kilobytes;
        public string Unit => "kilobyte";

        public MediumListing Read(ReadOnlySpan<byte> image) => SystemCode.Active ? SystemCode.MediaRead(Id, image) : MediumListing.Unreadable(Off);

        public MediumEdit Copy(ReadOnlySpan<byte> from, string save, ReadOnlySpan<byte> to) =>
            !SystemCode.Active ? MediumEdit.Refused(Off) : SystemCode.Takes(save) ? SystemCode.MediaCopy(Id, from, save, to) : MediumEdit.Refused($"{save} is not a name the card's file system holds");

        public MediumEdit Delete(ReadOnlySpan<byte> image, string save) =>
            !SystemCode.Active ? MediumEdit.Refused(Off) : SystemCode.Takes(save) ? SystemCode.MediaDelete(Id, image, save) : MediumEdit.Refused($"{save} is not a name the card's file system holds");

        public byte[]? Format() => SystemCode.Active ? SystemCode.MediaFormat(Id) : null;
    }
}
