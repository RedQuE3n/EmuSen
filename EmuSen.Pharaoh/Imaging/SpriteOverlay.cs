namespace EmuSen.Pharaoh.Imaging
{
    // A rectangle's outline, not filled, so the sprite's own pixels stay visible inside it.
    public static class SpriteOverlay
    {
        public static void DrawSpriteOutline(byte[] rgba, int width, int height, int spriteX, int spriteY, int spriteWidth, int spriteHeight)
        {
            void SetPixel(int x, int y)
            {
                if (x < 0 || x >= width || y < 0 || y >= height) return;
                int i = (y * width + x) * 4;
                rgba[i] = 0; rgba[i + 1] = 255; rgba[i + 2] = 0; rgba[i + 3] = 255;
            }

            for (int x = spriteX; x < spriteX + spriteWidth; x++)
            {
                SetPixel(x, spriteY);
                SetPixel(x, spriteY + spriteHeight - 1);
            }
            for (int y = spriteY; y < spriteY + spriteHeight; y++)
            {
                SetPixel(spriteX, y);
                SetPixel(spriteX + spriteWidth - 1, y);
            }
        }
    }
}
