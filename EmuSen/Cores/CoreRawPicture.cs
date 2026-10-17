namespace EmuSen.Cores
{
    // A frame's picture before its core's palette, in a named encoding: Height rows of Stride bytes, Width values of Element bytes, little-endian - see EmuSen_CoreAPI.md §6.30.
    public readonly record struct CoreRawPicture(string Encoding, int Width, int Height, int Element, int Stride, byte[] Values);
}
