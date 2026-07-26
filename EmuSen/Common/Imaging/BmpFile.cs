namespace EmuSen.Common.Imaging
{
    // Minimal uncompressed 32bpp BMP reader/writer - no case for PNG's
    // DEFLATE needed just to look at a frame. BMP rows are stored bottom-up
    // and BGRA rather than RGBA, both handled by walking rgba backwards a
    // row at a time and swapping R/B per pixel; everything else about the
    // format is a fixed-size header. Moved out of EmuSen.Pharaoh90's
    // Program.cs - general-purpose, not specific to that one CLI tool.
    public static class BmpFile
    {
        public static void Write(string path, byte[] rgba, int width, int height)
        {
            int rowSize = width * 4;
            int imageSize = rowSize * height;
            int fileSize = 54 + imageSize;

            using var fs = new FileStream(path, FileMode.Create);
            using var w = new BinaryWriter(fs);

            w.Write((byte)'B'); w.Write((byte)'M');
            w.Write(fileSize);
            w.Write(0); // reserved
            w.Write(54); // pixel data offset

            w.Write(40); // DIB header size
            w.Write(width);
            w.Write(height);
            w.Write((short)1); // planes
            w.Write((short)32); // bits per pixel
            w.Write(0); // no compression
            w.Write(imageSize);
            w.Write(2835); w.Write(2835); // ~72 DPI
            w.Write(0); w.Write(0); // colors used/important

            for (int y = height - 1; y >= 0; y--)
            {
                int rowStart = y * rowSize;
                for (int x = 0; x < width; x++)
                {
                    int i = rowStart + x * 4;
                    w.Write(rgba[i + 2]); // B
                    w.Write(rgba[i + 1]); // G
                    w.Write(rgba[i + 0]); // R
                    w.Write(rgba[i + 3]); // A
                }
            }
        }

        // Reads back exactly what Write writes - the fixed 54-byte header,
        // bottom-up BGRA rows - since callers only ever need to read this
        // toolkit's own screenshots/autoshots/contact sheets back, not
        // arbitrary externally-authored BMPs.
        public static (byte[] Rgba, int Width, int Height) Read(string path)
        {
            using var fs = new FileStream(path, FileMode.Open, FileAccess.Read);
            using var r = new BinaryReader(fs);

            fs.Position = 10;
            int dataOffset = r.ReadInt32();
            fs.Position = 18;
            int width = r.ReadInt32();
            int height = r.ReadInt32();

            fs.Position = dataOffset;
            int rowSize = width * 4;
            byte[] rgba = new byte[rowSize * height];
            for (int y = height - 1; y >= 0; y--)
            {
                int rowStart = y * rowSize;
                for (int x = 0; x < width; x++)
                {
                    int i = rowStart + x * 4;
                    rgba[i + 2] = r.ReadByte(); // B
                    rgba[i + 1] = r.ReadByte(); // G
                    rgba[i + 0] = r.ReadByte(); // R
                    rgba[i + 3] = r.ReadByte(); // A
                }
            }
            return (rgba, width, height);
        }
    }
}
