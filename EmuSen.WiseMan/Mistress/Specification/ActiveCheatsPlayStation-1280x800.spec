# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheatsPlayStation-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,77,812.7,615
title "Active Cheats"
footer "Load a game's cheats from the Cheat Database, or add a code on a console tab."
case ActiveCheatsPlayStation
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,177,813,412 by=panel limit=234,177,812.35,412 paints=#9C9CA0FF role=none seen=234,177,812.35,412
    1 DockPanel at=258,189,765,384 by=panel limit=234,177,812.35,412 role=none
      2 Tabs at=258,189,765,239 by=panel limit=234,177,812.35,412 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=258,189,116,48 by=clip drawn focus heard=GENERAL limit=258,189,765,239 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=270,195,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=258,189,116,48 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=374,189,65,48 by=clip drawn focus heard=NES limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=386,195,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=374,189,65,48 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=439,189,51,48 by=clip drawn focus heard=GB limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=451,195,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=439,189,51,48 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=490,189,77,48 by=clip drawn focus heard=SNES limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=502,195,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=490,189,77,48 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=567,189,64,48 by=clip drawn focus heard=N64 limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=579,195,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=567,189,64,48 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=631,189,110,48 by=clip drawn focus heard=GENESIS limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text=Genesis
          14 TextBlock at=643,195,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=631,189,110,48 owner=13 paints=#86868AFF role=text size=30 text=GENESIS
        15 TabItem at=741,189,109,48 by=clip drawn focus heard="SEGA CD" limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text="Sega CD"
          16 TextBlock at=753,195,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=741,189,109,48 owner=15 paints=#86868AFF role=text size=30 text="SEGA CD"
        17 TabItem at=850,189,156,48 by=clip drawn focus heard=PLAYSTATION limit=258,189,765,239 paints=#86868AFF role=tabitem sideways text=PlayStation
          18 TextBlock at=862,195,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=850,189,156,48 owner=17 paints=#86868AFF role=text size=30 text=PLAYSTATION
        19 Grid at=270,247,741,181 by=clip limit=258,239,765,189 role=none
          20 Border at=270,247,359,181 by=clip limit=258,239,765,189 paints=#141416FF role=none
            21 StackPanel at=280,257,339,161 by=clip limit=258,239,765,189 role=none
              22 LunaSwitch at=280,257,173,32 by=clip drawn focus heard="Enable cheats" limit=258,239,765,189 name=MasterSwitch op paints=#4C9AE8FF,#9C9CA0FF,#FFFFFFFF role=button
                23 TextBlock at=332,259,115,29 box=115,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Enable cheats" layout=114.24,28.8 limit=280,257,173,32 owner=22 paints=#9C9CA0FF role=text size=24 text="Enable cheats"
              24 HintText at=280,293,339,48 box=339,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off switches every cheat below off at once, without changing any of their own boxes." layout=292.08,48 limit=258,239,765,189 name=MasterHintText paints=#86868AFF role=text size=20 text="Off switches every cheat below off at once, without changing any of their own boxes."
          25 StackPanel at=653,247,358,181 by=clip limit=258,239,765,189 role=none
            26 TextBlock at=653,247,358,29 box=358,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Saved cheat list" layout=128.74,28.8 limit=258,239,765,189 paints=#9C9CA0FF role=text size=24 text="Saved cheat list"
            27 HintText at=653,282,359,96 box=359,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically." layout=357.02,96 limit=258,239,765,189 name=SaveTargetText paints=#86868AFF role=text size=20 text="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically."
            28 StackPanel at=653,384,358,44 by=clip limit=258,239,765,189 role=none
              29 Button at=653,384,79,44 by=clip drawn heard=LOAD limit=258,239,765,189 name=LoadButton off paints=#55555AFF,#9C9CA0FF role=button text=Load
                30 TextBlock at=670,390,45,32 box=45,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=LOAD layout=44.77,31.2 limit=653,384,79,44 off owner=29 paints=#55555AFF role=text size=26 text=LOAD
      31 TextBlock at=258,440,765,29 box=765,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="No cheats loaded" layout=140.47,28.8 limit=234,177,812.35,412 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="No cheats loaded"
      32 Border at=258,529,765,44 by=panel limit=234,177,812.35,412 role=none
        33 Grid at=258,529,765,44 by=panel limit=234,177,812.35,412 role=none
          34 ButtonBar at=285,529,711,44 by=panel limit=234,177,812.35,412 paints=#9C9CA0FF role=toolbar
            35 Button at=285,529,195,44 by=clip drawn focus heard="CHEAT DATABASE..." limit=285,529,711,44 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              36 TextBlock at=302,535,161,32 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=285,529,195,44 owner=35 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            37 Button at=488,529,160,44 by=clip drawn heard="APPLY CHEATS" limit=285,529,711,44 name=ApplyButton off paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              38 TextBlock at=505,535,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=488,529,160,44 off owner=37 paints=#55555AFF role=text size=26 text="APPLY CHEATS"
            39 Button at=656,529,77,44 by=clip drawn heard=SAVE limit=285,529,711,44 name=SaveButton off paints=#55555AFF,#9C9CA0FF role=button text=Save
              40 TextBlock at=673,535,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=656,529,77,44 off owner=39 paints=#55555AFF role=text size=26 text=SAVE
            41 Button at=741,529,105,44 by=clip drawn heard=REMOVE limit=285,529,711,44 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              42 TextBlock at=758,535,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=741,529,105,44 off owner=41 paints=#55555AFF role=text size=26 text=REMOVE
            43 Button at=854,529,142,44 by=clip drawn heard="REMOVE ALL" limit=285,529,711,44 name=ClearButton off paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              44 TextBlock at=871,535,108,32 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=854,529,142,44 off owner=43 paints=#55555AFF role=text size=26 text="REMOVE ALL"
      45 LunaTable at=258,477,765,40 by=panel heard="Cheats for this console" limit=234,177,812.35,412 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        46 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=270,481,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        47 GridSplitter at=304,481,6,29 by=clip focus heard="Resize On" limit=258,477,765,40 paints=#9C9CA0FF role=thumb
        48 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=310,481,44,29 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        49 GridSplitter at=348,481,6,29 by=clip focus heard="Resize Kind" limit=258,477,765,40 paints=#9C9CA0FF role=thumb
        50 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=354,481,616,29 box=616,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        51 GridSplitter at=964,481,6,29 by=clip focus heard="Resize Cheat" limit=258,477,765,40 paints=#9C9CA0FF role=thumb
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=970,481,41,29 box=41,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
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
