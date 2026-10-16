# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheatsGenesis-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,116.5,1219.05,922.5
title "Active Cheats"
footer "Load a game's cheats from the Cheat Database, or add a code on a console tab."
case ActiveCheatsGenesis
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,266,1221,618 by=panel limit=350.48,266,1219.05,618 paints=#9C9CA0FF role=none seen=350.48,266,1219.05,618
    1 DockPanel at=386,284,1149,576 by=panel limit=350.48,266,1219.05,618 role=none
      2 Tabs at=386,284,1149,358.5 by=panel limit=350.48,266,1219.05,618 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=386,284,174,72 by=clip drawn focus heard=GENERAL limit=386,284,1149,358.5 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=404,293,138,54 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=386,284,174,72 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=560,284,97.5,72 by=clip drawn focus heard=NES limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=578,293,61.5,54 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=560,284,97.5,72 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=657.5,284,76.5,72 by=clip drawn focus heard=GB limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=675.5,293,40.5,54 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=657.5,284,76.5,72 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=734,284,115.5,72 by=clip drawn focus heard=SNES limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=752,293,79.5,54 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=734,284,115.5,72 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=849.5,284,96,72 by=clip drawn focus heard=N64 limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=867.5,293,60,54 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=849.5,284,96,72 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=945.5,284,165,72 by=clip drawn focus heard=GENESIS limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text=Genesis
          14 TextBlock at=963.5,293,129,54 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=945.5,284,165,72 owner=13 paints=#86868AFF role=text size=30 text=GENESIS
        15 TabItem at=1110.5,284,163.5,72 by=clip drawn focus heard="SEGA CD" limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text="Sega CD"
          16 TextBlock at=1128.5,293,127.5,54 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=1110.5,284,163.5,72 owner=15 paints=#86868AFF role=text size=30 text="SEGA CD"
        17 TabItem at=1274,284,234,72 by=clip drawn focus heard=PLAYSTATION limit=386,284,1149,358.5 paints=#86868AFF role=tabitem sideways text=PlayStation
          18 TextBlock at=1292,293,198,54 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=1274,284,234,72 owner=17 paints=#86868AFF role=text size=30 text=PLAYSTATION
        19 Grid at=404,371,1113,271.5 by=clip limit=386,359,1149,283.5 role=none
          20 Border at=404,371,538.5,271.5 by=clip limit=386,359,1149,283.5 paints=#141416FF role=none
            21 StackPanel at=419,386,508.5,241.5 by=clip limit=386,359,1149,283.5 role=none
              22 LunaSwitch at=419,386,259.5,48 by=clip drawn focus heard="Enable cheats" limit=386,359,1149,283.5 name=MasterSwitch op paints=#4C9AE8FF,#9C9CA0FF,#FFFFFFFF role=button
                23 TextBlock at=497,389,172.5,43.5 box=115,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Enable cheats" layout=114.24,28.8 limit=419,386,259.5,48 owner=22 paints=#9C9CA0FF role=text size=24 text="Enable cheats"
              24 HintText at=419,440,508.5,72 box=339,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off switches every cheat below off at once, without changing any of their own boxes." layout=292.08,48 limit=386,359,1149,283.5 name=MasterHintText paints=#86868AFF role=text size=20 text="Off switches every cheat below off at once, without changing any of their own boxes."
          25 StackPanel at=978.5,371,538.5,271.5 by=clip limit=386,359,1149,283.5 role=none
            26 TextBlock at=978.5,371,538.5,43.5 box=359,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Saved cheat list" layout=128.74,28.8 limit=386,359,1149,283.5 paints=#9C9CA0FF role=text size=24 text="Saved cheat list"
            27 HintText at=978.5,423.5,538.5,144 box=359,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically." layout=357.02,96 limit=386,359,1149,283.5 name=SaveTargetText paints=#86868AFF role=text size=20 text="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically."
            28 StackPanel at=978.5,576.5,538.5,66 by=clip limit=386,359,1149,283.5 role=none
              29 Button at=978.5,576.5,118.5,66 by=clip drawn heard=LOAD limit=386,359,1149,283.5 name=LoadButton off paints=#55555AFF,#9C9CA0FF role=button text=Load
                30 TextBlock at=1004,585.5,67.5,48 box=45,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=LOAD layout=44.77,31.2 limit=978.5,576.5,118.5,66 off owner=29 paints=#55555AFF role=text size=26 text=LOAD
      31 TextBlock at=386,660.5,1149,43.5 box=766,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No cheats loaded" layout=140.47,28.8 limit=350.48,266,1219.05,618 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="No cheats loaded"
      32 Border at=386,794,1149,66 by=panel limit=350.48,266,1219.05,618 role=none
        33 Grid at=386,794,1149,66 by=panel limit=350.48,266,1219.05,618 role=none
          34 ButtonBar at=428,794,1066.5,66 by=panel limit=350.48,266,1219.05,618 paints=#9C9CA0FF role=toolbar
            35 Button at=428,794,292.5,66 by=clip drawn focus heard="CHEAT DATABASE..." limit=428,794,1066.5,66 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              36 TextBlock at=453.5,803,241.5,48 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=428,794,292.5,66 owner=35 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            37 Button at=732.5,794,240,66 by=clip drawn heard="APPLY CHEATS" limit=428,794,1066.5,66 name=ApplyButton off paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              38 TextBlock at=758,803,189,48 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=732.5,794,240,66 off owner=37 paints=#55555AFF role=text size=26 text="APPLY CHEATS"
            39 Button at=984.5,794,115.5,66 by=clip drawn heard=SAVE limit=428,794,1066.5,66 name=SaveButton off paints=#55555AFF,#9C9CA0FF role=button text=Save
              40 TextBlock at=1010,803,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=984.5,794,115.5,66 off owner=39 paints=#55555AFF role=text size=26 text=SAVE
            41 Button at=1112,794,157.5,66 by=clip drawn heard=REMOVE limit=428,794,1066.5,66 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              42 TextBlock at=1137.5,803,106.5,48 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=1112,794,157.5,66 off owner=41 paints=#55555AFF role=text size=26 text=REMOVE
            43 Button at=1281.5,794,213,66 by=clip drawn heard="REMOVE ALL" limit=428,794,1066.5,66 name=ClearButton off paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              44 TextBlock at=1307,803,162,48 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=1281.5,794,213,66 off owner=43 paints=#55555AFF role=text size=26 text="REMOVE ALL"
      45 LunaTable at=386,716,1149,60 by=panel heard="Cheats for this console" limit=350.48,266,1219.05,618 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        46 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=404,722,60,43.5 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        47 GridSplitter at=455,722,9,43.5 by=clip focus heard="Resize On" limit=386,716,1149,60 paints=#9C9CA0FF role=thumb
        48 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=464,722,66,43.5 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        49 GridSplitter at=521,722,9,43.5 by=clip focus heard="Resize Kind" limit=386,716,1149,60 paints=#9C9CA0FF role=thumb
        50 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=530,722,925.5,43.5 box=617,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        51 GridSplitter at=1446.5,722,9,43.5 by=clip focus heard="Resize Cheat" limit=386,716,1149,60 paints=#9C9CA0FF role=thumb
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1455.5,722,61.5,43.5 box=41,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
focus 3
edge 3 up same
edge 3 down 22
edge 22 up 3
edge 22 down 47
edge 22 left same
edge 22 right 3
edge 35 up 47
edge 35 down same
edge 35 left same
edge 35 right 3
edge 47 up 22
edge 47 down 35
edge 47 left same
edge 47 right 49
press pageup focus="TextBox PlayStationCodeBox (Cheat code)" tab=PlayStation title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press first focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press last focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press back top=none
