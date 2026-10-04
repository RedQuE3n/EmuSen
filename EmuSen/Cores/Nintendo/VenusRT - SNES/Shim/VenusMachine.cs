using EmuSen.Cores.Native;

namespace EmuSen.Cores.Nintendo.VenusRT
{
    // A VenusRT machine over the common interface, for the runners until stage 6's ICore shim - see VenusRT_Native.md §2.
    public sealed class VenusMachine : NativeMachine
    {
        // The spaces by id, named as C# Venus's debug target names them - see VenusRT_Plan.md §4.7.
        public static readonly string[] SpaceNames = { "CpuBus", "IO", "WRAM", "VRAM", "CGRAM", "OAM", "SRAM", "APURAM" };

        public static bool Available => VenusNative.Available;

        // File 0 is the battery save, file 1 the player's SPC700 boot image in place of the core's own (VenusRT_Native.md §38.4), file 2 a NEC DSP's firmware.
        public VenusMachine(ReadOnlySpan<byte> image, byte[]? battery = null, byte[]? dspFirmware = null, byte[]? boot = null)
            : base(VenusNative.Api, "VenusRT", "VenusRT", Own, OwnWords, image, "", Files(battery, dspFirmware, boot))
        {
        }

        private static (uint, byte[])[] Files(byte[]? battery, byte[]? dsp, byte[]? boot) =>
            new[] { (0u, battery), (1u, boot), (2u, dsp) }.Where(f => f.Item2 is not null).Select(f => (f.Item1, f.Item2!)).ToArray();

        private static Exception? Own(int status) => status switch
        {
            -9 => new InvalidDataException("Not an SNES image: shorter than one 32 KiB bank."),
            _ => null,
        };

        private static string? OwnWords(long status) => status switch
        {
            -9 => "an image shorter than one 32 KiB bank",
            _ => null,
        };

        public static int SpaceId(string name) => Array.FindIndex(SpaceNames, n => string.Equals(n, name, StringComparison.OrdinalIgnoreCase));

        public byte[] ReadSpace(string name)
        {
            int id = SpaceId(name);
            int size = id < 0 ? 0 : SpaceSize(id);
            var bytes = new byte[size];
            if (size > 0) ReadSpace(id, 0, bytes);
            return bytes;
        }
    }
}
