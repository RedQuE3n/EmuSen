# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheats-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,144,1219.05,873
title "Active Cheats"
case ActiveCheats
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,294,1221,693 by=panel limit=350.48,294,1219.05,693 paints=#9C9CA0FF role=none seen=350.48,294,1219.05,693
    1 DockPanel at=386,312,1149,651 by=panel limit=350.48,294,1219.05,693 role=none
      2 Tabs at=386,312,1149,325.5 by=panel limit=350.48,294,1219.05,693 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=386,312,174,72 by=clip drawn focus heard=GENERAL limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=General
          4 TextBlock at=404,321,138,54 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=386,312,174,72 owner=3 paints=#86868AFF role=text size=30 text=GENERAL
        5 TabItem at=560,312,97.5,72 by=clip drawn focus heard=NES limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=578,321,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=560,312,97.5,72 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=657.5,312,76.5,72 by=clip drawn focus heard=GB limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=675.5,321,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=657.5,312,76.5,72 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=734,312,115.5,72 by=clip drawn focus heard=SNES limit=386,312,1149,325.5 op paints=#EEEEF0FF role=tabitem sideways text=SNES
          10 TextBlock at=752,321,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=734,312,115.5,72 owner=9 paints=#EEEEF0FF role=text size=30 text=SNES
        11 TabItem at=849.5,312,96,72 by=clip drawn focus heard=N64 limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=867.5,321,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=849.5,312,96,72 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=945.5,312,160.5,72 by=clip drawn focus heard=SG-1000 limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=SG-1000
          14 TextBlock at=963.5,321,124.5,54 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=945.5,312,160.5,72 owner=13 paints=#86868AFF role=text size=30 text=SG-1000
        15 TabItem at=1106,312,279,72 by=clip drawn focus heard="MASTER SYSTEM" limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text="Master System"
          16 TextBlock at=1124,321,243,54 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=1106,312,279,72 owner=15 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=386,384,165,72 by=clip drawn focus heard=GENESIS limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=404,393,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=386,384,165,72 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=551,384,163.5,72 by=clip drawn focus heard="SEGA CD" limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=569,393,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=551,384,163.5,72 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=714.5,384,234,72 by=clip drawn focus heard=PLAYSTATION limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=732.5,393,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=714.5,384,234,72 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 TabItem at=948.5,384,261,72 by=clip drawn focus heard="PLAYSTATION 2" limit=386,312,1149,325.5 paints=#86868AFF role=tabitem sideways text="PlayStation 2"
          24 TextBlock at=966.5,393,225,54 box=150,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=149.97,36 limit=948.5,384,261,72 owner=23 paints=#86868AFF role=text size=30 text="PLAYSTATION 2"
        25 StackPanel at=404,471,1113,166.5 by=clip limit=386,459,1149,178.5 role=none
          26 TextBlock at=404,471,1113,43.5 box=742,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a SNES cheat" layout=142.18,28.8 limit=386,459,1149,178.5 paints=#9C9CA0FF role=text size=24 text="Add a SNES cheat"
          27 HintText at=404,523.5,1113,36 box=742,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Accepts: Game Genie, Pro Action Replay." layout=271.12,24 limit=386,459,1149,178.5 paints=#86868AFF role=text size=20 text="Accepts: Game Genie, Pro Action Replay."
          28 Grid at=404,568.5,1113,69 by=clip limit=386,459,1149,178.5 role=none
            29 TextBox at=404,568.5,499.5,69 by=clip drawn focus heard="Cheat code" limit=386,459,1149,178.5 name=SNESCodeBox op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              30 ScrollViewer allow="a text box scrolls its own text" at=420.5,579,466.5,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                31 TextBlock allow="a text box scrolls its own text" at=420.5,579,466.5,48 box=311,32 caps=18.1987 drawn face="Barlow Condensed" heard=Code in=30 layout=43.65,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Code
            32 TextBox at=915.5,568.5,486,69 by=clip drawn focus heard="Cheat description" limit=386,459,1149,178.5 name=SNESDescriptionBox op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              33 ScrollViewer allow="a text box scrolls its own text" at=932,579,453,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                34 TextBlock allow="a text box scrolls its own text" at=932,579,453,48 box=302,32 caps=18.1987 drawn face="Barlow Condensed" heard=Description in=33 layout=101.09,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Description
            35 Button at=1413.5,570,103.5,66 by=clip drawn focus heard=ADD limit=386,459,1149,178.5 name=SNESAddButton op paints=#55555AFF,#9C9CA0FF role=button text=Add
              36 TextBlock at=1439,579,52.5,48 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ADD layout=34.66,31.2 limit=1413.5,570,103.5,66 owner=35 paints=#9C9CA0FF role=text size=26 text=ADD
      37 TextBlock at=386,637.5,1149,43.5 box=766,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 cheat(s), 2 on" layout=123.53,28.8 limit=350.48,294,1219.05,693 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="2 cheat(s), 2 on"
      38 Border at=386,897,1149,66 by=panel limit=350.48,294,1219.05,693 role=none
        39 Grid at=386,897,1149,66 by=panel limit=350.48,294,1219.05,693 role=none
          40 ButtonBar at=428,897,1066.5,66 by=panel limit=350.48,294,1219.05,693 paints=#9C9CA0FF role=toolbar
            41 Button at=428,897,292.5,66 by=clip drawn focus heard="CHEAT DATABASE..." limit=428,897,1066.5,66 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              42 TextBlock at=453.5,906,241.5,48 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=428,897,292.5,66 owner=41 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            43 Button at=732.5,897,240,66 by=clip drawn focus heard="APPLY CHEATS" limit=428,897,1066.5,66 name=ApplyButton op paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              44 TextBlock at=758,906,189,48 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=732.5,897,240,66 owner=43 paints=#9C9CA0FF role=text size=26 text="APPLY CHEATS"
            45 Button at=984.5,897,115.5,66 by=clip drawn focus heard=SAVE limit=428,897,1066.5,66 name=SaveButton op paints=#55555AFF,#9C9CA0FF role=button text=Save
              46 TextBlock at=1010,906,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=984.5,897,115.5,66 owner=45 paints=#9C9CA0FF role=text size=26 text=SAVE
            47 Button at=1112,897,157.5,66 by=clip drawn heard=REMOVE limit=428,897,1066.5,66 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              48 TextBlock at=1137.5,906,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=1112,897,157.5,66 off owner=47 paints=#55555AFF role=text size=26 text=REMOVE
            49 Button at=1281.5,897,213,66 by=clip drawn focus heard="REMOVE ALL" limit=428,897,1066.5,66 name=ClearButton op paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              50 TextBlock at=1307,906,162,48 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=1281.5,897,213,66 owner=49 paints=#9C9CA0FF role=text size=26 text="REMOVE ALL"
      51 LunaTable at=386,693,1149,186 by=panel heard="Cheats for this console" limit=350.48,294,1219.05,693 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=404,699,60,43.5 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        53 GridSplitter at=455,699,9,43.5 by=clip focus heard="Resize On" limit=386,693,1149,60 paints=#9C9CA0FF role=thumb
        54 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=464,699,66,43.5 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        55 GridSplitter at=521,699,9,43.5 by=clip focus heard="Resize Kind" limit=386,693,1149,60 paints=#9C9CA0FF role=thumb
        56 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=530,699,591,43.5 box=394,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        57 GridSplitter at=1112,699,9,43.5 by=clip focus heard="Resize Cheat" limit=386,693,1149,60 paints=#9C9CA0FF role=thumb
        58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1121,699,396,43.5 box=264,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
        59 ScrollViewer at=386,753,1149,126 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=386,753,1149,126
          60 ListBoxItem at=386,753,1149,63 by=fixed-scroll drawn focus heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=59 limit=386,753,1149,126 op paints=#333337FF,#9C9CA0FF role=listitem
            61 Grid at=401,759,1119,48 by=clip heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=59 limit=386,753,1149,63 role=none
              62 CheckBox at=401,759,30,48 by=clip drawn focus heard=On in=59 limit=386,753,1149,63 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                63 ViewboxContainer at=401,768,30,30 by=clip in=59 limit=401,768,30,30 role=none
              64 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=461,765,66,36 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=59 layout=27.74,24 owner=60 paints=#808080FF role=text size=20 text=RAM
              65 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=527,762,597,43.5 box=398,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat in=59 layout=100.06,28.8 owner=60 paints=#9C9CA0FF role=edit size=24 text="Infinite lives"
              66 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1124,762,396,42 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0100 = 0x42" in=59 layout=264,27.24 owner=60 paints=#808080FF role=text size=20 text="CpuBus 0x7E0100 = 0x42"
          67 ListBoxItem at=386,816,1149,63 by=fixed-scroll drawn focus heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=59 limit=386,753,1149,126 op paints=#333337FF,#9C9CA0FF role=listitem
            68 Grid at=401,822,1119,48 by=clip heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=59 limit=386,816,1149,63 role=none
              69 CheckBox at=401,822,30,48 by=clip drawn focus heard=On in=59 limit=386,816,1149,63 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                70 ViewboxContainer at=401,831,30,30 by=clip in=59 limit=401,831,30,30 role=none
              71 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=461,828,66,36 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=59 layout=27.74,24 owner=67 paints=#808080FF role=text size=20 text=RAM
              72 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=527,825,597,43.5 box=398,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Cheat in=59 layout=396.86,28.8 owner=67 paints=#9C9CA0FF role=edit size=24 text="Nine continues, and a description long enough to need the whole row or more of it"
              73 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1124,825,396,42 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0101 = 0x09" in=59 layout=264,27.24 owner=67 paints=#808080FF role=text size=20 text="CpuBus 0x7E0101 = 0x09"
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
edge 45 right 47
edge 47 up 67
edge 47 down same
edge 47 left 45
edge 47 right 49
edge 49 up 67
edge 49 down same
edge 49 left 47
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
edge 60 right 49
edge 62 up 29
edge 62 down 67
edge 62 left same
edge 62 right 53
edge 67 up 60
edge 67 down 41
edge 67 left same
edge 67 right 49
edge 69 up 60
edge 69 down 43
edge 69 left same
edge 69 right 55
press pageup focus="TextBox GBCodeBox (Cheat code)" tab=GB title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press first focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press last focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press back top=none
