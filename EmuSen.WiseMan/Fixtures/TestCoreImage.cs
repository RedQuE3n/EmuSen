using System;
using System.Linq;

namespace EmuSen.WiseMan.Fixtures
{
    // The v1 test core's image: its magic, a little-endian length, the payload - see EmuSen_CoreAPI.md §7.
    public static class TestCoreImage
    {
        public static byte[] Of(params byte[] payload) => "V1TC"u8.ToArray().Concat(BitConverter.GetBytes((ushort)payload.Length)).Concat(payload).ToArray();
    }
}
