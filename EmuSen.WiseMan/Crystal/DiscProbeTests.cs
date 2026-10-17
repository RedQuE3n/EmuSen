using System;
using System.Buffers.Binary;
using System.Threading.Tasks;
using EmuSen.Crystal.Systems;
using EmuSen.Crystal.Systems.Psx;
using EmuSen.WiseMan.Fixtures;

namespace EmuSen.WiseMan.Crystal
{
    // A PlayStation disc whose volume or root directory is malformed still reads as a PlayStation disc with no serial, in both halves of the recogniser - see EmuSen_Backend.md §13.3.
    public class DiscProbeTests
    {
        private sealed class Probe(byte[] bytes) : IMediaProbe
        {
            public string Path => "disc.iso";
            public string ImageName => "disc.iso";
            public byte[] Read(int length) => bytes.Length <= length ? bytes : bytes[..length];
        }

        private const int Pvd = 16 * 2048, Root = 20 * 2048;

        private static byte[] Disc() => SyntheticPsxDisc.Build().Iso;

        // Both recognisers on the image, each within five seconds, as text so that an exception is an answer too.
        private static void ReadsWithNoSerial(byte[] image)
        {
            foreach (var recognise in new[] { PsxSystem.Literal.Recognise!, PsxSystem.Entry.Recognise! })
            {
                var task = Task.Run(() =>
                {
                    try { return recognise(new Probe(image)) is { } r ? $"{r.SystemId} {r.Serial ?? "no serial"} {r.Region}" : "none"; }
                    catch (Exception e) { return "threw " + e.GetType().Name; }
                });
                Assert.True(task.Wait(TimeSpan.FromSeconds(5)), "the recogniser did not return");
                Assert.Equal("psx no serial Americas", task.Result);
            }
        }

        [Fact]
        public void A_directory_record_running_past_its_sector_ends_the_scan()
        {
            byte[] image = Disc();
            image.AsSpan(Root, 2048).Clear();
            for (int at = 0; at < 2000; at += 250)
            {
                image[Root + at] = 250;
                image[Root + at + 32] = 8;
                "NOTTHIS!"u8.CopyTo(image.AsSpan(Root + at + 33));
            }
            image[Root + 2000] = 40;
            image[Root + 2000 + 32] = 100;
            ReadsWithNoSerial(image);
        }

        [Fact]
        public void A_root_directory_before_the_first_sector_is_none()
        {
            byte[] image = Disc();
            BinaryPrimitives.WriteInt32LittleEndian(image.AsSpan(Pvd + 158), -5);
            ReadsWithNoSerial(image);
        }

        [Fact]
        public void A_root_directory_claiming_two_gigabytes_is_scanned_and_ends()
        {
            byte[] image = Disc();
            BinaryPrimitives.WriteInt32LittleEndian(image.AsSpan(Pvd + 166), int.MaxValue);
            image.AsSpan(Root, 2048).Clear();
            ReadsWithNoSerial(image);
        }

        [Fact]
        public void A_file_of_negative_or_vast_length_is_none()
        {
            foreach (int length in new[] { -1, int.MaxValue })
            {
                byte[] image = Disc();
                for (int at = 0; image[Root + at] != 0; at += image[Root + at])
                    if (image[Root + at + 33] == (byte)'S') BinaryPrimitives.WriteInt32LittleEndian(image.AsSpan(Root + at + 10), length);
                ReadsWithNoSerial(image);
            }
        }
    }
}
