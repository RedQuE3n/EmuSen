namespace EmuSen.Cores.Nintendo.Mars.Rdp
{
    // Chroma key: the last combiner cycle measures how near its colour is to the key and passes its first input through - see Mars_RdpChromaKey.md.
    public sealed partial class Rdp
    {
        private Color _keyWidth;

        // How far each channel is inside its key's width, in sixteenths, and the narrowest of the three - see §2.
        private int ChromaKey(int red, int green, int blue)
        {
            int distance = System.Math.Min(Distance(red, _keyWidth.R), System.Math.Min(Distance(green, _keyWidth.G), Distance(blue, _keyWidth.B)));
            return System.Math.Clamp(distance, 0, 0xFF);
        }

        // A channel past the key's centre counts its distance back, and a distance whose low nibble is eight rounds the other way - see §2.
        private static int Distance(int channel, int width)
        {
            int value = (channel << 15) >> 15;
            if (value > 0) value = (value & 0xF) == 8 ? 0x10 - value : -value;

            return (width << 4) + value;
        }
    }
}
