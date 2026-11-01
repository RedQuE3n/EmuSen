# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state CoresDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Cores
case CoresDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 CoresPane at=258,136,765,536 by=panel heard=Cores limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search cores" in=6 layout=115.18,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search cores"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,66,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=274,149,66,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=348,149,149,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                13 TextBlock at=365,155,115,32 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=348,149,149,44 owner=12 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              14 ToggleButton at=505,149,164,44 by=panel drawn focus heard="PlayStation 2" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="PlayStation 2"
                15 TextBlock at=522,155,130,32 box=130,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=129.97,31.2 limit=505,149,164,44 owner=14 paints=#FFFFFFFF role=text size=26 text="PLAYSTATION 2"
              16 ToggleButton at=677,149,80,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                17 TextBlock at=694,155,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=677,149,80,44 owner=16 paints=#FFFFFFFF role=text size=26 text=SNES
        18 Grid at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
          19 Border at=274,206,733,227 by=panel limit=234,124,812.35,548 name=AdaptiveDetail role=none
            20 Border at=274,206,733,227 by=panel limit=234,124,812.35,548 role=none
              21 ScrollViewer at=282,214,717,211 by=panel fades=b limit=234,124,812.35,548 overflow=200 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=282,214,717,211
                22 StackPanel at=282,214,717,411 by=scroll in=21 limit=282,214,717,211 name=CoresDetail paints=#55555AFF,#9C9CA0FF role=none seen=282,214,717,211
                  23 TextBlock at=282,214,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)" in=21 layout=578.16,28.8 limit=282,214,717,211 paints=#9C9CA0FF role=text size=24 text="Sony - PlayStation 2 (LRPS2, the PCSX2 libretro core with its long name)"
                  24 TextBlock at=282,249,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="PCSX2 Team, libretro · GPLv3 · libretro" in=21 layout=305.5,28.8 limit=282,214,717,211 paints=#9C9CA0FF role=text size=24 text="PCSX2 Team, libretro · GPLv3 · libretro"
                  25 TextBlock at=282,284,717,29 box=717,29 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Runs: PlayStation 2 · available" in=21 layout=242.11,28.8 limit=282,214,717,211 paints=#9C9CA0FF role=text size=24 text="Runs: PlayStation 2 · available"
                  26 TextBlock at=282,319,717,58 box=717,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58" in=21 layout=687.26,57.6 limit=282,214,717,211 paints=#9C9CA0FF role=text size=24 text="From https://buildbot.libretro.com/nightly/linux/x86_64/latest/pcsx2_libretro.so.zip, index CRC-32 3a224a58"
                  27 TextBlock at=282,383,717,58 box=717,58 by=scroll caps=16.7988 drawn face="Barlow Condensed" heard="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded" in=21 layout=651.96,57.6 limit=282,214,717,211 paints=#9C9CA0FF role=text seen=282,383,717,42 size=24 text="Firmware: PlayStation 2 BIOS (pcsx2/bios/ps2.bin), required; your own file, never downloaded"
        28 Border at=274,441,733,96 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          29 WrapPanel at=274,441,733,96 by=panel limit=234,124,812.35,548 role=none
            30 ActionButton at=274,441,105,44 by=panel drawn focus heard=INSTALL limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Install
              31 TextBlock at=291,447,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=INSTALL layout=70.23,31.2 limit=274,441,105,44 owner=30 paints=#9C9CA0FF role=text size=26 text=INSTALL
            32 ActionButton at=391,441,101,44 by=panel drawn focus heard=UPDATE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Update
              33 TextBlock at=408,447,67,32 box=67,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=UPDATE layout=66.3,31.2 limit=391,441,101,44 owner=32 paints=#9C9CA0FF role=text size=26 text=UPDATE
            34 ActionButton at=504,441,105,44 by=panel drawn focus heard=REMOVE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Remove
              35 TextBlock at=521,447,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REMOVE layout=70.56,31.2 limit=504,441,105,44 owner=34 paints=#9C9CA0FF role=text size=26 text=REMOVE
            36 ActionButton at=621,441,130,44 by=panel drawn heard="APPROVE…" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Approve…"
              37 TextBlock at=638,447,96,32 box=96,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPROVE…" layout=95.76,31.2 limit=621,441,130,44 off owner=36 paints=#55555AFF role=text size=26 text="APPROVE…"
            38 ActionButton at=763,441,162,44 by=panel drawn heard="MAKE DEFAULT" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Make Default"
              39 TextBlock at=780,447,128,32 box=128,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE DEFAULT" layout=127.82,31.2 limit=763,441,162,44 off owner=38 paints=#55555AFF role=text size=26 text="MAKE DEFAULT"
            40 ActionButton at=274,493,126,44 by=panel drawn heard="OPTIONS…" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Options…"
              41 TextBlock at=291,499,92,32 box=92,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="OPTIONS…" layout=91.68,31.2 limit=274,493,126,44 off owner=40 paints=#55555AFF role=text size=26 text="OPTIONS…"
            42 ActionButton at=412,493,96,44 by=panel drawn focus heard=VERIFY limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Verify
              43 TextBlock at=429,499,62,32 box=62,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=VERIFY layout=61.2,31.2 limit=412,493,96,44 owner=42 paints=#9C9CA0FF role=text size=26 text=VERIFY
            44 ActionButton at=520,493,214,44 by=panel drawn focus heard="CHECK FOR UPDATES" limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text="Check for Updates"
              45 TextBlock at=537,499,180,32 box=180,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CHECK FOR UPDATES" layout=179.58,31.2 limit=520,493,214,44 owner=44 paints=#9C9CA0FF role=text size=26 text="CHECK FOR UPDATES"
            46 ActionButton at=746,493,82,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              47 TextBlock at=763,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=746,493,82,44 owner=46 paints=#9C9CA0FF role=text size=26 text=MARK
        48 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          49 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            50 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              51 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
            52 StackPanel at=491,586,528,44 by=panel limit=234,124,812.35,548 role=none
              53 Button at=491,586,105,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                54 TextBlock at=508,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=491,586,105,44 off owner=53 paints=#55555AFF role=text size=26 text="APPLY…"
              55 StackPanel at=608,586,411,44 by=panel limit=234,124,812.35,548 role=none
                56 ProgressBar at=608,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                57 TextBlock at=816,594,203,29 box=203,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="/Cores 11 MB · 410 GB free" layout=202.22,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="/Cores 11 MB · 410 GB free"
        58 HintBar at=274,642,733,22 by=panel heard="Select, Back, Section, Apply…, Install, Check for Updates, Mark" limit=234,124,812.35,548 role=group
focus 5
edge 5 up same
edge 5 down 34
edge 5 left 16
edge 5 right same
edge 10 up same
edge 10 down 30
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 30
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 34
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 44
edge 16 left 14
edge 16 right 5
edge 30 up 10
edge 30 down 42
edge 30 left same
edge 30 right 32
edge 32 up 12
edge 32 down 42
edge 32 left 30
edge 32 right 34
edge 34 up 14
edge 34 down 44
edge 34 left 32
edge 34 right 46
edge 42 up 32
edge 42 down path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0
edge 42 left 30
edge 42 right 44
edge 44 up 34
edge 44 down path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0
edge 44 left 42
edge 44 right 46
edge 46 up 34
edge 46 down path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0
edge 46 left 44
edge 46 right 5
edge path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0 up 10
edge path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0 down 30
edge path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/0/1/1/0/0/0/0/0/8/0 right 34
press pageup focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press pagedown focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press first focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press last focus="TextBox Search cores (Search)" title=Cores top=CoresWindow
press back focus="GridSplitter  (Resize Mark)" title=Cores top=CoresWindow
