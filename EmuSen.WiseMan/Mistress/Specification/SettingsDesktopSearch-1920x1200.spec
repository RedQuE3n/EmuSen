# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsDesktopSearch-1920x1200
size 1920 1200
kind window
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 0
frame 0,0,960,640
case SettingsDesktopSearch
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
window 960 640
tree
  0 Panel at=0,0,960,640 by=panel limit=0,0,960,640 role=none
    1 DockPanel at=0,0,960,640 by=panel limit=0,0,960,640 paints=#1B1B1EFF role=none
      2 Border at=0,0,201,640 by=panel limit=0,0,960,640 paints=#333337FF role=none
        3 SourceList at=0,12,200,628 by=panel focus limit=0,0,960,640 name=SettingsSourceList paints=#1B1B1EFF,#FFFFFFFF role=list
          4 ScrollViewer at=0,12,200,628 name=PART_Scroll overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=0,12,200,628
            5 SourceListGroupView at=0,12,200,144 by=fixed-scroll in=4 limit=0,12,200,628 role=group
              6 SourceListRow at=6,12,188,24 by=fixed-scroll heard=Controllers in=4 limit=0,12,200,628 role=listitem
                7 Grid at=26,12,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  8 TextBlock at=26,16,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Controllers in=4 layout=59.77,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Controllers
              9 SourceListRow at=6,36,188,24 by=fixed-scroll heard=Display in=4 limit=0,12,200,628 paints=#4C9AE8FF role=listitem
                10 Grid at=26,36,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  11 TextBlock at=26,40,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Display in=4 layout=39.94,15.6 limit=0,12,200,628 paints=#0E0E10FF role=text size=13 text=Display
              12 SourceListRow at=6,60,188,24 by=fixed-scroll heard=Sound in=4 limit=0,12,200,628 role=listitem
                13 Grid at=26,60,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  14 TextBlock at=26,64,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Sound in=4 layout=34.92,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Sound
              15 SourceListRow at=6,84,188,24 by=fixed-scroll heard=Consoles in=4 limit=0,12,200,628 role=listitem
                16 Grid at=26,84,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  17 TextBlock at=26,88,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Consoles in=4 layout=49.32,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Consoles
              18 SourceListRow at=6,108,188,24 by=fixed-scroll heard=Library in=4 limit=0,12,200,628 role=listitem
                19 Grid at=26,108,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  20 TextBlock at=26,112,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Library in=4 layout=39.23,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Library
              21 SourceListRow at=6,132,188,24 by=fixed-scroll heard=EmuSen in=4 limit=0,12,200,628 role=listitem
                22 Grid at=26,132,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  23 TextBlock at=26,136,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=EmuSen in=4 layout=45.84,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=EmuSen
      24 DockPanel at=225,8,711,608 by=panel limit=0,0,960,640 role=none
        25 DockPanel at=225,8,711,48 by=panel limit=0,0,960,640 name=SettingsHeader role=none
          26 TextBox at=716,16,220,32 by=panel drawn focus heard="Search settings" limit=0,0,960,640 name=SettingsSearch op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit text=volume
            27 ScrollViewer at=727,23,202,19 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
          28 TextBlock at=225,22,491,20 box=491,20 by=panel caps=11.1992 drawn face="Barlow Condensed" heard=DISPLAY layout=41.25,19.2 limit=0,0,960,640 name=SettingsTitle paints=#EEEEF0FF role=text size=16 text=DISPLAY
        29 Border at=660,64,276,92 by=panel limit=0,0,960,640 name=SettingsPane paints=#26262BFF role=none
          30 StackPanel at=676,80,244,60 by=panel limit=0,0,960,640 role=none
            31 Border at=676,80,244,16 by=panel limit=0,0,960,640 name=SettingsMeter paints=#707070FF role=none
              32 Border at=676,80,244,16 by=panel limit=0,0,960,640 paints=#4C9AE8FF role=none
            33 TextBlock at=676,108,244,32 box=244,32 by=panel caps=9.0994 drawn face=Barlow heard="The game's volume. On the handheld this row defers to the device's quick menu." layout=222.4,31.2 limit=0,0,960,640 name=SettingsLine paints=#D4D4D4FF role=text size=13 text="The game's volume. On the handheld this row defers to the device's quick menu."
        34 ScrollViewer at=225,56,419,560 by=panel limit=0,0,960,640 overflow=0 paints=#FFFFFFFF role=scrollviewer viewport=225,56,419,560
          35 StackPanel at=225,56,419,560 by=fixed-scroll heard=Display in=34 limit=225,56,419,560 name=SettingsRows role=none
            36 Border at=225,56,419,48 by=fixed-scroll focus heard="Volume, 100%" in=34 limit=225,56,419,560 name=SettingsRow.sound/volume paints=#26262BFF,#333337FF role=none
              37 DockPanel at=233,60,403,39 by=fixed-scroll in=34 limit=225,56,419,560 role=none
                38 StackPanel at=514,65,114,29 by=fixed-scroll in=34 limit=225,56,419,560 role=none
                  39 Button at=514,65,25,29 by=fixed-scroll drawn focus heard="Volume down" in=34 limit=225,56,419,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="−"
                    40 AccessText at=523,71,7,16 box=7,16 by=clip caps=9.0994 drawn face=Barlow in=34 layout=6.19,15.6 limit=514,65,25,29 owner=39 paints=#D4D4D4FF role=none size=13 text="−"
                  41 TextBlock at=543,71,56,16 box=56,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=100% in=34 layout=27.74,15.6 limit=225,56,419,560 paints=#FFFFFFFF role=text size=13 text=100%
                  42 Button at=603,65,25,29 by=fixed-scroll drawn focus heard="Volume up" in=34 limit=225,56,419,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=+
                    43 AccessText at=612,71,7,16 box=7,16 by=clip caps=9.0994 drawn face=Barlow in=34 layout=6.19,15.6 limit=603,65,25,29 owner=42 paints=#D4D4D4FF role=none size=13 text=+
                44 StackPanel at=233,64,265,30 by=fixed-scroll in=34 limit=225,56,419,560 role=none
                  45 TextBlock at=233,64,265,16 box=265,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Volume in=34 layout=40.62,15.6 limit=225,56,419,560 paints=#D4D4D4FF role=text size=13 text=Volume
                  46 TextBlock at=233,80,265,14 box=265,14 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="The game's volume." in=34 layout=91.33,13.2 limit=225,56,419,560 paints=#8D8D91FF role=text size=11 text="The game's volume."
            47 Border at=225,104,419,52 by=fixed-scroll focus heard="Sound volume, 60%" in=34 limit=225,56,419,560 name=SettingsRow.sound/soundvolume paints=#333337FF role=none
              48 DockPanel at=233,108,403,43 by=fixed-scroll in=34 limit=225,56,419,560 role=none
                49 StackPanel at=514,115,114,29 by=fixed-scroll in=34 limit=225,56,419,560 role=none
                  50 Button at=514,115,25,29 by=fixed-scroll drawn focus heard="Sound volume down" in=34 limit=225,56,419,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="−"
                    51 AccessText at=523,121,7,16 box=7,16 by=clip caps=9.0994 drawn face=Barlow in=34 layout=6.19,15.6 limit=514,115,25,29 owner=50 paints=#D4D4D4FF role=none size=13 text="−"
                  52 TextBlock at=543,121,56,16 box=56,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=60% in=34 layout=23.01,15.6 limit=225,56,419,560 paints=#FFFFFFFF role=text size=13 text=60%
                  53 Button at=603,115,25,29 by=fixed-scroll drawn focus heard="Sound volume up" in=34 limit=225,56,419,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text=+
                    54 AccessText at=612,121,7,16 box=7,16 by=clip caps=9.0994 drawn face=Barlow in=34 layout=6.19,15.6 limit=603,115,25,29 owner=53 paints=#D4D4D4FF role=none size=13 text=+
                55 StackPanel at=233,108,265,43 by=fixed-scroll in=34 limit=225,56,419,560 role=none
                  56 TextBlock at=233,108,265,16 box=265,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard="Sound volume" in=34 layout=77.26,15.6 limit=225,56,419,560 paints=#D4D4D4FF role=text size=13 text="Sound volume"
                  57 TextBlock at=233,124,265,27 box=265,27 by=fixed-scroll caps=7.6995 drawn face=Barlow heard="How loud the navigation sounds are, beside the game's own volume." in=34 layout=251.85,26.4 limit=225,56,419,560 paints=#8D8D91FF role=text size=11 text="How loud the navigation sounds are, beside the game's own volume."
focus none
