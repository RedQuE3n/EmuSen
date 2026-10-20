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
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,141,813,448 by=panel limit=234,141,812.35,448 paints=#9C9CA0FF role=none seen=234,141,812.35,448
    1 DockPanel at=258,153,765,420 by=panel limit=234,141,812.35,448 role=none
      2 Tabs at=258,153,765,287 by=panel limit=234,141,812.35,448 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=258,153,116,48 by=clip drawn focus heard=GENERAL limit=258,153,765,287 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=270,159,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=258,153,116,48 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=374,153,65,48 by=clip drawn focus heard=NES limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=386,159,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=374,153,65,48 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=439,153,51,48 by=clip drawn focus heard=GB limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=451,159,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=439,153,51,48 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=490,153,77,48 by=clip drawn focus heard=SNES limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=502,159,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=490,153,77,48 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=567,153,64,48 by=clip drawn focus heard=N64 limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=579,159,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=567,153,64,48 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=631,153,107,48 by=clip drawn focus heard=SG-1000 limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=SG-1000
          14 TextBlock at=643,159,83,36 box=83,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=82.68,36 limit=631,153,107,48 owner=13 paints=#86868AFF role=text size=30 text=SG-1000
        15 TabItem at=738,153,186,48 by=clip drawn focus heard="MASTER SYSTEM" limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text="Master System"
          16 TextBlock at=750,159,162,36 box=162,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=161.79,36 limit=738,153,186,48 owner=15 paints=#86868AFF role=text size=30 text="MASTER SYSTEM"
        17 TabItem at=258,201,110,48 by=clip drawn focus heard=GENESIS limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=Genesis
          18 TextBlock at=270,207,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=258,201,110,48 owner=17 paints=#86868AFF role=text size=30 text=GENESIS
        19 TabItem at=368,201,109,48 by=clip drawn focus heard="SEGA CD" limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text="Sega CD"
          20 TextBlock at=380,207,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=368,201,109,48 owner=19 paints=#86868AFF role=text size=30 text="SEGA CD"
        21 TabItem at=477,201,156,48 by=clip drawn focus heard=PLAYSTATION limit=258,153,765,287 paints=#86868AFF role=tabitem sideways text=PlayStation
          22 TextBlock at=489,207,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=477,201,156,48 owner=21 paints=#86868AFF role=text size=30 text=PLAYSTATION
        23 Grid at=270,259,741,181 by=clip limit=258,251,765,189 role=none
          24 Border at=270,259,359,181 by=clip limit=258,251,765,189 paints=#141416FF role=none
            25 StackPanel at=280,269,339,161 by=clip limit=258,251,765,189 role=none
              26 LunaSwitch at=280,269,173,32 by=clip drawn focus heard="Enable cheats" limit=258,251,765,189 name=MasterSwitch op paints=#4C9AE8FF,#9C9CA0FF,#FFFFFFFF role=button
                27 TextBlock at=332,271,115,29 box=115,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Enable cheats" layout=114.24,28.8 limit=280,269,173,32 owner=26 paints=#9C9CA0FF role=text size=24 text="Enable cheats"
              28 HintText at=280,305,339,48 box=339,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off switches every cheat below off at once, without changing any of their own boxes." layout=292.08,48 limit=258,251,765,189 name=MasterHintText paints=#86868AFF role=text size=20 text="Off switches every cheat below off at once, without changing any of their own boxes."
          29 StackPanel at=653,259,358,181 by=clip limit=258,251,765,189 role=none
            30 TextBlock at=653,259,358,29 box=358,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Saved cheat list" layout=128.74,28.8 limit=258,251,765,189 paints=#9C9CA0FF role=text size=24 text="Saved cheat list"
            31 HintText at=653,294,359,96 box=359,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically." layout=357.02,96 limit=258,251,765,189 name=SaveTargetText paints=#86868AFF role=text size=20 text="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically."
            32 StackPanel at=653,396,358,44 by=clip limit=258,251,765,189 role=none
              33 Button at=653,396,79,44 by=clip drawn heard=LOAD limit=258,251,765,189 name=LoadButton off paints=#55555AFF,#9C9CA0FF role=button text=Load
                34 TextBlock at=670,402,45,32 box=45,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=LOAD layout=44.77,31.2 limit=653,396,79,44 off owner=33 paints=#55555AFF role=text size=26 text=LOAD
      35 TextBlock at=258,440,765,29 box=765,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No cheats loaded" layout=140.47,28.8 limit=234,141,812.35,448 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="No cheats loaded"
      36 Border at=258,529,765,44 by=panel limit=234,141,812.35,448 role=none
        37 Grid at=258,529,765,44 by=panel limit=234,141,812.35,448 role=none
          38 ButtonBar at=285,529,711,44 by=panel limit=234,141,812.35,448 paints=#9C9CA0FF role=toolbar
            39 Button at=285,529,195,44 by=clip drawn focus heard="CHEAT DATABASE..." limit=285,529,711,44 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              40 TextBlock at=302,535,161,32 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=285,529,195,44 owner=39 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            41 Button at=488,529,160,44 by=clip drawn heard="APPLY CHEATS" limit=285,529,711,44 name=ApplyButton off paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              42 TextBlock at=505,535,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=488,529,160,44 off owner=41 paints=#55555AFF role=text size=26 text="APPLY CHEATS"
            43 Button at=656,529,77,44 by=clip drawn heard=SAVE limit=285,529,711,44 name=SaveButton off paints=#55555AFF,#9C9CA0FF role=button text=Save
              44 TextBlock at=673,535,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=656,529,77,44 off owner=43 paints=#55555AFF role=text size=26 text=SAVE
            45 Button at=741,529,105,44 by=clip drawn heard=REMOVE limit=285,529,711,44 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              46 TextBlock at=758,535,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=741,529,105,44 off owner=45 paints=#55555AFF role=text size=26 text=REMOVE
            47 Button at=854,529,142,44 by=clip drawn heard="REMOVE ALL" limit=285,529,711,44 name=ClearButton off paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              48 TextBlock at=871,535,108,32 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=854,529,142,44 off owner=47 paints=#55555AFF role=text size=26 text="REMOVE ALL"
      49 LunaTable at=258,477,765,40 by=panel heard="Cheats for this console" limit=234,141,812.35,448 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        50 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=270,481,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        51 GridSplitter at=304,481,6,29 by=clip focus heard="Resize On" limit=258,477,765,40 paints=#9C9CA0FF role=thumb
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=310,481,44,29 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        53 GridSplitter at=348,481,6,29 by=clip focus heard="Resize Kind" limit=258,477,765,40 paints=#9C9CA0FF role=thumb
        54 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=354,481,616,29 box=616,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        55 GridSplitter at=964,481,6,29 by=clip focus heard="Resize Cheat" limit=258,477,765,40 paints=#9C9CA0FF role=thumb
        56 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=970,481,41,29 box=41,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
focus 3
edge 3 up same
edge 3 down 26
edge 26 up 3
edge 26 down 51
edge 26 left same
edge 26 right 3
edge 39 up 51
edge 39 down same
edge 39 left same
edge 39 right 3
edge 51 up 26
edge 51 down 39
edge 51 left same
edge 51 right 53
press pageup focus="TextBox PlayStationCodeBox (Cheat code)" tab=PlayStation title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press first focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press last focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press back top=none
