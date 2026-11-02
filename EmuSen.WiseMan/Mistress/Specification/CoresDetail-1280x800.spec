# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case CoresDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 CoresPane at=48,28,1184,764 by=panel heard=Cores limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 DockPanel at=64,40,1152,44 by=panel limit=0,0,1280,800 role=none
              6 Border at=1036,40,180,44 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,44 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,30 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,50,158,24 box=158,24 caps=13.999 drawn face=Barlow heard="Search cores" in=8 layout=110.28,24 name=PART_Placeholder paints=#8D8D91FF part role=text size=20 text="Search cores"
              10 Border at=64,40,972,44 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,44 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
                  12 ToggleButton at=64,40,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=80,48,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,40,66,44 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=138,40,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=154,48,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,40,146,44 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=292,40,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=308,48,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,40,160,44 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=460,40,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=476,48,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,40,79,44 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=547,40,127,44 by=panel drawn focus heard=Installed limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=563,48,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=INSTALLED layout=94.8,24 limit=547,40,127,44 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=682,40,127,44 by=panel drawn focus heard=Available limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=698,48,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=AVAILABLE layout=94.54,24 limit=682,40,127,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=817,40,112,44 by=panel drawn focus heard=Updates limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=833,48,80,24 box=80,24 by=clip caps=13.999 drawn face=Barlow heard=UPDATES layout=79.8,24 limit=817,40,112,44 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=64,96,1152,471 by=panel limit=0,0,1280,800 role=none
              27 Border at=64,96,1152,471 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                28 Border at=64,96,1152,471 by=panel limit=0,0,1280,800 role=none
                  29 ScrollViewer at=72,104,1136,455 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=72,104,1136,455
                    30 StackPanel at=72,104,1136,455 by=fixed-scroll in=29 limit=72,104,1136,455 name=CoresDetail role=none
                      31 TextBlock at=72,104,1136,27 box=1136,27 by=fixed-scroll caps=16.0864 drawn face="Barlow Medium" heard="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)" in=29 layout=670.82,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                      32 TextBlock at=72,137,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="PCSX2 Team, libretro · GPLv3 · libretro" in=29 layout=345.91,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="PCSX2 Team, libretro · GPLv3 · libretro"
                      33 TextBlock at=72,170,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Runs: PlayStation 2 · available" in=29 layout=275,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="Runs: PlayStation 2 · available"
                      34 TextBlock at=72,203,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58" in=29 layout=1005,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58"
                      35 TextBlock at=72,236,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded" in=29 layout=859.67,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded"
                      36 TextBlock at=72,269,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Notes: (!) needs a BIOS · disc images by path" in=29 layout=404.62,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="Notes: (!) needs a BIOS · disc images by path"
                      37 TextBlock at=72,302,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Not tested by EmuSen." in=29 layout=210.43,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="Not tested by EmuSen."
                      38 TextBlock at=72,335,1136,27 box=1136,27 by=fixed-scroll caps=15.3989 drawn face=Barlow heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=29 layout=831.78,26.4 limit=72,104,1136,455 paints=#D4D4D4FF role=text size=22 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      39 WrapPanel at=72,368,1136,44 by=fixed-scroll in=29 limit=72,104,1136,455 role=none
                        40 Button at=72,368,246,44 by=fixed-scroll drawn focus heard="Open Firmware" in=29 limit=72,104,1136,455 name=CoresFirmware op paints=#707070FF,#D4D4D4FF role=button text="Open Firmware…"
                          41 TextBlock at=89,374,212,32 box=212,32 by=clip caps=18.1987 drawn face=Barlow heard="OPEN FIRMWARE…" in=29 layout=211.98,31.2 limit=72,368,246,44 owner=40 paints=#EEEEF0FF role=text size=26 text="OPEN FIRMWARE…"
            42 Border at=64,575,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              43 WrapPanel at=64,575,1152,96 by=panel limit=0,0,1280,800 role=none
                44 ActionButton at=64,575,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  45 TextBlock at=81,581,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,575,101,44 owner=44 paints=#D4D4D4FF role=text size=26 text=INSTALL
                46 ActionButton at=177,575,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  47 TextBlock at=194,581,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,575,98,44 owner=46 paints=#D4D4D4FF role=text size=26 text=UPDATE
                48 ActionButton at=287,575,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  49 TextBlock at=304,581,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,575,101,44 owner=48 paints=#D4D4D4FF role=text size=26 text=REMOVE
                50 ActionButton at=400,575,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  51 TextBlock at=417,581,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,575,127,44 off owner=50 paints=#707070FF role=text size=26 text="APPROVE…"
                52 ActionButton at=539,575,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  53 TextBlock at=556,581,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,575,155,44 off owner=52 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                54 ActionButton at=706,575,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  55 TextBlock at=723,581,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,575,121,44 off owner=54 paints=#707070FF role=text size=26 text="OPTIONS…"
                56 ActionButton at=839,575,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  57 TextBlock at=856,581,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,575,92,44 owner=56 paints=#D4D4D4FF role=text size=26 text=VERIFY
                58 ActionButton at=943,575,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  59 TextBlock at=960,581,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,575,205,44 owner=58 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                60 ActionButton at=64,627,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  61 TextBlock at=81,633,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,627,80,44 owner=60 paints=#D4D4D4FF role=text size=26 text=MARK
            62 Border at=48,679,1184,67 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              63 Grid at=52,683,1176,59 by=panel limit=0,0,1280,800 role=none
                64 Grid at=52,683,1176,27 by=panel limit=0,0,1280,800 role=none
                  65 TextBlock at=60,683,1168,27 box=1168,27 by=panel caps=15.3989 drawn face=Barlow heard="Nothing marked" layout=149.56,26.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=22 text="Nothing marked"
                66 StackPanel at=813,718,415,24 by=panel limit=0,0,1280,800 role=none
                  67 StackPanel at=813,718,415,24 by=panel limit=0,0,1280,800 role=none
                    68 ProgressBar at=813,723,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    69 TextBlock at=1021,718,207,24 box=207,24 by=panel caps=13.999 drawn face=Barlow heard="/Cores 11 MB · 410 GB free" layout=206.6,24 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=20 text="/Cores 11 MB · 410 GB free"
            70 Border at=48,746,1184,46 by=panel limit=0,0,1280,800 name=AdaptiveHelpBar paints=#707070FF role=none
              71 HintBar at=96,757,1088,29 by=panel heard="Select, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,800 role=group
focus 40
edge 7 up same
edge 7 down 58
edge 7 left 24
edge 7 right same
edge 12 up same
edge 12 down 40
edge 12 left same
edge 12 right 14
edge 14 up same
edge 14 down 40
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 48
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 40
edge 18 left 16
edge 18 right 20
edge 20 up same
edge 20 down 40
edge 20 left 18
edge 20 right 22
edge 22 up same
edge 22 down 40
edge 22 left 20
edge 22 right 24
edge 24 up same
edge 24 down 56
edge 24 left 22
edge 24 right 7
edge 40 up 12
edge 40 down 44
edge 40 left same
edge 40 right 20
edge 44 up 40
edge 44 down 60
edge 44 left same
edge 44 right 46
edge 46 up 40
edge 46 down 60
edge 46 left 44
edge 46 right 48
edge 48 up 40
edge 48 down 60
edge 48 left 46
edge 48 right 56
edge 56 up 24
edge 56 down 60
edge 56 left 48
edge 56 right 58
edge 58 up 7
edge 58 down 60
edge 58 left 56
edge 58 right same
edge 60 up 44
edge 60 down same
edge 60 left same
edge 60 right 46
press pageup top=CoresWindow
press pagedown top=CoresWindow
press first focus="TextBox Search cores (Search)" top=CoresWindow
press last focus="TextBox Search cores (Search)" top=CoresWindow
press back focus="ListBoxItem  (Mark: not marked, Core: Sapphire, State: built in)" top=CoresWindow
