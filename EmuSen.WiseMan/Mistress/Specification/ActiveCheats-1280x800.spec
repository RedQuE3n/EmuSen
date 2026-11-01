# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheats-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,96,812.7,582
title "Active Cheats"
case ActiveCheats
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,196,813,462 by=panel limit=234,196,812.35,462 paints=#D4D4D4FF role=none seen=234,196,812.35,462
    1 DockPanel at=258,208,765,434 by=panel limit=234,196,812.35,462 role=none
      2 Tabs at=258,208,765,217 by=panel limit=234,196,812.35,462 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=258,208,112,48 by=clip drawn focus heard=GENERAL limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=General
          4 TextBlock at=270,214,88,36 box=88,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=87.63,36 limit=258,208,112,48 owner=3 paints=#8D8D91FF role=text size=30 text=GENERAL
        5 TabItem at=370,208,62,48 by=clip drawn focus heard=NES limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=382,214,38,36 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=37.71,36 limit=370,208,62,48 owner=5 paints=#8D8D91FF role=text size=30 text=NES
        7 TabItem at=432,208,50,48 by=clip drawn focus heard=GB limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=444,214,26,36 box=26,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=25.77,36 limit=432,208,50,48 owner=7 paints=#8D8D91FF role=text size=30 text=GB
        9 TabItem at=482,208,74,48 by=clip drawn focus heard=SNES limit=258,208,765,217 op paints=#EEEEF0FF role=tabitem sideways text=SNES
          10 TextBlock at=494,214,50,36 box=50,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=49.53,36 limit=482,208,74,48 owner=9 paints=#EEEEF0FF role=text size=30 text=SNES
        11 TabItem at=556,208,62,48 by=clip drawn focus heard=N64 limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=N64
          12 TextBlock at=568,214,38,36 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=37.98,36 limit=556,208,62,48 owner=11 paints=#8D8D91FF role=text size=30 text=N64
        13 TabItem at=618,208,104,48 by=clip drawn focus heard=SG-1000 limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=630,214,80,36 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=79.8,36 limit=618,208,104,48 owner=13 paints=#8D8D91FF role=text size=30 text=SG-1000
        15 TabItem at=722,208,178,48 by=clip drawn focus heard="MASTER SYSTEM" limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=734,214,154,36 box=154,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=153.69,36 limit=722,208,178,48 owner=15 paints=#8D8D91FF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=900,208,104,48 by=clip drawn focus heard=GENESIS limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=912,214,80,36 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=79.89,36 limit=900,208,104,48 owner=17 paints=#8D8D91FF role=text size=30 text=GENESIS
        19 TabItem at=258,256,105,48 by=clip drawn focus heard="SEGA CD" limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=270,262,81,36 box=81,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=80.43,36 limit=258,256,105,48 owner=19 paints=#8D8D91FF role=text size=30 text="SEGA CD"
        21 TabItem at=363,256,147,48 by=clip drawn focus heard=PLAYSTATION limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          22 TextBlock at=375,262,123,36 box=123,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=122.67,36 limit=363,256,147,48 owner=21 paints=#8D8D91FF role=text size=30 text=PLAYSTATION
        23 TabItem at=510,256,165,48 by=clip drawn focus heard="PLAYSTATION 2" limit=258,208,765,217 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          24 TextBlock at=522,262,141,36 box=141,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=140.19,36 limit=510,256,165,48 owner=23 paints=#8D8D91FF role=text size=30 text="PLAYSTATION 2"
        25 StackPanel at=270,314,741,111 by=clip limit=258,306,765,119 role=none
          26 TextBlock at=270,314,741,29 box=741,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a SNES cheat" layout=137.18,28.8 limit=258,306,765,119 paints=#D4D4D4FF role=text size=24 text="Add a SNES cheat"
          27 HintText at=270,349,741,24 box=741,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Accepts: Game Genie, Pro Action Replay." layout=260.66,24 limit=258,306,765,119 paints=#8D8D91FF role=text size=20 text="Accepts: Game Genie, Pro Action Replay."
          28 Grid at=270,379,741,46 by=clip limit=258,306,765,119 role=none
            29 TextBox at=270,379,333,46 by=clip drawn focus heard="Cheat code" limit=258,306,765,119 name=SNESCodeBox op paints=#0E0E10FF,#26262BFF,#4C9AE8FF,#707070FF,#EEEEF0FF role=edit
              30 ScrollViewer allow="a text box scrolls its own text" at=281,386,311,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                31 TextBlock allow="a text box scrolls its own text" at=281,386,311,32 box=311,32 caps=18.1987 drawn face="Barlow Condensed" heard=Code in=30 layout=42.04,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text=Code
            32 TextBox at=611,379,324,46 by=clip drawn focus heard="Cheat description" limit=258,306,765,119 name=SNESDescriptionBox op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              33 ScrollViewer allow="a text box scrolls its own text" at=622,386,302,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                34 TextBlock allow="a text box scrolls its own text" at=622,386,302,32 box=302,32 caps=18.1987 drawn face="Barlow Condensed" heard=Description in=33 layout=94.67,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text=Description
            35 Button at=943,380,68,44 by=clip drawn focus heard=ADD limit=258,306,765,119 name=SNESAddButton op paints=#707070FF,#D4D4D4FF role=button text=Add
              36 TextBlock at=960,386,34,32 box=34,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ADD layout=33.51,31.2 limit=943,380,68,44 owner=35 paints=#D4D4D4FF role=text size=26 text=ADD
      37 TextBlock at=258,425,765,29 box=765,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 cheat(s), 2 on" layout=117.6,28.8 limit=234,196,812.35,462 name=ListHeaderText paints=#D4D4D4FF role=text size=24 text="2 cheat(s), 2 on"
      38 Border at=258,598,765,44 by=panel limit=234,196,812.35,462 role=none
        39 Grid at=258,598,765,44 by=panel limit=234,196,812.35,462 role=none
          40 ButtonBar at=297,598,687,44 by=panel limit=234,196,812.35,462 paints=#D4D4D4FF role=toolbar
            41 Button at=297,598,189,44 by=clip drawn focus heard="CHEAT DATABASE..." limit=297,598,687,44 name=DatabaseButton op paints=#707070FF,#D4D4D4FF role=button text="Cheat Database..."
              42 TextBlock at=314,604,155,32 box=155,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=154.41,31.2 limit=297,598,189,44 owner=41 paints=#D4D4D4FF role=text size=26 text="CHEAT DATABASE..."
            43 Button at=494,598,153,44 by=clip drawn focus heard="APPLY CHEATS" limit=297,598,687,44 name=ApplyButton op paints=#707070FF,#D4D4D4FF role=button text="Apply Cheats"
              44 TextBlock at=511,604,119,32 box=119,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=118.22,31.2 limit=494,598,153,44 owner=43 paints=#D4D4D4FF role=text size=26 text="APPLY CHEATS"
            45 Button at=655,598,75,44 by=clip drawn focus heard=SAVE limit=297,598,687,44 name=SaveButton op paints=#707070FF,#D4D4D4FF role=button text=Save
              46 TextBlock at=672,604,41,32 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=40.87,31.2 limit=655,598,75,44 owner=45 paints=#D4D4D4FF role=text size=26 text=SAVE
            47 Button at=738,598,101,44 by=clip drawn heard=REMOVE limit=297,598,687,44 name=RemoveButton off paints=#707070FF,#D4D4D4FF role=button text=Remove
              48 TextBlock at=755,604,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=738,598,101,44 off owner=47 paints=#707070FF role=text size=26 text=REMOVE
            49 Button at=847,598,137,44 by=clip drawn focus heard="REMOVE ALL" limit=297,598,687,44 name=ClearButton op paints=#707070FF,#D4D4D4FF role=button text="Remove All"
              50 TextBlock at=864,604,103,32 box=103,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=102.49,31.2 limit=847,598,137,44 owner=49 paints=#D4D4D4FF role=text size=26 text="REMOVE ALL"
      51 LunaTable at=258,462,765,124 by=panel heard="Cheats for this console" limit=234,196,812.35,462 name=CheatsList paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=270,466,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=20.21,28.8 paints=#D4D4D4FF role=text size=24 text=On
        53 GridSplitter at=304,466,6,29 by=clip focus heard="Resize On" limit=258,462,765,40 paints=#D4D4D4FF role=thumb
        54 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=310,466,44,29 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=34.25,28.8 paints=#D4D4D4FF role=text size=24 text=Kind
        55 GridSplitter at=348,466,6,29 by=clip focus heard="Resize Kind" limit=258,462,765,40 paints=#D4D4D4FF role=thumb
        56 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=354,466,393,29 box=393,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=44.11,28.8 paints=#D4D4D4FF role=text size=24 text=Cheat
        57 GridSplitter at=741,466,6,29 by=clip focus heard="Resize Cheat" limit=258,462,765,40 paints=#D4D4D4FF role=thumb
        58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=747,466,264,29 box=264,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=38.81,28.8 paints=#D4D4D4FF role=text size=24 text=Code
        59 ScrollViewer at=258,502,765,84 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=258,502,765,84
          60 ListBoxItem at=258,502,765,42 by=fixed-scroll drawn focus heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=59 limit=258,502,765,84 op paints=#333337FF,#D4D4D4FF role=listitem
            61 Grid at=268,506,745,32 by=clip heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=59 limit=258,502,765,42 role=none
              62 CheckBox at=268,506,20,32 by=clip drawn focus heard=On in=59 limit=258,502,765,42 paints=#4C9AE8FF,#D4D4D4FF role=checkbox
                63 ViewboxContainer at=268,512,20,20 by=clip in=59 limit=268,512,20,20 role=none
              64 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=308,510,44,24 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=59 layout=26.82,24 owner=60 paints=#8D8D91FF role=text size=20 text=RAM
              65 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=352,508,397,29 box=397,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat in=59 layout=94.15,28.8 owner=60 paints=#D4D4D4FF role=edit size=24 text="Infinite lives"
              66 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=749,508,264,28 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0100 = 0x42" in=59 layout=264,27.24 owner=60 paints=#8D8D91FF role=text size=20 text="CpuBus 0x7E0100 = 0x42"
          67 ListBoxItem at=258,544,765,42 by=fixed-scroll drawn focus heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=59 limit=258,502,765,84 op paints=#333337FF,#D4D4D4FF role=listitem
            68 Grid at=268,548,745,32 by=clip heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=59 limit=258,544,765,42 role=none
              69 CheckBox at=268,548,20,32 by=clip drawn focus heard=On in=59 limit=258,544,765,42 paints=#4C9AE8FF,#D4D4D4FF role=checkbox
                70 ViewboxContainer at=268,554,20,20 by=clip in=59 limit=268,554,20,20 role=none
              71 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=308,552,44,24 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=59 layout=26.82,24 owner=67 paints=#8D8D91FF role=text size=20 text=RAM
              72 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=352,550,397,29 box=397,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Cheat in=59 layout=394.01,28.8 owner=67 paints=#D4D4D4FF role=edit size=24 text="Nine continues, and a description long enough to need the whole row or more of it"
              73 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=749,550,264,28 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0101 = 0x09" in=59 layout=264,27.24 owner=67 paints=#8D8D91FF role=text size=20 text="CpuBus 0x7E0101 = 0x09"
focus 29
edge 9 up same
edge 9 down 29
edge 29 up 9
edge 29 down 53
edge 29 left same
edge 29 right 32
edge 32 up 9
edge 32 down 57
edge 32 left 29
edge 32 right 35
edge 35 up 9
edge 35 down 60
edge 35 left 32
edge 35 right same
edge 41 up 67
edge 41 down same
edge 41 left 69
edge 41 right 43
edge 43 up 67
edge 43 down same
edge 43 left 41
edge 43 right 45
edge 45 up 67
edge 45 down same
edge 45 left 43
edge 45 right 49
edge 49 up 67
edge 49 down same
edge 49 left 45
edge 49 right same
edge 53 up 29
edge 53 down 60
edge 53 left 62
edge 53 right 55
edge 55 up 29
edge 55 down 60
edge 55 left 53
edge 55 right 57
edge 57 up 32
edge 57 down 60
edge 57 left 55
edge 57 right 49
edge 60 up 53
edge 60 down 67
edge 60 left same
edge 60 right same
edge 62 up 29
edge 62 down 67
edge 62 left same
edge 62 right 41
edge 67 up 60
edge 67 down 41
edge 67 left same
edge 67 right same
edge 69 up 60
edge 69 down 41
edge 69 left same
edge 69 right 41
press pageup focus="TextBox GBCodeBox (Cheat code)" tab=GB title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press first focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press last focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press back top=none
