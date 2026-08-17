namespace EmuSen.Cores.Nintendo.Venus.Coprocessors.NecDsp
{
    // Which NEC DSP, if any, a cartridge header describes - see Venus_NecDSP.md §1.
    public static class NecDspDetection
    {
        // Cartridge type high nibble $0 is a DSP, $F with chip subtype $01 an ST01x.
        public static NecDspVariant? Detect(byte cartType, byte chipType, string cartName)
        {
            if ((cartType & 0x0F) < 0x03) return null;

            if ((cartType & 0xF0) == 0xF0 && chipType == 0x01)
            {
                return cartName == "2DAN MORITA SHOUGI" ? NecDspVariant.St011 : NecDspVariant.St010;
            }

            if ((cartType & 0xF0) != 0x00) return null;

            return cartName switch
            {
                "DUNGEON MASTER" => NecDspVariant.Dsp2,
                "PILOTWINGS" => NecDspVariant.Dsp1,
                "SD\xB6\xDE\xDD\xC0\xDE\xD1GX" => NecDspVariant.Dsp3,
                "PLANETS CHAMP TG3000" or "TOP GEAR 3000" => NecDspVariant.Dsp4,

                // Every other DSP cartridge shipped the revised DSP-1B.
                _ => NecDspVariant.Dsp1B,
            };
        }
    }
}
