# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresDetail-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case CoresDetail
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 CoresPane at=72,42,1776,1107 by=panel heard=Cores limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 DockPanel at=96,60,1728,69 by=panel limit=0,0,1920,1161 role=none
              6 Border at=1554,60,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                7 TextBox at=1554,60,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  8 ScrollViewer allow="a text box scrolls its own text" at=1570.5,70.5,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    9 TextBlock allow="a text box scrolls its own text" at=1570.5,70.5,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=8 layout=110.16,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search cores"
              10 Border at=96,60,1458,69 by=panel limit=0,0,1920,1161 name=AdaptiveNavigation role=none
                11 WholeRow at=96,60,1458,69 by=panel limit=0,0,1920,1161 paints=#FFFFFFFF role=none
                  12 ToggleButton at=96,61.5,97.5,66 by=panel drawn focus heard=All limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=All
                    13 TextBlock at=121.5,70.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=96,61.5,97.5,66 owner=12 paints=#EEEEF0FF role=text size=26 text=ALL
                  14 ToggleButton at=205.5,61.5,211.5,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=PlayStation
                    15 TextBlock at=231,70.5,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=205.5,61.5,211.5,66 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  16 ToggleButton at=429,61.5,234,66 by=panel drawn focus heard="PlayStation 2" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="PlayStation 2"
                    17 TextBlock at=454.5,70.5,183,48 box=122,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=121.5,31.2 limit=429,61.5,234,66 owner=16 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
                  18 ToggleButton at=675,61.5,115.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=SNES
                    19 TextBlock at=700.5,70.5,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=675,61.5,115.5,66 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
                  20 ToggleButton at=802.5,61.5,183,66 by=panel drawn focus heard=Installed limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Installed
                    21 TextBlock at=828,70.5,132,48 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALLED layout=87.88,31.2 limit=802.5,61.5,183,66 owner=20 paints=#FFFFFFFF role=text size=26 text=INSTALLED
                  22 ToggleButton at=997.5,61.5,183,66 by=panel drawn focus heard=Available limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Available
                    23 TextBlock at=1023,70.5,132,48 box=88,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=AVAILABLE layout=87.59,31.2 limit=997.5,61.5,183,66 owner=22 paints=#FFFFFFFF role=text size=26 text=AVAILABLE
                  24 ToggleButton at=1192.5,61.5,162,66 by=panel drawn focus heard=Updates limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=Updates
                    25 TextBlock at=1218,70.5,111,48 box=74,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATES layout=73.87,31.2 limit=1192.5,61.5,162,66 owner=24 paints=#FFFFFFFF role=text size=26 text=UPDATES
            26 Grid at=96,147,1728,643.5 by=panel limit=0,0,1920,1161 role=none
              27 Border at=96,147,1728,643.5 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
                28 Border at=96,147,1728,643.5 by=panel limit=0,0,1920,1161 role=none
                  29 ScrollViewer at=108,159,1704,619.5 by=panel limit=0,0,1920,1161 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=108,159,1704,619.5
                    30 StackPanel at=108,159,1704,619.5 by=fixed-scroll in=29 limit=108,159,1704,619.5 name=CoresDetail role=none
                      31 TextBlock at=108,159,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)" in=29 layout=550.51,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                      32 TextBlock at=108,211.5,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="PCSX2 Team, libretro · GPLv3 · libretro" in=29 layout=290.66,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="PCSX2 Team, libretro · GPLv3 · libretro"
                      33 TextBlock at=108,264,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Runs: PlayStation 2 · available" in=29 layout=231,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="Runs: PlayStation 2 · available"
                      34 TextBlock at=108,316.5,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58" in=29 layout=833.64,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58"
                      35 TextBlock at=108,369,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded" in=29 layout=723.07,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded"
                      36 TextBlock at=108,421.5,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Notes: (!) needs a BIOS · disc images by path" in=29 layout=345.1,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="Notes: (!) needs a BIOS · disc images by path"
                      37 TextBlock at=108,474,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Not tested by EmuSen." in=29 layout=175.22,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="Not tested by EmuSen."
                      38 TextBlock at=108,526.5,1704,43.5 box=1136,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it." in=29 layout=702.17,28.8 limit=108,159,1704,619.5 paints=#D4D4D4FF role=text size=24 text="Its SHA-256 is known once it is downloaded; nothing downloaded runs until you approve it."
                      39 WrapPanel at=108,579,1704,66 by=fixed-scroll in=29 limit=108,159,1704,619.5 role=none
                        40 Button at=108,579,280.5,66 by=fixed-scroll drawn focus heard="Open Firmware" in=29 limit=108,159,1704,619.5 name=CoresFirmware op paints=#707070FF,#D4D4D4FF role=button text="Open Firmware…"
                          41 TextBlock at=133.5,588,229.5,48 box=153,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPEN FIRMWARE…" in=29 layout=152.98,31.2 limit=108,579,280.5,66 owner=40 paints=#EEEEF0FF role=text size=26 text="OPEN FIRMWARE…"
            42 Border at=96,802.5,1728,144 by=panel limit=0,0,1920,1161 name=AdaptiveActions role=none
              43 WrapPanel at=96,802.5,1728,144 by=panel limit=0,0,1920,1161 role=none
                44 ActionButton at=96,802.5,151.5,66 by=panel drawn focus heard=INSTALL limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Install
                  45 TextBlock at=121.5,811.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=66.22,31.2 limit=96,802.5,151.5,66 owner=44 paints=#D4D4D4FF role=text size=26 text=INSTALL
                46 ActionButton at=265.5,802.5,147,66 by=panel drawn focus heard=UPDATE limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Update
                  47 TextBlock at=291,811.5,96,48 box=64,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=63.62,31.2 limit=265.5,802.5,147,66 owner=46 paints=#D4D4D4FF role=text size=26 text=UPDATE
                48 ActionButton at=430.5,802.5,151.5,66 by=panel drawn focus heard=REMOVE limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Remove
                  49 TextBlock at=456,811.5,100.5,48 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=66.33,31.2 limit=430.5,802.5,151.5,66 owner=48 paints=#D4D4D4FF role=text size=26 text=REMOVE
                50 ActionButton at=600,802.5,190.5,66 by=panel drawn heard="APPROVE…" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Approve…"
                  51 TextBlock at=625.5,811.5,139.5,48 box=93,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=92.09,31.2 limit=600,802.5,190.5,66 off owner=50 paints=#707070FF role=text size=26 text="APPROVE…"
                52 ActionButton at=808.5,802.5,232.5,66 by=panel drawn heard="MAKE DEFAULT" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Make Default"
                  53 TextBlock at=834,811.5,181.5,48 box=121,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=120.87,31.2 limit=808.5,802.5,232.5,66 off owner=52 paints=#707070FF role=text size=26 text="MAKE DEFAULT"
                54 ActionButton at=1059,802.5,181.5,66 by=panel drawn heard="OPTIONS…" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Options…"
                  55 TextBlock at=1084.5,811.5,130.5,48 box=87,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=86.76,31.2 limit=1059,802.5,181.5,66 off owner=54 paints=#707070FF role=text size=26 text="OPTIONS…"
                56 ActionButton at=1258.5,802.5,138,66 by=panel drawn focus heard=VERIFY limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Verify
                  57 TextBlock at=1284,811.5,87,48 box=58,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=57.02,31.2 limit=1258.5,802.5,138,66 owner=56 paints=#D4D4D4FF role=text size=26 text=VERIFY
                58 ActionButton at=1414.5,802.5,307.5,66 by=panel drawn focus heard="CHECK FOR UPDATES" limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text="Check for Updates"
                  59 TextBlock at=1440,811.5,256.5,48 box=171,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=170.85,31.2 limit=1414.5,802.5,307.5,66 owner=58 paints=#D4D4D4FF role=text size=26 text="CHECK FOR UPDATES"
                60 ActionButton at=96,880.5,120,66 by=panel drawn focus heard=MARK limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Mark
                  61 TextBlock at=121.5,889.5,69,48 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=96,880.5,120,66 owner=60 paints=#D4D4D4FF role=text size=26 text=MARK
            62 Border at=72,958.5,1776,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
              63 Grid at=78,964.5,1764,121.5 by=panel limit=0,0,1920,1161 role=none
                64 Grid at=78,964.5,1764,43.5 by=panel limit=0,0,1920,1161 role=none
                  65 TextBlock at=90,964.5,1752,43.5 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
                66 StackPanel at=1063.5,1020,778.5,66 by=panel limit=0,0,1920,1161 role=none
                  67 Button at=1063.5,1020,153,66 by=panel drawn heard="Apply the marked changes" limit=0,0,1920,1161 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    68 TextBlock at=1089,1029,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=1063.5,1020,153,66 off owner=67 paints=#707070FF role=text size=26 text="APPLY…"
                  69 StackPanel at=1234.5,1020,607.5,66 by=panel limit=0,0,1920,1161 role=none
                    70 ProgressBar at=1234.5,1042.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1161 paints=#333337FF,#4C9AE8FF role=progressbar
                    71 TextBlock at=1546.5,1032,295.5,43.5 box=197,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=196.94,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="/Cores 11 MB · 410 GB free"
            72 HintBar at=96,1104,1728,33 by=panel heard="Select, Back, Section, Apply…, Install, Check for Updates, Mark" limit=0,0,1920,1161 role=group
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
edge 40 right 48
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
press back focus="GridSplitter  (Resize Mark)" top=CoresWindow
