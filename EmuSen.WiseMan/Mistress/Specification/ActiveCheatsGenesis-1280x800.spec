# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheatsGenesis-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,41,812.7,651
title "Active Cheats"
footer "Load a game's cheats from the Cheat Database, or add a code on a console tab."
case ActiveCheatsGenesis
entered down
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,141,813,448 by=panel limit=234,141,812.35,448 paints=#D4D4D4FF role=none seen=234,141,812.35,448
    1 DockPanel at=258,153,765,420 by=panel limit=234,141,812.35,448 role=none
      2 Tabs at=258,153,765,287 by=panel limit=234,141,812.35,448 name=Tabs paints=#D4D4D4FF role=tab
        3 TabItem at=258,153,112,48 by=clip drawn focus heard=GENERAL limit=258,153,765,287 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=270,159,88,36 box=88,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=87.63,36 limit=258,153,112,48 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=370,153,62,48 by=clip drawn focus heard=NES limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=NES
          6 TextBlock at=382,159,38,36 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=37.71,36 limit=370,153,62,48 owner=5 paints=#8D8D91FF role=text size=30 text=NES
        7 TabItem at=432,153,50,48 by=clip drawn focus heard=GB limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=GB
          8 TextBlock at=444,159,26,36 box=26,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=25.77,36 limit=432,153,50,48 owner=7 paints=#8D8D91FF role=text size=30 text=GB
        9 TabItem at=482,153,74,48 by=clip drawn focus heard=SNES limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=SNES
          10 TextBlock at=494,159,50,36 box=50,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=49.53,36 limit=482,153,74,48 owner=9 paints=#8D8D91FF role=text size=30 text=SNES
        11 TabItem at=556,153,62,48 by=clip drawn focus heard=N64 limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=N64
          12 TextBlock at=568,159,38,36 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=37.98,36 limit=556,153,62,48 owner=11 paints=#8D8D91FF role=text size=30 text=N64
        13 TabItem at=618,153,104,48 by=clip drawn focus heard=SG-1000 limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          14 TextBlock at=630,159,80,36 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=79.8,36 limit=618,153,104,48 owner=13 paints=#8D8D91FF role=text size=30 text=SG-1000
        15 TabItem at=722,153,178,48 by=clip drawn focus heard="MASTER SYSTEM" limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text="Master System"
          16 TextBlock at=734,159,154,36 box=154,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=153.69,36 limit=722,153,178,48 owner=15 paints=#8D8D91FF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=900,153,104,48 by=clip drawn focus heard=GENESIS limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=Genesis
          18 TextBlock at=912,159,80,36 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=79.89,36 limit=900,153,104,48 owner=17 paints=#8D8D91FF role=text size=30 text=GENESIS
        19 TabItem at=258,201,105,48 by=clip drawn focus heard="SEGA CD" limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          20 TextBlock at=270,207,81,36 box=81,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=80.43,36 limit=258,201,105,48 owner=19 paints=#8D8D91FF role=text size=30 text="SEGA CD"
        21 TabItem at=363,201,147,48 by=clip drawn focus heard=PLAYSTATION limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text=PlayStation
          22 TextBlock at=375,207,123,36 box=123,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=122.67,36 limit=363,201,147,48 owner=21 paints=#8D8D91FF role=text size=30 text=PLAYSTATION
        23 TabItem at=510,201,165,48 by=clip drawn focus heard="PLAYSTATION 2" limit=258,153,765,287 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          24 TextBlock at=522,207,141,36 box=141,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=140.19,36 limit=510,201,165,48 owner=23 paints=#8D8D91FF role=text size=30 text="PLAYSTATION 2"
        25 Grid at=270,259,741,181 by=clip limit=258,251,765,189 role=none
          26 Border at=270,259,359,181 by=clip limit=258,251,765,189 paints=#26262BFF role=none
            27 StackPanel at=280,269,339,161 by=clip limit=258,251,765,189 role=none
              28 LunaSwitch at=280,269,167,32 by=clip drawn focus heard="Enable cheats" limit=258,251,765,189 name=MasterSwitch op paints=#4C9AE8FF,#D4D4D4FF,#FFFFFFFF role=button
                29 TextBlock at=332,271,109,29 box=109,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Enable cheats" layout=108.46,28.8 limit=280,269,167,32 owner=28 paints=#D4D4D4FF role=text size=24 text="Enable cheats"
              30 HintText at=280,305,339,48 box=339,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off switches every cheat below off at once, without changing any of their own boxes." layout=328.66,48 limit=258,251,765,189 name=MasterHintText paints=#8D8D91FF role=text size=20 text="Off switches every cheat below off at once, without changing any of their own boxes."
          31 StackPanel at=653,259,358,181 by=clip limit=258,251,765,189 role=none
            32 TextBlock at=653,259,358,29 box=358,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Saved cheat list" layout=122.06,28.8 limit=258,251,765,189 paints=#D4D4D4FF role=text size=24 text="Saved cheat list"
            33 HintText at=653,294,358,96 box=358,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically." layout=338.66,96 limit=258,251,765,189 name=SaveTargetText paints=#8D8D91FF role=text size=20 text="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically."
            34 StackPanel at=653,396,358,44 by=clip limit=258,251,765,189 role=none
              35 Button at=653,396,78,44 by=clip drawn heard=LOAD limit=258,251,765,189 name=LoadButton off paints=#707070FF,#D4D4D4FF role=button text=Load
                36 TextBlock at=670,402,44,32 box=44,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=LOAD layout=43.37,31.2 limit=653,396,78,44 off owner=35 paints=#707070FF role=text size=26 text=LOAD
      37 TextBlock at=258,440,765,29 box=765,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No cheats loaded" layout=134.62,28.8 limit=234,141,812.35,448 name=ListHeaderText paints=#D4D4D4FF role=text size=24 text="No cheats loaded"
      38 Border at=258,529,765,44 by=panel limit=234,141,812.35,448 role=none
        39 Grid at=258,529,765,44 by=panel limit=234,141,812.35,448 role=none
          40 ButtonBar at=297,529,687,44 by=panel limit=234,141,812.35,448 paints=#D4D4D4FF role=toolbar
            41 Button at=297,529,189,44 by=clip drawn focus heard="CHEAT DATABASE..." limit=297,529,687,44 name=DatabaseButton op paints=#707070FF,#D4D4D4FF role=button text="Cheat Database..."
              42 TextBlock at=314,535,155,32 box=155,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=154.41,31.2 limit=297,529,189,44 owner=41 paints=#D4D4D4FF role=text size=26 text="CHEAT DATABASE..."
            43 Button at=494,529,153,44 by=clip drawn heard="APPLY CHEATS" limit=297,529,687,44 name=ApplyButton off paints=#707070FF,#D4D4D4FF role=button text="Apply Cheats"
              44 TextBlock at=511,535,119,32 box=119,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=118.22,31.2 limit=494,529,153,44 off owner=43 paints=#707070FF role=text size=26 text="APPLY CHEATS"
            45 Button at=655,529,75,44 by=clip drawn heard=SAVE limit=297,529,687,44 name=SaveButton off paints=#707070FF,#D4D4D4FF role=button text=Save
              46 TextBlock at=672,535,41,32 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=40.87,31.2 limit=655,529,75,44 off owner=45 paints=#707070FF role=text size=26 text=SAVE
            47 Button at=738,529,101,44 by=clip drawn heard=REMOVE limit=297,529,687,44 name=RemoveButton off paints=#707070FF,#D4D4D4FF role=button text=Remove
              48 TextBlock at=755,535,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=738,529,101,44 off owner=47 paints=#707070FF role=text size=26 text=REMOVE
            49 Button at=847,529,137,44 by=clip drawn heard="REMOVE ALL" limit=297,529,687,44 name=ClearButton off paints=#707070FF,#D4D4D4FF role=button text="Remove All"
              50 TextBlock at=864,535,103,32 box=103,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=102.49,31.2 limit=847,529,137,44 off owner=49 paints=#707070FF role=text size=26 text="REMOVE ALL"
      51 LunaTable at=258,477,765,40 by=panel heard="Cheats for this console" limit=234,141,812.35,448 name=CheatsList paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=datagrid
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=270,481,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=20.21,28.8 paints=#D4D4D4FF role=text size=24 text=On
        53 GridSplitter at=304,481,6,29 by=clip focus heard="Resize On" limit=258,477,765,40 paints=#D4D4D4FF role=thumb
        54 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=310,481,44,29 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=34.25,28.8 paints=#D4D4D4FF role=text size=24 text=Kind
        55 GridSplitter at=348,481,6,29 by=clip focus heard="Resize Kind" limit=258,477,765,40 paints=#D4D4D4FF role=thumb
        56 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=354,481,618,29 box=618,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=44.11,28.8 paints=#D4D4D4FF role=text size=24 text=Cheat
        57 GridSplitter at=966,481,6,29 by=clip focus heard="Resize Cheat" limit=258,477,765,40 paints=#D4D4D4FF role=thumb
        58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=972,481,39,29 box=39,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=38.81,28.8 paints=#D4D4D4FF role=text size=24 text=Code
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
