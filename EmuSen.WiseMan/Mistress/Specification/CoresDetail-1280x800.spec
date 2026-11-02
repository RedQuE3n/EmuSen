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
            5 DockPanel at=64,40,1152,46 by=panel limit=0,0,1280,800 role=none
              6 Border at=1036,40,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                7 TextBox at=1036,40,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1047,47,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1047,47,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=64,40,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                11 WholeRow at=64,40,972,46 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
                  12 ToggleButton at=64,41,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    13 TextBlock at=80,49,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,41,66,44 owner=12 paints=#EEEEF0FF role=text size=20 text=ALL
                  14 ToggleButton at=138,41,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    15 TextBlock at=154,49,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=138,41,146,44 owner=14 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  16 ToggleButton at=292,41,160,44 by=panel drawn focus heard="PlayStation 2" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="PlayStation 2"
                    17 TextBlock at=308,49,128,24 box=128,24 by=clip caps=13.999 drawn face=Barlow heard="PLAYSTATION 2" layout=127.7,24 limit=292,41,160,44 owner=16 paints=#D4D4D4FF role=text size=20 text="PLAYSTATION 2"
                  18 ToggleButton at=460,41,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    19 TextBlock at=476,49,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=460,41,79,44 owner=18 paints=#D4D4D4FF role=text size=20 text=SNES
                  20 ToggleButton at=547,41,127,44 by=panel drawn focus heard=Installed limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Installed
                    21 TextBlock at=563,49,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=INSTALLED layout=94.8,24 limit=547,41,127,44 owner=20 paints=#D4D4D4FF role=text size=20 text=INSTALLED
                  22 ToggleButton at=682,41,127,44 by=panel drawn focus heard=Available limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Available
                    23 TextBlock at=698,49,95,24 box=95,24 by=clip caps=13.999 drawn face=Barlow heard=AVAILABLE layout=94.54,24 limit=682,41,127,44 owner=22 paints=#D4D4D4FF role=text size=20 text=AVAILABLE
                  24 ToggleButton at=817,41,112,44 by=panel drawn focus heard=Updates limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=Updates
                    25 TextBlock at=833,49,80,24 box=80,24 by=clip caps=13.999 drawn face=Barlow heard=UPDATES layout=79.8,24 limit=817,41,112,44 owner=24 paints=#D4D4D4FF role=text size=20 text=UPDATES
            26 Grid at=64,98,1152,468 by=panel limit=0,0,1280,800 role=none
              27 Border at=64,98,1152,468 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                28 Border at=64,98,1152,468 by=panel limit=0,0,1280,800 role=none
                  29 ScrollViewer at=72,106,1136,452 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=72,106,1136,452
                    30 StackPanel at=72,106,1136,452 by=fixed-scroll in=29 limit=72,106,1136,452 name=CoresDetail role=none
                      31 TextBlock at=72,106,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)" in=29 layout=550.51,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                      32 TextBlock at=72,141,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="PCSX2 Team, libretro · GPLv3 · libretro" in=29 layout=290.66,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="PCSX2 Team, libretro · GPLv3 · libretro"
                      33 TextBlock at=72,176,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Runs: PlayStation 2 · available" in=29 layout=231,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="Runs: PlayStation 2 · available"
                      34 TextBlock at=72,211,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58" in=29 layout=833.64,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58"
                      35 TextBlock at=72,246,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded" in=29 layout=723.07,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded"
                      36 TextBlock at=72,281,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Notes: (!) needs a BIOS · disc images by path" in=29 layout=345.1,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="Notes: (!) needs a BIOS · disc images by path"
                      37 TextBlock at=72,316,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Not tested by EmuSen." in=29 layout=175.22,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="Not tested by EmuSen."
                      38 TextBlock at=72,351,1136,29 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=29 layout=702.17,28.8 limit=72,106,1136,452 paints=#D4D4D4FF role=text size=24 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      39 WrapPanel at=72,386,1136,44 by=fixed-scroll in=29 limit=72,106,1136,452 role=none
                        40 Button at=72,386,187,44 by=fixed-scroll drawn focus heard="Open Firmware" in=29 limit=72,106,1136,452 name=CoresFirmware op paints=#707070FF,#D4D4D4FF role=button text="Open Firmware…"
                          41 TextBlock at=89,392,153,32 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPEN FIRMWARE…" in=29 layout=152.98,31.2 limit=72,386,187,44 owner=40 paints=#EEEEF0FF role=text size=26 text="OPEN FIRMWARE…"
            42 Border at=64,574,1152,96 by=panel limit=0,0,1280,800 name=AdaptiveActions role=none
              43 WrapPanel at=64,574,1152,96 by=panel limit=0,0,1280,800 role=none
                44 ActionButton at=64,574,101,44 by=panel drawn focus heard=INSTALL limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  45 TextBlock at=81,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=64,574,101,44 owner=44 paints=#D4D4D4FF role=text size=26 text=INSTALL
                46 ActionButton at=177,574,98,44 by=panel drawn focus heard=UPDATE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  47 TextBlock at=194,580,64,32 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=177,574,98,44 owner=46 paints=#D4D4D4FF role=text size=26 text=UPDATE
                48 ActionButton at=287,574,101,44 by=panel drawn focus heard=REMOVE limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  49 TextBlock at=304,580,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=287,574,101,44 owner=48 paints=#D4D4D4FF role=text size=26 text=REMOVE
                50 ActionButton at=400,574,127,44 by=panel drawn heard="APPROVE…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  51 TextBlock at=417,580,93,32 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=400,574,127,44 off owner=50 paints=#707070FF role=text size=26 text="APPROVE…"
                52 ActionButton at=539,574,155,44 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  53 TextBlock at=556,580,121,32 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=539,574,155,44 off owner=52 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                54 ActionButton at=706,574,121,44 by=panel drawn heard="OPTIONS…" limit=0,0,1280,800 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  55 TextBlock at=723,580,87,32 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=706,574,121,44 off owner=54 paints=#707070FF role=text size=26 text="OPTIONS…"
                56 ActionButton at=839,574,92,44 by=panel drawn focus heard=VERIFY limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  57 TextBlock at=856,580,58,32 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=839,574,92,44 owner=56 paints=#D4D4D4FF role=text size=26 text=VERIFY
                58 ActionButton at=943,574,205,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  59 TextBlock at=960,580,171,32 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=943,574,205,44 owner=58 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                60 ActionButton at=64,626,80,44 by=panel drawn focus heard=MARK limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  61 TextBlock at=81,632,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=64,626,80,44 owner=60 paints=#D4D4D4FF role=text size=26 text=MARK
            62 Border at=48,678,1184,74 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              63 Grid at=52,682,1176,66 by=panel limit=0,0,1280,800 role=none
                64 Grid at=52,682,1176,29 by=panel limit=0,0,1280,800 role=none
                  65 TextBlock at=60,682,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                66 StackPanel at=823,719,405,29 by=panel limit=0,0,1280,800 role=none
                  67 StackPanel at=823,719,405,29 by=panel limit=0,0,1280,800 role=none
                    68 ProgressBar at=823,727,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    69 TextBlock at=1031,719,197,29 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            70 HintBar at=64,760,1152,24 by=panel heard="Select, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1280,800 role=group
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
edge 48 up 16
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
