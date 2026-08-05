using System;
using System.Globalization;
using EmuSen.Cores.Nintendo.Moon.Cheats;

namespace EmuSen.WiseMan.Cores
{
    // Every case here was produced by Mesen's own ConvertFromNesGameGenie,
    // transcribed and run as an oracle - see Moon_Cheats.md §2. Regenerating
    // them by hand from this implementation would prove nothing.
    public class NesGameGenieCodecTests
    {
        // "<code> <address> <value> <compare|->"
        private static readonly string[] MesenCases =
        {
            "SXIOPO 91D9 AD -",
            "AEUZLZ A2B3 00 -",
            "GXXZZE A0A2 2C -",
            "OXNGZX C2F2 A9 -",
            "PEUZUGAA ACB3 01 00",
            "SLXPLOVS 9123 BD DE",
            "AAAAAA 8000 00 -",
            "NNNNNNNN FFFF FF FF",
            "ZEXPYGLA 94A7 02 03",
            "XGKPZL 9342 C2 -",
            "UPTPZS 9562 9B -",
            "SZYZSP A975 A5 -",
            "LYPKPY C719 73 -",
            "PGOSGL D31C 41 -",
            "OILTUL EB33 D1 -",
            "ZPTNSX FA6D 1A -",
            "VVUOYI 95BF E6 -",
            "YZONXV FE1A 2F -",
            "OZLSIX D23D A9 -",
            "GNSPZX 92D2 7C -",
            "XUNVZZ E2FA B2 -",
            "ENZPOV 9EA1 F8 -",
            "OKUAVU 8BB6 C9 -",
            "ILNPTO 9176 3D -",
            "GYKKNZ CA4F 74 -",
            "IVKEGS 85CC 6D -",
            "ESUKYG C4BF D0 -",
            "ZIGYYA F047 52 -",
            "NIEOAG 9408 D7 -",
            "SUXGPV C6A1 BD -",
            "KKKKLN C7CB CC -",
            "KPTZTV A666 9C -",
            "ILXPLA 9023 35 -",
            "GLUAZT 8632 34 -",
            "KGEUUN BF0B CC -",
            "LLNVNN EF7F 3B -",
            "OZGLXE B842 A9 -",
            "NIATUG EC03 D7 -",
            "AOZEUI 8DAB 10 -",
            "UYXYTY F726 F3 -",
            "KYTNUA F86B F4 -",
            "AENETU 83FE 08 -",
            "VUUZYL A3B7 B6 -",
            "YNTXTN A7EE 7F -",
            "ANUZLK A4B3 78 -",
            "TNISXZ DADA 76 -",
            "KVKZII A5C5 E4 -",
            "GAGVGN E74C 0C -",
            "UGGAAL 8340 C3 -",
            "GSTTAETO E0E0 5C 1E",
            "YXESGPUV D18C 2F E3",
            "SGGAVIAG 8D46 C5 40",
            "IGNLPXNL B271 45 BF",
            "PYTEPLVA 8369 71 86",
            "ZVXTEVNY EEA0 62 FF",
            "ETVGSLKV CB65 E8 E4",
            "XZYSZTOL D67A A2 B1",
            "GUGEGVYL 86CC 34 3F",
            "KNIYISKX F5D5 FC AC",
            "STUXZUAX A33A ED 28",
            "VVAKXOZL C98A E6 3A",
            "YLZEEPIE 8928 3F 05",
            "GSEKGNXZ C78C 54 AA",
            "EPISZEAZ D05A 90 28",
            "EZYZELVA AB70 A0 86",
            "XSEGPYLI C781 D2 53",
            "EPITOOTO E951 98 1E",
            "VIEUAEPA B008 D6 09",
            "ATNYVLSN FB76 68 F5",
            "KOTYXTGK FEE2 9C 44",
            "UPGAZESI 8042 93 DD",
            "PZKOYOPV 914F 29 69",
            "IIEVAEUX E008 5D AB",
            "XYPOTUIA 931E F2 0D",
            "XKZNETYA FEA8 C2 07",
            "ZEZGKPKA C9A4 02 84",
            "OOYZGKXN A4F4 99 FA",
            "GOGPSGAY 9CC5 14 70",
            "ZAPGULKV CB13 0A E4",
            "PAYNEAVZ F878 01 A6",
            "ZZNEZEYT 807A 22 6F",
            "YVNKZNOP C7FA 67 99",
            "TZGXEOGA A948 26 0C",
            "NPNELTNO 867B 9F 97",
            "OVVVLTOZ E6EB E1 A1",
            "NAOVZVEK E61A 8F C8",
            "TTZZGEUG A024 66 CB",
            "ELUYNNKA FF37 B0 8C",
            "IANVKOGS E97C 0D 5C",
        };

        [Fact]
        public void Every_code_decodes_the_way_Mesen_decodes_it()
        {
            foreach (string line in MesenCases)
            {
                string[] parts = line.Split(' ');
                (int address, byte value, byte? compare) = NesGameGenieCodec.DecodeFull(parts[0]);

                Assert.Equal(int.Parse(parts[1], NumberStyles.HexNumber), address);
                Assert.Equal(byte.Parse(parts[2], NumberStyles.HexNumber), value);
                Assert.Equal(
                    parts[3] == "-" ? (byte?)null : byte.Parse(parts[3], NumberStyles.HexNumber),
                    compare);
            }
        }
    }
}
