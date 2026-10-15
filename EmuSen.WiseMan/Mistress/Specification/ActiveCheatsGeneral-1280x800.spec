# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheatsGeneral-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,76,812.7,616
title "Active Cheats"
case ActiveCheatsGeneral
entered shoulders
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,176,813,496 by=panel limit=234,176,812.35,496 paints=#9C9CA0FF role=none seen=234,176,812.35,496
    1 DockPanel at=258,188,765,468 by=panel limit=234,176,812.35,496 role=none
      2 Tabs at=258,188,765,239 by=panel limit=234,176,812.35,496 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=258,188,116,48 by=clip drawn focus heard=GENERAL limit=258,188,765,239 op paints=#EEEEF0FF role=tabitem sideways text=General
          4 TextBlock at=270,194,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=258,188,116,48 owner=3 paints=#EEEEF0FF role=text size=30 text=GENERAL
        5 TabItem at=374,188,65,48 by=clip drawn focus heard=NES limit=258,188,765,239 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=386,194,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=374,188,65,48 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=439,188,51,48 by=clip drawn focus heard=GB limit=258,188,765,239 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=451,194,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=439,188,51,48 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=490,188,77,48 by=clip drawn focus heard=SNES limit=258,188,765,239 paints=#86868AFF role=tabitem sideways text=SNES
          10 TextBlock at=502,194,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=490,188,77,48 owner=9 paints=#86868AFF role=text size=30 text=SNES
        11 TabItem at=567,188,64,48 by=clip drawn focus heard=N64 limit=258,188,765,239 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=579,194,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=567,188,64,48 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=631,188,110,48 by=clip drawn focus heard=GENESIS limit=258,188,765,239 paints=#86868AFF role=tabitem sideways text=Genesis
          14 TextBlock at=643,194,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=631,188,110,48 owner=13 paints=#86868AFF role=text size=30 text=GENESIS
        15 TabItem at=741,188,156,48 by=clip drawn focus heard=PLAYSTATION limit=258,188,765,239 paints=#86868AFF role=tabitem sideways text=PlayStation
          16 TextBlock at=753,194,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=741,188,156,48 owner=15 paints=#86868AFF role=text size=30 text=PLAYSTATION
        17 Grid at=270,246,741,181 by=clip limit=258,238,765,189 role=none
          18 Border at=270,246,359,181 by=clip limit=258,238,765,189 paints=#141416FF role=none
            19 StackPanel at=280,256,339,161 by=clip limit=258,238,765,189 role=none
              20 LunaSwitch at=280,256,173,32 by=clip drawn focus heard="Enable cheats" limit=258,238,765,189 name=MasterSwitch op paints=#050507FF,#4C9AE8FF,#EEEEF0FF,#FFFFFFFF role=button
                21 TextBlock at=332,258,115,29 box=115,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Enable cheats" layout=114.24,28.8 limit=280,256,173,32 owner=20 paints=#EEEEF0FF role=text size=24 text="Enable cheats"
              22 HintText at=280,292,339,48 box=339,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Off switches every cheat below off at once, without changing any of their own boxes." layout=292.08,48 limit=258,238,765,189 name=MasterHintText paints=#86868AFF role=text size=20 text="Off switches every cheat below off at once, without changing any of their own boxes."
          23 StackPanel at=653,246,358,181 by=clip limit=258,238,765,189 role=none
            24 TextBlock at=653,246,358,29 box=358,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Saved cheat list" layout=128.74,28.8 limit=258,238,765,189 paints=#9C9CA0FF role=text size=24 text="Saved cheat list"
            25 HintText at=653,281,359,96 box=359,96 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically." layout=357.02,96 limit=258,238,765,189 name=SaveTargetText paints=#86868AFF role=text size=20 text="Save writes 'Cobalt Harbor (Synthetic)', which comes back automatically next time you start it, and is kept up to date when you leave the game. Save As writes anywhere and is never loaded automatically."
            26 StackPanel at=653,383,358,44 by=clip limit=258,238,765,189 role=none
              27 Button at=653,383,79,44 by=clip drawn heard=LOAD limit=258,238,765,189 name=LoadButton off paints=#55555AFF,#9C9CA0FF role=button text=Load
                28 TextBlock at=670,389,45,32 box=45,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=LOAD layout=44.77,31.2 limit=653,383,79,44 off owner=27 paints=#55555AFF role=text size=26 text=LOAD
      29 TextBlock at=258,439,765,29 box=765,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 cheat(s), 2 on" layout=123.53,28.8 limit=234,176,812.35,496 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="2 cheat(s), 2 on"
      30 Border at=258,612,765,44 by=panel limit=234,176,812.35,496 role=none
        31 Grid at=258,612,765,44 by=panel limit=234,176,812.35,496 role=none
          32 ButtonBar at=285,612,711,44 by=panel limit=234,176,812.35,496 paints=#9C9CA0FF role=toolbar
            33 Button at=285,612,195,44 by=clip drawn focus heard="CHEAT DATABASE..." limit=285,612,711,44 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              34 TextBlock at=302,618,161,32 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=285,612,195,44 owner=33 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            35 Button at=488,612,160,44 by=clip drawn focus heard="APPLY CHEATS" limit=285,612,711,44 name=ApplyButton op paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              36 TextBlock at=505,618,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=488,612,160,44 owner=35 paints=#9C9CA0FF role=text size=26 text="APPLY CHEATS"
            37 Button at=656,612,77,44 by=clip drawn focus heard=SAVE limit=285,612,711,44 name=SaveButton op paints=#55555AFF,#9C9CA0FF role=button text=Save
              38 TextBlock at=673,618,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=656,612,77,44 owner=37 paints=#9C9CA0FF role=text size=26 text=SAVE
            39 Button at=741,612,105,44 by=clip drawn heard=REMOVE limit=285,612,711,44 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              40 TextBlock at=758,618,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=741,612,105,44 off owner=39 paints=#55555AFF role=text size=26 text=REMOVE
            41 Button at=854,612,142,44 by=clip drawn focus heard="REMOVE ALL" limit=285,612,711,44 name=ClearButton op paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              42 TextBlock at=871,618,108,32 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=854,612,142,44 owner=41 paints=#9C9CA0FF role=text size=26 text="REMOVE ALL"
      43 LunaTable at=258,476,765,124 by=panel heard="Cheats for this console" limit=234,176,812.35,496 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        44 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=270,480,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        45 GridSplitter at=304,480,6,29 by=clip focus heard="Resize On" limit=258,476,765,40 paints=#9C9CA0FF role=thumb
        46 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=310,480,44,29 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        47 GridSplitter at=348,480,6,29 by=clip focus heard="Resize Kind" limit=258,476,765,40 paints=#9C9CA0FF role=thumb
        48 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=354,480,393,29 box=393,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        49 GridSplitter at=741,480,6,29 by=clip focus heard="Resize Cheat" limit=258,476,765,40 paints=#9C9CA0FF role=thumb
        50 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=747,480,264,29 box=264,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
        51 ScrollViewer at=258,516,765,84 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=258,516,765,84
          52 ListBoxItem at=258,516,765,42 by=fixed-scroll drawn focus heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=51 limit=258,516,765,84 op paints=#333337FF,#9C9CA0FF role=listitem
            53 Grid at=268,520,745,32 by=clip heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=51 limit=258,516,765,42 role=none
              54 CheckBox at=268,520,20,32 by=clip drawn focus heard=On in=51 limit=258,516,765,42 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                55 ViewboxContainer at=268,526,20,20 by=clip in=51 limit=268,526,20,20 role=none
              56 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=308,524,44,24 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=51 layout=27.74,24 owner=52 paints=#808080FF role=text size=20 text=RAM
              57 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=352,522,397,29 box=397,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat in=51 layout=100.06,28.8 owner=52 paints=#9C9CA0FF role=edit size=24 text="Infinite lives"
              58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=749,522,264,28 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0100 = 0x42" in=51 layout=264,27.24 owner=52 paints=#808080FF role=text size=20 text="CpuBus 0x7E0100 = 0x42"
          59 ListBoxItem at=258,558,765,42 by=fixed-scroll drawn focus heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=51 limit=258,516,765,84 op paints=#333337FF,#9C9CA0FF role=listitem
            60 Grid at=268,562,745,32 by=clip heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=51 limit=258,558,765,42 role=none
              61 CheckBox at=268,562,20,32 by=clip drawn focus heard=On in=51 limit=258,558,765,42 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                62 ViewboxContainer at=268,568,20,20 by=clip in=51 limit=268,568,20,20 role=none
              63 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=308,566,44,24 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=51 layout=27.74,24 owner=59 paints=#808080FF role=text size=20 text=RAM
              64 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=352,564,397,29 box=397,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Cheat in=51 layout=396.86,28.8 owner=59 paints=#9C9CA0FF role=edit size=24 text="Nine continues, and a description long enough to need the whole row or more of it"
              65 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=749,564,264,28 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0101 = 0x09" in=51 layout=264,27.24 owner=59 paints=#808080FF role=text size=20 text="CpuBus 0x7E0101 = 0x09"
focus 20
edge 3 up same
edge 3 down 20
edge 20 up 3
edge 20 down 45
edge 20 left same
edge 20 right 3
edge 33 up 59
edge 33 down same
edge 33 left same
edge 33 right 35
edge 35 up 59
edge 35 down same
edge 35 left 33
edge 35 right 37
edge 37 up 59
edge 37 down same
edge 37 left 35
edge 37 right 41
edge 41 up 59
edge 41 down same
edge 41 left 37
edge 41 right same
edge 45 up 20
edge 45 down 52
edge 45 left 54
edge 45 right 47
edge 47 up 20
edge 47 down 52
edge 47 left 45
edge 47 right 49
edge 49 up 3
edge 49 down 52
edge 49 left 47
edge 49 right 41
edge 52 up 45
edge 52 down 59
edge 52 left same
edge 52 right same
edge 54 up 20
edge 54 down 59
edge 54 left same
edge 54 right 45
edge 59 up 52
edge 59 down 33
edge 59 left same
edge 59 right same
press pageup focus="TextBox PlayStationCodeBox (Cheat code)" tab=PlayStation title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press first focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press last focus="LunaSwitch MasterSwitch" tab=General title="Active Cheats" top=ActiveCheatsWindow
press back top=none
