# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheats-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,171,1219.05,819
title "Active Cheats"
case ActiveCheats
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,321,1221,639 by=panel limit=350.48,321,1219.05,639 paints=#9C9CA0FF role=none seen=350.48,321,1219.05,639
    1 DockPanel at=386,339,1149,597 by=panel limit=350.48,321,1219.05,639 role=none
      2 Tabs at=386,339,1149,253.5 by=panel limit=350.48,321,1219.05,639 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=386,339,174,72 by=clip drawn focus heard=GENERAL limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text=General
          4 TextBlock at=404,348,138,54 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=386,339,174,72 owner=3 paints=#86868AFF role=text size=30 text=GENERAL
        5 TabItem at=560,339,97.5,72 by=clip drawn focus heard=NES limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=578,348,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=560,339,97.5,72 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=657.5,339,76.5,72 by=clip drawn focus heard=GB limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=675.5,348,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=657.5,339,76.5,72 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=734,339,115.5,72 by=clip drawn focus heard=SNES limit=386,339,1149,253.5 op paints=#EEEEF0FF role=tabitem sideways text=SNES
          10 TextBlock at=752,348,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=734,339,115.5,72 owner=9 paints=#EEEEF0FF role=text size=30 text=SNES
        11 TabItem at=849.5,339,96,72 by=clip drawn focus heard=N64 limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=867.5,348,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=849.5,339,96,72 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=945.5,339,165,72 by=clip drawn focus heard=GENESIS limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text=Genesis
          14 TextBlock at=963.5,348,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=945.5,339,165,72 owner=13 paints=#86868AFF role=text size=30 text=GENESIS
        15 TabItem at=1110.5,339,163.5,72 by=clip drawn focus heard="SEGA CD" limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text="Sega CD"
          16 TextBlock at=1128.5,348,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=1110.5,339,163.5,72 owner=15 paints=#86868AFF role=text size=30 text="SEGA CD"
        17 TabItem at=1274,339,234,72 by=clip drawn focus heard=PLAYSTATION limit=386,339,1149,253.5 paints=#86868AFF role=tabitem sideways text=PlayStation
          18 TextBlock at=1292,348,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=1274,339,234,72 owner=17 paints=#86868AFF role=text size=30 text=PLAYSTATION
        19 StackPanel at=404,426,1113,166.5 by=clip limit=386,414,1149,178.5 role=none
          20 TextBlock at=404,426,1113,43.5 box=742,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a SNES cheat" layout=142.18,28.8 limit=386,414,1149,178.5 paints=#9C9CA0FF role=text size=24 text="Add a SNES cheat"
          21 HintText at=404,478.5,1113,36 box=742,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Accepts: Game Genie, Pro Action Replay." layout=271.12,24 limit=386,414,1149,178.5 paints=#86868AFF role=text size=20 text="Accepts: Game Genie, Pro Action Replay."
          22 Grid at=404,523.5,1113,69 by=clip limit=386,414,1149,178.5 role=none
            23 TextBox at=404,523.5,499.5,69 by=clip drawn focus heard="Cheat code" limit=386,414,1149,178.5 name=SNESCodeBox op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              24 ScrollViewer allow="a text box scrolls its own text" at=420.5,534,466.5,48 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                25 TextBlock allow="a text box scrolls its own text" at=420.5,534,466.5,48 box=311,32 caps=18.1987 drawn face="Barlow Condensed" heard=Code in=24 layout=43.65,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Code
            26 TextBox at=915.5,523.5,486,69 by=clip drawn focus heard="Cheat description" limit=386,414,1149,178.5 name=SNESDescriptionBox op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              27 ScrollViewer allow="a text box scrolls its own text" at=932,534,453,48 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                28 TextBlock allow="a text box scrolls its own text" at=932,534,453,48 box=302,32 caps=18.1987 drawn face="Barlow Condensed" heard=Description in=27 layout=101.09,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Description
            29 Button at=1413.5,525,103.5,66 by=clip drawn focus heard=ADD limit=386,414,1149,178.5 name=SNESAddButton op paints=#55555AFF,#9C9CA0FF role=button text=Add
              30 TextBlock at=1439,534,52.5,48 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ADD layout=34.66,31.2 limit=1413.5,525,103.5,66 owner=29 paints=#9C9CA0FF role=text size=26 text=ADD
      31 TextBlock at=386,610.5,1149,43.5 box=766,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 cheat(s), 2 on" layout=123.53,28.8 limit=350.48,321,1219.05,639 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="2 cheat(s), 2 on"
      32 Border at=386,870,1149,66 by=panel limit=350.48,321,1219.05,639 role=none
        33 Grid at=386,870,1149,66 by=panel limit=350.48,321,1219.05,639 role=none
          34 ButtonBar at=428,870,1066.5,66 by=panel limit=350.48,321,1219.05,639 paints=#9C9CA0FF role=toolbar
            35 Button at=428,870,292.5,66 by=clip drawn focus heard="CHEAT DATABASE..." limit=428,870,1066.5,66 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              36 TextBlock at=453.5,879,241.5,48 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=428,870,292.5,66 owner=35 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            37 Button at=732.5,870,240,66 by=clip drawn focus heard="APPLY CHEATS" limit=428,870,1066.5,66 name=ApplyButton op paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              38 TextBlock at=758,879,189,48 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=732.5,870,240,66 owner=37 paints=#9C9CA0FF role=text size=26 text="APPLY CHEATS"
            39 Button at=984.5,870,115.5,66 by=clip drawn focus heard=SAVE limit=428,870,1066.5,66 name=SaveButton op paints=#55555AFF,#9C9CA0FF role=button text=Save
              40 TextBlock at=1010,879,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=984.5,870,115.5,66 owner=39 paints=#9C9CA0FF role=text size=26 text=SAVE
            41 Button at=1112,870,157.5,66 by=clip drawn heard=REMOVE limit=428,870,1066.5,66 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              42 TextBlock at=1137.5,879,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=1112,870,157.5,66 off owner=41 paints=#55555AFF role=text size=26 text=REMOVE
            43 Button at=1281.5,870,213,66 by=clip drawn focus heard="REMOVE ALL" limit=428,870,1066.5,66 name=ClearButton op paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              44 TextBlock at=1307,879,162,48 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=1281.5,870,213,66 owner=43 paints=#9C9CA0FF role=text size=26 text="REMOVE ALL"
      45 LunaTable at=386,666,1149,186 by=panel heard="Cheats for this console" limit=350.48,321,1219.05,639 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        46 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=404,672,60,43.5 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        47 GridSplitter at=455,672,9,43.5 by=clip focus heard="Resize On" limit=386,666,1149,60 paints=#9C9CA0FF role=thumb
        48 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=464,672,66,43.5 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        49 GridSplitter at=521,672,9,43.5 by=clip focus heard="Resize Kind" limit=386,666,1149,60 paints=#9C9CA0FF role=thumb
        50 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=530,672,591,43.5 box=394,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        51 GridSplitter at=1112,672,9,43.5 by=clip focus heard="Resize Cheat" limit=386,666,1149,60 paints=#9C9CA0FF role=thumb
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1121,672,396,43.5 box=264,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
        53 ScrollViewer at=386,726,1149,126 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=386,726,1149,126
          54 ListBoxItem at=386,726,1149,63 by=fixed-scroll drawn focus heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=53 limit=386,726,1149,126 op paints=#333337FF,#9C9CA0FF role=listitem
            55 Grid at=401,732,1119,48 by=clip heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=53 limit=386,726,1149,63 role=none
              56 CheckBox at=401,732,30,48 by=clip drawn focus heard=On in=53 limit=386,726,1149,63 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                57 ViewboxContainer at=401,741,30,30 by=clip in=53 limit=401,741,30,30 role=none
              58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=461,738,66,36 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=53 layout=27.74,24 owner=54 paints=#808080FF role=text size=20 text=RAM
              59 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=527,735,597,43.5 box=398,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat in=53 layout=100.06,28.8 owner=54 paints=#9C9CA0FF role=edit size=24 text="Infinite lives"
              60 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1124,735,396,42 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0100 = 0x42" in=53 layout=264,27.24 owner=54 paints=#808080FF role=text size=20 text="CpuBus 0x7E0100 = 0x42"
          61 ListBoxItem at=386,789,1149,63 by=fixed-scroll drawn focus heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=53 limit=386,726,1149,126 op paints=#333337FF,#9C9CA0FF role=listitem
            62 Grid at=401,795,1119,48 by=clip heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=53 limit=386,789,1149,63 role=none
              63 CheckBox at=401,795,30,48 by=clip drawn focus heard=On in=53 limit=386,789,1149,63 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                64 ViewboxContainer at=401,804,30,30 by=clip in=53 limit=401,804,30,30 role=none
              65 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=461,801,66,36 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=53 layout=27.74,24 owner=61 paints=#808080FF role=text size=20 text=RAM
              66 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=527,798,597,43.5 box=398,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Cheat in=53 layout=396.86,28.8 owner=61 paints=#9C9CA0FF role=edit size=24 text="Nine continues, and a description long enough to need the whole row or more of it"
              67 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1124,798,396,42 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0101 = 0x09" in=53 layout=264,27.24 owner=61 paints=#808080FF role=text size=20 text="CpuBus 0x7E0101 = 0x09"
focus 23
edge 9 up same
edge 9 down 23
edge 23 up 9
edge 23 down 47
edge 23 left same
edge 23 right 26
edge 26 up 9
edge 26 down 51
edge 26 left 23
edge 26 right 29
edge 29 up 9
edge 29 down 54
edge 29 left 26
edge 29 right same
edge 35 up 61
edge 35 down same
edge 35 left 63
edge 35 right 37
edge 37 up 61
edge 37 down same
edge 37 left 35
edge 37 right 39
edge 39 up 61
edge 39 down same
edge 39 left 37
edge 39 right 41
edge 41 up 61
edge 41 down same
edge 41 left 39
edge 41 right 43
edge 43 up 61
edge 43 down same
edge 43 left 41
edge 43 right same
edge 47 up 23
edge 47 down 54
edge 47 left 56
edge 47 right 49
edge 49 up 23
edge 49 down 54
edge 49 left 47
edge 49 right 51
edge 51 up 26
edge 51 down 54
edge 51 left 49
edge 51 right 9
edge 54 up 47
edge 54 down 61
edge 54 left same
edge 54 right 43
edge 56 up 23
edge 56 down 61
edge 56 left same
edge 56 right 47
edge 61 up 54
edge 61 down 35
edge 61 left same
edge 61 right 43
edge 63 up 54
edge 63 down 37
edge 63 left same
edge 63 right 49
press pageup focus="TextBox GBCodeBox (Cheat code)" tab=GB title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press first focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press last focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press back top=none
