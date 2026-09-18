using System.Collections.Generic;

namespace EmuSen.Cores.Nintendo.Mars.Rom
{
    // The save chips a game's behaviour cannot reveal, by the header's two checksums - see Mars_Save.md §6.
    public static class SaveTypes
    {
        public readonly record struct Entry(N64SaveType Type, string Title);

        // The rows mupen64plus and Project64 both give as 16 Kbit EEPROM, and no others - see Mars_Save.md §6.
        public static readonly IReadOnlyDictionary<(uint Crc1, uint Crc2), Entry> Titles = new Dictionary<(uint, uint), Entry>
        {
            [(0xB695_1A94, 0x63C8_49AF)] = new(N64SaveType.Eeprom16k, "Akumajou Dracula Mokushiroku - Real Action Adventure (J) [!]"),
            [(0xA553_3106, 0xB9F2_5E5B)] = new(N64SaveType.Eeprom16k, "Akumajou Dracula Mokushiroku Gaiden - Legend of Cornell (J) [!]"),
            [(0x514B_6900, 0xB4B1_9881)] = new(N64SaveType.Eeprom16k, "Banjo to Kazooie no Daibouken 2 (J) [!]"),
            [(0x155B_7CDF, 0xF0DA_7325)] = new(N64SaveType.Eeprom16k, "Banjo-Tooie (A) [!]"),
            [(0xC917_6D39, 0xEA47_79D1)] = new(N64SaveType.Eeprom16k, "Banjo-Tooie (E) (M4) [!]"),
            [(0xC2E9_AA9A, 0x475D_70AA)] = new(N64SaveType.Eeprom16k, "Banjo-Tooie (U) [!]"),
            [(0xF800_9DB0, 0x6B29_1823)] = new(N64SaveType.Eeprom16k, "City-Tour GP - Zennihon GT Senshuken (J) [!]"),
            [(0x373F_5889, 0x9A6C_A80A)] = new(N64SaveType.Eeprom16k, "Conker's Bad Fur Day (E) [!]"),
            [(0x30C7_AC50, 0x7704_072D)] = new(N64SaveType.Eeprom16k, "Conker's Bad Fur Day (U) [!]"),
            [(0x83F3_931E, 0xCB72_223D)] = new(N64SaveType.Eeprom16k, "Cruis'n World (E) [!]"),
            [(0xDFE6_1153, 0xD761_18E6)] = new(N64SaveType.Eeprom16k, "Cruis'n World (U) [!]"),
            [(0x0795_01B9, 0xAB02_32AB)] = new(N64SaveType.Eeprom16k, "Custom Robo V2 (J) [!]"),
            [(0x17C5_4A61, 0x4A83_F2E7)] = new(N64SaveType.Eeprom16k, "Densha de Go! 64 (J) [!]"),
            [(0x1193_6D8C, 0x6F2C_4B43)] = new(N64SaveType.Eeprom16k, "Donkey Kong 64 (E) (M4) [!]"),
            [(0x053C_89A7, 0xA506_4302)] = new(N64SaveType.Eeprom16k, "Donkey Kong 64 (J) [!]"),
            [(0x0DD4_ABAB, 0xB5A2_A91E)] = new(N64SaveType.Eeprom16k, "Donkey Kong 64 (U) (Kiosk Demo) [!]"),
            [(0xEC58_EABF, 0xAD7C_7169)] = new(N64SaveType.Eeprom16k, "Donkey Kong 64 (U) [!]"),
            [(0xB630_6E99, 0xB63E_D2B2)] = new(N64SaveType.Eeprom16k, "Doraemon 2 - Nobita to Hikari no Shinden (J) [!]"),
            [(0xA827_5140, 0xB9B0_56E8)] = new(N64SaveType.Eeprom16k, "Doraemon 3 - Nobita no Machi SOS! (J) [!]"),
            [(0x202A_8EE4, 0x83F8_8B89)] = new(N64SaveType.Eeprom16k, "Excitebike 64 (E) [!]"),
            [(0x861C_3519, 0xF609_1CE5)] = new(N64SaveType.Eeprom16k, "Excitebike 64 (J) [!]"),
            [(0xAF75_4F7B, 0x1DD1_7381)] = new(N64SaveType.Eeprom16k, "Excitebike 64 (U) (Kiosk Demo) [!]"),
            [(0x0786_1842, 0xA12E_BC9F)] = new(N64SaveType.Eeprom16k, "Excitebike 64 (U) (V1.0) [!]"),
            [(0xF9D4_11E3, 0x7CB2_9BC0)] = new(N64SaveType.Eeprom16k, "Excitebike 64 (U) (V1.1) [!]"),
            [(0xEE4A_0E33, 0x8FD5_88C9)] = new(N64SaveType.Eeprom16k, "GT 64 - Championship Edition (E) (M3) [!]"),
            [(0xC49A_DCA2, 0xF150_1B62)] = new(N64SaveType.Eeprom16k, "GT 64 - Championship Edition (U) [!]"),
            [(0x0C58_1C7A, 0x3D6E_20E4)] = new(N64SaveType.Eeprom16k, "Hoshi no Kirby 64 (J) (V1.2) [!]"),
            [(0xBCB1_F89F, 0x0607_52A2)] = new(N64SaveType.Eeprom16k, "Hoshi no Kirby 64 (J) (V1.3) [!]"),
            [(0x77DA_3B8D, 0x162B_0D7C)] = new(N64SaveType.Eeprom16k, "Ide Yousuke no Mahjong Juku (J) [!]"),
            [(0x0D93_BA11, 0x6838_68A6)] = new(N64SaveType.Eeprom16k, "Kirby 64 - The Crystal Shards (E) [!]"),
            [(0x4603_9FB4, 0x0337_822C)] = new(N64SaveType.Eeprom16k, "Kirby 64 - The Crystal Shards (U) [!]"),
            [(0x1739_EFBA, 0xD0B4_3A68)] = new(N64SaveType.Eeprom16k, "Kobe Bryant in NBA Courtside (E) [!]"),
            [(0xC567_4160, 0x0F5F_453C)] = new(N64SaveType.Eeprom16k, "Mario Party 3 (E) (M4) [!]"),
            [(0x0B0A_B4CD, 0x7B15_8937)] = new(N64SaveType.Eeprom16k, "Mario Party 3 (J) [!]"),
            [(0x7C38_29D9, 0x6E82_47CE)] = new(N64SaveType.Eeprom16k, "Mario Party 3 (U) [!]"),
            [(0x839F_3AD5, 0x406D_15FA)] = new(N64SaveType.Eeprom16k, "Mario Tennis (E) [!]"),
            [(0x5001_CF4F, 0xF30C_B3BD)] = new(N64SaveType.Eeprom16k, "Mario Tennis (U) [!]"),
            [(0x3A6C_42B5, 0x1ACA_DA1B)] = new(N64SaveType.Eeprom16k, "Mario Tennis 64 (J) [!]"),
            [(0x147E_0EDB, 0x36C5_B12C)] = new(N64SaveType.Eeprom16k, "Neon Genesis Evangelion (J) [!]"),
            [(0xF468_118C, 0xE32E_E44E)] = new(N64SaveType.Eeprom16k, "PD Ultraman Battle Collection 64 (J) [!]"),
            [(0xCFE2_CB31, 0x4D6B_1E1D)] = new(N64SaveType.Eeprom16k, "Parlor! Pro 64 - Pachinko Jikki Simulation Game (J) [!]"),
            [(0xE4B0_8007, 0xA602_FF33)] = new(N64SaveType.Eeprom16k, "Perfect Dark (E) (M5) [!]"),
            [(0x9674_7EB4, 0x104B_B243)] = new(N64SaveType.Eeprom16k, "Perfect Dark (J) [!]"),
            [(0xDDF4_60CC, 0x3CA6_34C0)] = new(N64SaveType.Eeprom16k, "Perfect Dark (U) (V1.0) [!]"),
            [(0x41F2_B98F, 0xB458_B466)] = new(N64SaveType.Eeprom16k, "Perfect Dark (U) (V1.1) [!]"),
            [(0xFEE9_7010, 0x4E94_A9A0)] = new(N64SaveType.Eeprom16k, "RR64 - Ridge Racer 64 (E) [!]"),
            [(0x2500_267E, 0x2A7E_C3CE)] = new(N64SaveType.Eeprom16k, "RR64 - Ridge Racer 64 (U) [!]"),
            [(0x272B_690F, 0xAD0A_7A77)] = new(N64SaveType.Eeprom16k, "Robot Ponkottsu 64 - 7tsu no Umi no Caramel (J) [!]"),
            [(0x53ED_2DC4, 0x0625_8002)] = new(N64SaveType.Eeprom16k, "Star Wars Episode I - Racer (E) (M3) [!]"),
            [(0x61F5_B152, 0x0461_22AB)] = new(N64SaveType.Eeprom16k, "Star Wars Episode I - Racer (J) [!]"),
            [(0x72F7_0398, 0x6556_A98B)] = new(N64SaveType.Eeprom16k, "Star Wars Episode I - Racer (U) [!]"),
            [(0x2DCF_CA60, 0x8354_B147)] = new(N64SaveType.Eeprom16k, "Yoshi Story (J) [!]"),
            [(0xD3F9_7D49, 0x6924_135B)] = new(N64SaveType.Eeprom16k, "Yoshi's Story (E) (M3) [!]"),
            [(0x2337_D8E8, 0x6B8E_7CEC)] = new(N64SaveType.Eeprom16k, "Yoshi's Story (U) (M2) [!]"),
        };

        // The image's own word first, then the table; anything else is left for the game to show - see Mars_Save.md §1.
        public static N64SaveType Declared(RomImage rom)
        {
            if (rom.SaveType != N64SaveType.Unknown) return rom.SaveType;

            return Titles.TryGetValue((rom.Crc1, rom.Crc2), out Entry entry) ? entry.Type : N64SaveType.Unknown;
        }
    }
}
