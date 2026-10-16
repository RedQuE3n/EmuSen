# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ActiveCheats-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,114,812.7,546
title "Active Cheats"
case ActiveCheats
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,214,813,426 by=panel limit=234,214,812.35,426 paints=#9C9CA0FF role=none seen=234,214,812.35,426
    1 DockPanel at=258,226,765,398 by=panel limit=234,214,812.35,426 role=none
      2 Tabs at=258,226,765,169 by=panel limit=234,214,812.35,426 name=Tabs paints=#9C9CA0FF role=tab
        3 TabItem at=258,226,116,48 by=clip drawn focus heard=GENERAL limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text=General
          4 TextBlock at=270,232,92,36 box=92,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENERAL layout=91.92,36 limit=258,226,116,48 owner=3 paints=#86868AFF role=text size=30 text=GENERAL
        5 TabItem at=374,226,65,48 by=clip drawn focus heard=NES limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text=NES
          6 TextBlock at=386,232,41,36 box=41,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=40.14,36 limit=374,226,65,48 owner=5 paints=#86868AFF role=text size=30 text=NES
        7 TabItem at=439,226,51,48 by=clip drawn focus heard=GB limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text=GB
          8 TextBlock at=451,232,27,36 box=27,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=27,36 limit=439,226,51,48 owner=7 paints=#86868AFF role=text size=30 text=GB
        9 TabItem at=490,226,77,48 by=clip drawn focus heard=SNES limit=258,226,765,169 op paints=#EEEEF0FF role=tabitem sideways text=SNES
          10 TextBlock at=502,232,53,36 box=53,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=52.41,36 limit=490,226,77,48 owner=9 paints=#EEEEF0FF role=text size=30 text=SNES
        11 TabItem at=567,226,64,48 by=clip drawn focus heard=N64 limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text=N64
          12 TextBlock at=579,232,40,36 box=40,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=39.54,36 limit=567,226,64,48 owner=11 paints=#86868AFF role=text size=30 text=N64
        13 TabItem at=631,226,110,48 by=clip drawn focus heard=GENESIS limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text=Genesis
          14 TextBlock at=643,232,86,36 box=86,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=85.47,36 limit=631,226,110,48 owner=13 paints=#86868AFF role=text size=30 text=GENESIS
        15 TabItem at=741,226,109,48 by=clip drawn focus heard="SEGA CD" limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text="Sega CD"
          16 TextBlock at=753,232,85,36 box=85,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=84.18,36 limit=741,226,109,48 owner=15 paints=#86868AFF role=text size=30 text="SEGA CD"
        17 TabItem at=850,226,156,48 by=clip drawn focus heard=PLAYSTATION limit=258,226,765,169 paints=#86868AFF role=tabitem sideways text=PlayStation
          18 TextBlock at=862,232,132,36 box=132,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=131.91,36 limit=850,226,156,48 owner=17 paints=#86868AFF role=text size=30 text=PLAYSTATION
        19 StackPanel at=270,284,741,111 by=clip limit=258,276,765,119 role=none
          20 TextBlock at=270,284,741,29 box=741,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Add a SNES cheat" layout=142.18,28.8 limit=258,276,765,119 paints=#9C9CA0FF role=text size=24 text="Add a SNES cheat"
          21 HintText at=270,319,741,24 box=741,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="Accepts: Game Genie, Pro Action Replay." layout=271.12,24 limit=258,276,765,119 paints=#86868AFF role=text size=20 text="Accepts: Game Genie, Pro Action Replay."
          22 Grid at=270,349,741,46 by=clip limit=258,276,765,119 role=none
            23 TextBox at=270,349,332,46 by=clip drawn focus heard="Cheat code" limit=258,276,765,119 name=SNESCodeBox op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              24 ScrollViewer allow="a text box scrolls its own text" at=281,356,310,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                25 TextBlock allow="a text box scrolls its own text" at=281,356,310,32 box=310,32 caps=18.1987 drawn face="Barlow Condensed" heard=Code in=24 layout=43.65,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Code
            26 TextBox at=610,349,324,46 by=clip drawn focus heard="Cheat description" limit=258,276,765,119 name=SNESDescriptionBox op paints=#0E0E10FF,#55555AFF,#D2D2D6FF role=edit
              27 ScrollViewer allow="a text box scrolls its own text" at=621,356,302,32 name=PART_ScrollViewer paints=#D2D2D6FF part role=scrollviewer
                28 TextBlock allow="a text box scrolls its own text" at=621,356,302,32 box=302,32 caps=18.1987 drawn face="Barlow Condensed" heard=Description in=27 layout=101.09,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text=Description
            29 Button at=942,350,69,44 by=clip drawn focus heard=ADD limit=258,276,765,119 name=SNESAddButton op paints=#55555AFF,#9C9CA0FF role=button text=Add
              30 TextBlock at=959,356,35,32 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ADD layout=34.66,31.2 limit=942,350,69,44 owner=29 paints=#9C9CA0FF role=text size=26 text=ADD
      31 TextBlock at=258,407,765,29 box=765,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 cheat(s), 2 on" layout=123.53,28.8 limit=234,214,812.35,426 name=ListHeaderText paints=#9C9CA0FF role=text size=24 text="2 cheat(s), 2 on"
      32 Border at=258,580,765,44 by=panel limit=234,214,812.35,426 role=none
        33 Grid at=258,580,765,44 by=panel limit=234,214,812.35,426 role=none
          34 ButtonBar at=285,580,711,44 by=panel limit=234,214,812.35,426 paints=#9C9CA0FF role=toolbar
            35 Button at=285,580,195,44 by=clip drawn focus heard="CHEAT DATABASE..." limit=285,580,711,44 name=DatabaseButton op paints=#55555AFF,#9C9CA0FF role=button text="Cheat Database..."
              36 TextBlock at=302,586,161,32 box=161,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHEAT DATABASE..." layout=160.06,31.2 limit=285,580,195,44 owner=35 paints=#9C9CA0FF role=text size=26 text="CHEAT DATABASE..."
            37 Button at=488,580,160,44 by=clip drawn focus heard="APPLY CHEATS" limit=285,580,711,44 name=ApplyButton op paints=#55555AFF,#9C9CA0FF role=button text="Apply Cheats"
              38 TextBlock at=505,586,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY CHEATS" layout=125.87,31.2 limit=488,580,160,44 owner=37 paints=#9C9CA0FF role=text size=26 text="APPLY CHEATS"
            39 Button at=656,580,77,44 by=clip drawn focus heard=SAVE limit=285,580,711,44 name=SaveButton op paints=#55555AFF,#9C9CA0FF role=button text=Save
              40 TextBlock at=673,586,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SAVE layout=42.59,31.2 limit=656,580,77,44 owner=39 paints=#9C9CA0FF role=text size=26 text=SAVE
            41 Button at=741,580,105,44 by=clip drawn heard=REMOVE limit=285,580,711,44 name=RemoveButton off paints=#55555AFF,#9C9CA0FF role=button text=Remove
              42 TextBlock at=758,586,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=741,580,105,44 off owner=41 paints=#55555AFF role=text size=26 text=REMOVE
            43 Button at=854,580,142,44 by=clip drawn focus heard="REMOVE ALL" limit=285,580,711,44 name=ClearButton op paints=#55555AFF,#9C9CA0FF role=button text="Remove All"
              44 TextBlock at=871,586,108,32 box=108,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="REMOVE ALL" layout=107.02,31.2 limit=854,580,142,44 owner=43 paints=#9C9CA0FF role=text size=26 text="REMOVE ALL"
      45 LunaTable at=258,444,765,124 by=panel heard="Cheats for this console" limit=234,214,812.35,426 name=CheatsList paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=datagrid
        46 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=270,448,40,29 box=40,29 caps=16.7988 drawn face="Barlow Condensed" heard=On layout=21.12,28.8 paints=#9C9CA0FF role=text size=24 text=On
        47 GridSplitter at=304,448,6,29 by=clip focus heard="Resize On" limit=258,444,765,40 paints=#9C9CA0FF role=thumb
        48 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=310,448,44,29 box=44,29 caps=16.7988 drawn face="Barlow Condensed" heard=Kind layout=36.17,28.8 paints=#9C9CA0FF role=text size=24 text=Kind
        49 GridSplitter at=348,448,6,29 by=clip focus heard="Resize Kind" limit=258,444,765,40 paints=#9C9CA0FF role=thumb
        50 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=354,448,393,29 box=393,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat layout=46.87,28.8 paints=#9C9CA0FF role=text size=24 text=Cheat
        51 GridSplitter at=741,448,6,29 by=clip focus heard="Resize Cheat" limit=258,444,765,40 paints=#9C9CA0FF role=thumb
        52 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=747,448,264,29 box=264,29 caps=16.7988 drawn face="Barlow Condensed" heard=Code layout=40.3,28.8 paints=#9C9CA0FF role=text size=24 text=Code
        53 ScrollViewer at=258,484,765,84 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=258,484,765,84
          54 ListBoxItem at=258,484,765,42 by=fixed-scroll drawn focus heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=53 limit=258,484,765,84 op paints=#333337FF,#9C9CA0FF role=listitem
            55 Grid at=268,488,745,32 by=clip heard="On: Infinite lives, Kind: RAM, Cheat: Infinite lives, Code: CpuBus 0x7E0100 = 0x42" in=53 limit=258,484,765,42 role=none
              56 CheckBox at=268,488,20,32 by=clip drawn focus heard=On in=53 limit=258,484,765,42 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                57 ViewboxContainer at=268,494,20,20 by=clip in=53 limit=268,494,20,20 role=none
              58 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=308,492,44,24 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=53 layout=27.74,24 owner=54 paints=#808080FF role=text size=20 text=RAM
              59 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=352,490,397,29 box=397,29 caps=16.7988 drawn face="Barlow Condensed" heard=Cheat in=53 layout=100.06,28.8 owner=54 paints=#9C9CA0FF role=edit size=24 text="Infinite lives"
              60 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=749,490,264,28 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0100 = 0x42" in=53 layout=264,27.24 owner=54 paints=#808080FF role=text size=20 text="CpuBus 0x7E0100 = 0x42"
          61 ListBoxItem at=258,526,765,42 by=fixed-scroll drawn focus heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=53 limit=258,484,765,84 op paints=#333337FF,#9C9CA0FF role=listitem
            62 Grid at=268,530,745,32 by=clip heard="On: Nine continues, and a description long enough to need the whole row or more of it, Kind: RAM, Cheat: Nine continues, and a description long enough to need the whole row or more of it, Code: CpuBus 0x7E0101 = 0x09" in=53 limit=258,526,765,42 role=none
              63 CheckBox at=268,530,20,32 by=clip drawn focus heard=On in=53 limit=258,526,765,42 paints=#4C9AE8FF,#9C9CA0FF role=checkbox
                64 ViewboxContainer at=268,536,20,20 by=clip in=53 limit=268,536,20,20 role=none
              65 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=308,534,44,24 box=44,24 caps=13.999 drawn face="Barlow Condensed" heard=RAM in=53 layout=27.74,24 owner=61 paints=#808080FF role=text size=20 text=RAM
              66 TableCell allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=352,532,397,29 box=397,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard=Cheat in=53 layout=396.86,28.8 owner=61 paints=#9C9CA0FF role=edit size=24 text="Nine continues, and a description long enough to need the whole row or more of it"
              67 TextBlock allow="a cheat's description or code cut in the table is whole in the panel's footer while its row is chosen" at=749,532,264,28 box=264,28 caps=14.2822 drawn face="Noto Sans Mono" heard="CpuBus 0x7E0101 = 0x09" in=53 layout=264,27.24 owner=61 paints=#808080FF role=text size=20 text="CpuBus 0x7E0101 = 0x09"
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
edge 35 left same
edge 35 right 37
edge 37 up 61
edge 37 down same
edge 37 left 35
edge 37 right 39
edge 39 up 61
edge 39 down same
edge 39 left 37
edge 39 right 43
edge 43 up 61
edge 43 down same
edge 43 left 39
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
edge 51 right 43
edge 54 up 47
edge 54 down 61
edge 54 left same
edge 54 right same
edge 56 up 23
edge 56 down 61
edge 56 left same
edge 56 right 47
edge 61 up 54
edge 61 down 35
edge 61 left same
edge 61 right same
press pageup focus="TextBox GBCodeBox (Cheat code)" tab=GB title="Active Cheats" top=ActiveCheatsWindow
press pagedown focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press first focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press last focus="TextBox SNESCodeBox (Cheat code)" tab=SNES title="Active Cheats" top=ActiveCheatsWindow
press back top=none
