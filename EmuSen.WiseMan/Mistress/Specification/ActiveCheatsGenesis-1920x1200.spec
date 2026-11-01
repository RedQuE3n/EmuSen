# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheatsGenesis-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,62.5,1219.05,976.5
title "Active Cheats"
footer "Load a game's cheats from the Cheat Database, or add a code on a console tab."
case ActiveCheatsGenesis
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,212,1221,672 by=panel limit=350.48,212,1219.05,672 paints=#D4D4D4FF role=none seen=350.48,212,1219.05,672
    1 DockPanel at=386,230,1149,630 by=panel limit=350.48,212,1219.05,672 role=none
      2 Tabs at=386,230,1149,430.5 by=panel limit=350.48,212,1219.05,672 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=386,230,168,72 by=clip drawn focus heard=GENERAL limit=386,230,1149,430.5 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=404,239,132,54 box=88,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=87.63,36 limit=386,230,168,72 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=554,230,93,72 by=clip drawn focus heard=NES limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=572,239,57,54 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=37.71,36 limit=554,230,93,72 owner=5 paints=#8D8D91FF role=text size=30 text=NES
        7 TabItem at=647,230,75,72 by=clip drawn focus heard=GB limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=665,239,39,54 box=26,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=25.77,36 limit=647,230,75,72 owner=7 paints=#8D8D91FF role=text size=30 text=GB
        9 TabItem at=722,230,111,72 by=clip drawn focus heard=SNES limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=SNES
          10 TextBlock at=740,239,75,54 box=50,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=49.53,36 limit=722,230,111,72 owner=9 paints=#8D8D91FF role=text size=30 text=SNES
        11 TabItem at=833,230,93,72 by=clip drawn focus heard=N64 limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=N64
          12 TextBlock at=851,239,57,54 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=37.98,36 limit=833,230,93,72 owner=11 paints=#8D8D91FF role=text size=30 text=N64
        13 TabItem at=926,230,156,72 by=clip drawn focus heard=SG-1000 limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=944,239,120,54 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=79.8,36 limit=926,230,156,72 owner=13 paints=#8D8D91FF role=text size=30 text=SG-1000
        15 TabItem at=1082,230,267,72 by=clip drawn focus heard="MASTER SYSTEM" limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=1100,239,231,54 box=154,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=153.69,36 limit=1082,230,267,72 owner=15 paints=#8D8D91FF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=1349,230,156,72 by=clip drawn focus heard=GENESIS limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=1367,239,120,54 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=79.89,36 limit=1349,230,156,72 owner=17 paints=#8D8D91FF role=text size=30 text=GENESIS
        19 TabItem at=386,302,157.5,72 by=clip drawn focus heard="SEGA CD" limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=404,311,121.5,54 box=81,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=80.43,36 limit=386,302,157.5,72 owner=19 paints=#8D8D91FF role=text size=30 text="SEGA CD"
        21 TabItem at=543.5,302,220.5,72 by=clip drawn focus heard=PLAYSTATION limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          22 TextBlock at=561.5,311,184.5,54 box=123,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=122.67,36 limit=543.5,302,220.5,72 owner=21 paints=#8D8D91FF role=text size=30 text=PLAYSTATION
        23 TabItem at=764,302,247.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=386,230,1149,430.5 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          24 TextBlock at=782,311,211.5,54 box=141,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=140.19,36 limit=764,302,247.5,72 owner=23 paints=#8D8D91FF role=text size=30 text="PLAYSTATION 2"
        25 Grid at=404,389,1113,271.5 by=clip limit=386,377,1149,283.5 role=none
          26 Border at=404,389,538.5,271.5 by=clip limit=386,377,1149,283.5 paints=#26262BFF role=none
            27 StackPanel at=419,404,508.5,241.5 by=clip limit=386,377,1149,283.5 role=none
              28 LunaSwitch at=419,404,250.5,48 by=clip drawn focus heard="Enable cheats" limit=386,377,1149,283.5 name=MasterSwitch op paints=#4C9AE8FF,#D4D4D4FF,#FFFFFFFF role=button
                29 TextBlock at=497,407,163.5,43.5 box=109,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Enable cheats" layout=108.46,28.8 limit=419,404,250.5,48 owner=28 paints=#D4D4D4FF role=text size=24 text="Enable cheats"
              30 HintText at=419,458,508.5,72 box=339,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off switches every cheat below off at once, without changing any of their own boxes." layout=328.66,48 limit=386,377,1149,283.5 name=MasterHintText paints=#8D8D91FF role=text size=20 text="Off switches every cheat below off at once, without changing any of their own boxes."
          31 StackPanel at=978.5,389,538.5,271.5 by=clip limit=386,377,1149,283.5 role=none
            32 TextBlock at=978.5,389,538.5,43.5 box=359,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Saved cheat list" layout=122.06,28.8 limit=386,377,1149,283.5 paints=#D4D4D4FF role=text size=24 text="Saved cheat list"
            33 HintText at=978.5,441.5,538.5,144 box=359,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically." layout=338.66,96 limit=386,377,1149,283.5 name=SaveTargetText paints=#8D8D91FF role=text size=20 text="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically."
            34 StackPanel at=978.5,594.5,538.5,66 by=clip limit=386,377,1149,283.5 role=none
              35 Button at=978.5,594.5,117,66 by=clip drawn heard=LOAD limit=386,377,1149,283.5 name=LoadButton off paints=#707070FF,#D4D4D4FF role=button text=Load
                36 TextBlock at=1004,603.5,66,48 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=LOAD layout=43.37,31.2 limit=978.5,594.5,117,66 off owner=35 paints=#707070FF role=text size=26 text=LOAD
      37 TextBlock at=386,660.5,1149,43.5 box=766,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No cheats loaded" layout=134.62,28.8 limit=350.48,212,1219.05,672 name=ListHeaderText paints=#D4D4D4FF role=text size=24 text="No cheats loaded"
      38 Border at=386,794,1149,66 by=panel limit=350.48,212,1219.05,672 role=none
        39 Grid at=386,794,1149,66 by=panel limit=350.48,212,1219.05,672 role=none
          40 ButtonBar at=446,794,1030.5,66 by=panel limit=350.48,212,1219.05,672 paints=#D4D4D4FF role=toolbar
            41 Button at=446,794,283.5,66 by=clip drawn focus heard="CHEAT DATABASE..." limit=446,794,1030.5,66 name=DatabaseButton op paints=#707070FF,#D4D4D4FF role=button text="Cheat Database..."
              42 TextBlock at=471.5,803,232.5,48 box=155,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=154.41,31.2 limit=446,794,283.5,66 owner=41 paints=#D4D4D4FF role=text size=26 text="CHEAT DATABASE..."
            43 Button at=741.5,794,229.5,66 by=clip drawn heard="APPLY CHEATS" limit=446,794,1030.5,66 name=ApplyButton off paints=#707070FF,#D4D4D4FF role=button text="Apply Cheats"
              44 TextBlock at=767,803,178.5,48 box=119,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=118.22,31.2 limit=741.5,794,229.5,66 off owner=43 paints=#707070FF role=text size=26 text="APPLY CHEATS"
            45 Button at=983,794,112.5,66 by=clip drawn heard=SAVE limit=446,794,1030.5,66 name=SaveButton off paints=#707070FF,#D4D4D4FF role=button text=Save
              46 TextBlock at=1008.5,803,61.5,48 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=40.87,31.2 limit=983,794,112.5,66 off owner=45 paints=#707070FF role=text size=26 text=SAVE
            47 Button at=1107.5,794,151.5,66 by=clip drawn heard=REMOVE limit=446,794,1030.5,66 name=RemoveButton off paints=#707070FF,#D4D4D4FF role=button text=Remove
              48 TextBlock at=1133,803,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=1107.5,794,151.5,66 off owner=47 paints=#707070FF role=text size=26 text=REMOVE
            49 Button at=1271,794,205.5,66 by=clip drawn heard="REMOVE ALL" limit=446,794,1030.5,66 name=ClearButton off paints=#707070FF,#D4D4D4FF role=button text="Remove All"
              50 TextBlock at=1296.5,803,154.5,48 box=103,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=102.49,31.2 limit=1271,794,205.5,66 off owner=49 paints=#707070FF role=text size=26 text="REMOVE ALL"
      51 LunaTable at=386,716,1149,60 by=panel heard="Cheats for this console" limit=350.48,212,1219.05,672 name=CheatsList paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=404,722,60,43.5 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=20.21,28.8 paints=#D4D4D4FF role=text size=24 text=On
        53 GridSplitter at=455,722,9,43.5 by=clip focus heard="Resize On" limit=386,716,1149,60 paints=#D4D4D4FF role=thumb
        54 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=464,722,66,43.5 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=34.25,28.8 paints=#D4D4D4FF role=text size=24 text=Kind
        55 GridSplitter at=521,722,9,43.5 by=clip focus heard="Resize Kind" limit=386,716,1149,60 paints=#D4D4D4FF role=thumb
        56 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=530,722,928.5,43.5 box=619,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=44.11,28.8 paints=#D4D4D4FF role=text size=24 text=Cheat
        57 GridSplitter at=1449.5,722,9,43.5 by=clip focus heard="Resize Cheat" limit=386,716,1149,60 paints=#D4D4D4FF role=thumb
        58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=1458.5,722,58.5,43.5 box=39,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=38.81,28.8 paints=#D4D4D4FF role=text size=24 text=Code
focus 3
edge 3 up same
edge 3 down 28
edge 28 up 3
edge 28 down 53
edge 28 left same
edge 28 right 57
edge 41 up 53
edge 41 down same
edge 41 left same
edge 41 right 57
edge 53 up 28
edge 53 down 41
edge 53 left same
edge 53 right 55
edge 57 up 3
edge 57 down 41
edge 57 left 55
edge 57 right same
press pageup focus="TextBox PlayStation 2CodeBox (Cheat code)" tab="PlayStation 2" title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press first focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press last focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press back top=none
