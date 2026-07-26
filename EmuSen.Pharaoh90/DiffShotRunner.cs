using EmuSen.Common.Imaging;

namespace EmuSen.Pharaoh90
{
    // The standalone `--diffshot <bmp1> <bmp2> <outpath>` mode - unrelated
    // to running a ROM at all, operates purely on two already-rendered BMP
    // files (screenshot/autoshot/contact sheet output from this same
    // toolkit, not arbitrary externally-authored images). Highlights every
    // differing pixel in magenta over a dimmed/grayed copy of the second
    // frame, so the change stands out at a glance instead of needing two
    // screenshots held side by side.
    public static class DiffShotRunner
    {
        public static int Run(string path1, string path2, string outPath)
        {
            var (rgbaA, widthA, heightA) = BmpFile.Read(path1);
            var (rgbaB, widthB, heightB) = BmpFile.Read(path2);
            if (widthA != widthB || heightA != heightB)
            {
                Console.WriteLine($"[ERROR] Size mismatch: {path1} is {widthA}x{heightA}, {path2} is {widthB}x{heightB}");
                return 1;
            }

            byte[] outRgba = new byte[rgbaA.Length];
            int changedCount = 0;
            int minX = widthA, minY = heightA, maxX = -1, maxY = -1;

            for (int y = 0; y < heightA; y++)
            {
                for (int x = 0; x < widthA; x++)
                {
                    int i = (y * widthA + x) * 4;
                    bool changed = rgbaA[i] != rgbaB[i] || rgbaA[i + 1] != rgbaB[i + 1] || rgbaA[i + 2] != rgbaB[i + 2];
                    if (changed)
                    {
                        changedCount++;
                        if (x < minX) minX = x;
                        if (x > maxX) maxX = x;
                        if (y < minY) minY = y;
                        if (y > maxY) maxY = y;
                        outRgba[i] = 255; outRgba[i + 1] = 0; outRgba[i + 2] = 255; outRgba[i + 3] = 255; // magenta
                    }
                    else
                    {
                        byte gray = (byte)((rgbaB[i] + rgbaB[i + 1] + rgbaB[i + 2]) / 3 / 2);
                        outRgba[i] = gray; outRgba[i + 1] = gray; outRgba[i + 2] = gray; outRgba[i + 3] = 255;
                    }
                }
            }

            BmpFile.Write(outPath, outRgba, widthA, heightA);

            int totalPixels = widthA * heightA;
            if (changedCount == 0)
            {
                Console.WriteLine($"[DIFFSHOT] No differences found ({widthA}x{heightA}, identical) -> {outPath}");
            }
            else
            {
                double pct = 100.0 * changedCount / totalPixels;
                Console.WriteLine($"[DIFFSHOT] {changedCount}/{totalPixels} pixels changed ({pct:F2}%), bounding box ({minX},{minY})-({maxX},{maxY}) -> {outPath}");
            }
            return 0;
        }
    }
}
