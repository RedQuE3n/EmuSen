using System;
using System.IO;

namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // The program and data ROMs masked into the DSP die. Nintendo never put
    // them on the cartridge bus, so they can only come from a dump - either
    // appended to the ROM file or dropped in Usr/Home/Firmware. See Venus_NecDSP.md §2.
    public sealed class NecDspFirmware
    {
        public byte[] Program { get; }
        public ushort[] DataRom { get; }

        private NecDspFirmware(byte[] program, ushort[] dataRom)
        {
            Program = program;
            DataRom = dataRom;
        }

        // How many bytes of firmware are appended to a ROM of this size, or 0
        // if none are - see Venus_NecDSP.md §2.1.
        public static int EmbeddedSize(int romSize)
        {
            if ((romSize & 0x7FFF) == 0x2000) return 0x2000;
            if ((romSize & 0xFFFF) == 0xD000) return 0xD000;
            return 0;
        }

        // Split one program+data blob. Returns null if it isn't the right length.
        public static NecDspFirmware? FromBlob(NecDspProfile profile, byte[] blob)
        {
            if (blob.Length != profile.FirmwareBytes) return null;

            byte[] program = new byte[profile.ProgramBytes];
            Array.Copy(blob, 0, program, 0, program.Length);

            // The data ROM is 16-bit words, stored little-endian in the dump.
            ushort[] data = new ushort[profile.DataRomBytes / 2];
            for (int i = 0; i < data.Length; i++)
            {
                data[i] = (ushort)(blob[profile.ProgramBytes + (i * 2)] | (blob[profile.ProgramBytes + (i * 2) + 1] << 8));
            }

            return new NecDspFirmware(program, data);
        }

        // What the core-agnostic firmware layer needs to find, validate and
        // prompt for this chip's dump - see EmuSen_Firmware.md §1.
        public static Common.Firmware.FirmwareRequest RequestFor(NecDspVariant variant)
        {
            NecDspProfile profile = NecDspProfile.For(variant);
            return new Common.Firmware.FirmwareRequest(
                CoreName: "SNES",
                ChipName: variant.ToString().ToUpperInvariant(),
                FileName: profile.FirmwareName + ".rom",
                Size: profile.FirmwareBytes,
                Purpose: $"the {variant} coprocessor's mask ROM, which the cartridge does not carry");
        }

        // Usr/Home/Firmware, as either one combined dump or the split pair.
        // The combined form goes through FirmwareLibrary so it shares the
        // whole engine's discovery and validation rules; the split pair is a
        // NEC-DSP-specific convention and stays here.
        public static NecDspFirmware? FromFirmwareDirectory(NecDspProfile profile)
        {
            NecDspVariant variant = VariantFor(profile);
            byte[]? combined = Common.Firmware.FirmwareLibrary.TryLoad(RequestFor(variant));
            if (combined != null) return FromBlob(profile, combined);

            string dir = Common.Firmware.FirmwareLibrary.Directory;
            string name = profile.FirmwareName;
            byte[]? program = TryRead(Path.Combine(dir, name + ".program.rom"), profile.ProgramBytes);
            byte[]? data = TryRead(Path.Combine(dir, name + ".data.rom"), profile.DataRomBytes);
            if (program == null || data == null) return null;

            byte[] blob = new byte[profile.FirmwareBytes];
            Array.Copy(program, 0, blob, 0, program.Length);
            Array.Copy(data, 0, blob, program.Length, data.Length);
            return FromBlob(profile, blob);
        }

        // A profile carries no back-reference, and the firmware name is
        // unique across the seven, so this recovers the variant from it.
        private static NecDspVariant VariantFor(NecDspProfile profile)
        {
            foreach (NecDspVariant candidate in Enum.GetValues<NecDspVariant>())
            {
                if (NecDspProfile.For(candidate).FirmwareName == profile.FirmwareName) return candidate;
            }
            return NecDspVariant.Dsp1B;
        }

        private static byte[]? TryRead(string path, int expectedSize)
        {
            try
            {
                if (!File.Exists(path)) return null;
                byte[] bytes = File.ReadAllBytes(path);
                return bytes.Length == expectedSize ? bytes : null;
            }
            catch (IOException)
            {
                return null;
            }
            catch (UnauthorizedAccessException)
            {
                return null;
            }
        }
    }
}
