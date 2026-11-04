using EmuSen.Crystal.Systems;

namespace EmuSen.WiseMan.Fixtures
{
    // An image in memory as a recogniser reads it, the first bytes asked for.
    public sealed class ImageProbe(byte[] bytes, string name = "image.iso") : IMediaProbe
    {
        public string Path => name;
        public string ImageName => name;
        public byte[] Read(int length) => bytes.Length <= length ? bytes : bytes[..length];
    }
}
