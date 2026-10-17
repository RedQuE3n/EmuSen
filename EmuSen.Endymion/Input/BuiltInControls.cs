namespace EmuSen.Endymion.Input
{
    // The controllers built into a handheld, by the USB vendor and product SDL reports; each entry's evidence is EmuSen_Input.md §8.13.2.
    public static class BuiltInControls
    {
        private static readonly (ushort Vendor, ushort Product, string Device)[] Known =
        {
            (0x28DE, 0x1205, "Steam Deck"),
            (0x1A86, 0xE310, "Legion Go S"),
            (0x28DE, 0x12FF, "Legion Go S"),
        };

        public static bool Contains(ushort vendor, ushort product)
        {
            foreach (var known in Known)
                if (known.Vendor == vendor && known.Product == product) return true;
            return false;
        }
    }
}
