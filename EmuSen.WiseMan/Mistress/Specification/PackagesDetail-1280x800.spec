# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesDetail-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 233.65,24,812.7,668
title Packages
case PackagesDetail
entered down
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#9C9CA0FF role=none seen=234,124,812.35,548
    1 PackagesPane at=258,136,765,536 by=panel heard=Packages limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#050507FF,#0E0E10FF,#55555AFF,#EEEEF0FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#EEEEF0FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=124.77,31.2 name=PART_Placeholder paints=#86868AFF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,66,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,32,32 box=32,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=31.25,31.2 limit=274,149,66,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=348,149,69,44 by=panel drawn focus heard=NES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=365,155,35,32 box=35,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=34.79,31.2 limit=348,149,69,44 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=425,149,149,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=442,155,115,32 box=115,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=114.32,31.2 limit=425,149,149,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=582,149,107,44 by=panel drawn focus heard="Sega CD" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=599,155,73,32 box=73,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=72.96,31.2 limit=582,149,107,44 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=697,149,80,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=714,155,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=45.42,31.2 limit=697,149,80,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          21 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveDetail role=none
            22 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              23 ScrollViewer at=282,214,717,263 by=panel fades=b limit=234,124,812.35,548 overflow=512 paints=#55555AFF,#9C9CA0FF,#FFFFFFFF role=scrollviewer viewport=282,214,717,263
                24 PackageDetail at=282,214,717,775 by=scroll in=23 limit=282,214,717,263 name=PackagesDetail paints=#55555AFF,#86868AFF,#9C9CA0FF role=none seen=282,214,717,263
                  25 FieldRow at=282,214,717,91 by=scroll heard=Package in=23 limit=282,214,717,263 paints=#9C9CA0FF role=group
                    26 TextBlock at=282,214,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=23 layout=69.48,28.8 limit=282,214,717,91 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Package
                    27 TextBlock at=282,247,717,58 box=717,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read." in=23 layout=675.02,57.6 limit=282,214,717,91 paints=#9C9CA0FF role=text size=24 text="Not a package yet. Making one keeps everything EmuSen holds for the game in one folder, and the library is only read."
                  28 FieldRow at=282,315,717,91 by=scroll heard=Game in=23 limit=282,214,717,263 paints=#9C9CA0FF role=group
                    29 TextBlock at=282,315,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Game in=23 layout=46.06,28.8 limit=282,315,717,91 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=Game
                    30 TextBlock at=282,348,717,58 box=717,58 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)" in=23 layout=674.42,57.6 limit=282,315,717,91 paints=#9C9CA0FF role=text size=24 text="psx/Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3).cue in the library (PlayStation)"
                  31 FieldRow at=282,416,717,62 by=scroll heard=State in=23 limit=282,214,717,263 paints=#9C9CA0FF role=group seen=282,416,717,61
                    32 TextBlock at=282,416,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=State in=23 layout=42.48,28.8 limit=282,416,717,62 name=PART_Label paints=#9C9CA0FF part role=text size=24 text=State
                    33 TextBlock at=282,449,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not packaged" in=23 layout=112.3,28.8 limit=282,416,717,62 paints=#9C9CA0FF role=text seen=282,449,717,28 size=24 text="Not packaged"
        34 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          35 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            36 ActionButton at=274,493,82,44 by=panel drawn focus heard=MAKE limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Make
              37 TextBlock at=291,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=47.11,31.2 limit=274,493,82,44 owner=36 paints=#9C9CA0FF role=text size=26 text=MAKE
            38 ActionButton at=368,493,93,44 by=panel drawn focus heard=CHECK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Check
              39 TextBlock at=385,499,59,32 box=59,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=58.19,31.2 limit=368,493,93,44 owner=38 paints=#9C9CA0FF role=text size=26 text=CHECK
            40 ActionButton at=473,493,62,44 by=panel drawn focus heard=ZIP limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Zip
              41 TextBlock at=490,499,28,32 box=28,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=27.35,31.2 limit=473,493,62,44 owner=40 paints=#9C9CA0FF role=text size=26 text=ZIP
            42 ActionButton at=547,493,184,44 by=panel drawn focus heard="CONVERT TO CHD" limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text="Convert to CHD"
              43 TextBlock at=564,499,150,32 box=150,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=149.58,31.2 limit=547,493,184,44 owner=42 paints=#9C9CA0FF role=text size=26 text="CONVERT TO CHD"
            44 ActionButton at=743,493,113,44 by=panel drawn focus heard=REFRESH limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Refresh
              45 TextBlock at=760,499,79,32 box=79,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=78.94,31.2 limit=743,493,113,44 owner=44 paints=#9C9CA0FF role=text size=26 text=REFRESH
            46 ActionButton at=868,493,82,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#55555AFF,#9C9CA0FF role=button text=Mark
              47 TextBlock at=885,499,48,32 box=48,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=47.48,31.2 limit=868,493,82,44 owner=46 paints=#9C9CA0FF role=text size=26 text=MARK
        48 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          49 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            50 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              51 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=130.13,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="Nothing marked"
            52 StackPanel at=432,586,587,44 by=panel limit=234,124,812.35,548 role=none
              53 Button at=432,586,105,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#55555AFF,#9C9CA0FF role=button text="Apply…"
                54 TextBlock at=449,592,71,32 box=71,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=70.54,31.2 limit=432,586,105,44 off owner=53 paints=#55555AFF role=text size=26 text="APPLY…"
              55 StackPanel at=549,586,470,44 by=panel limit=234,124,812.35,548 role=none
                56 ProgressBar at=549,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                57 TextBlock at=757,594,262,29 box=262,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=261.82,28.8 limit=234,124,812.35,548 paints=#9C9CA0FF role=text size=24 text="263 KB in packages · 412 GB free"
        58 HintBar at=274,642,733,22 by=panel heard="Select, Back, Section, Apply…, Refresh, Mark" limit=234,124,812.35,548 role=group
focus 5
edge 5 up same
edge 5 down 44
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 36
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 38
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 38
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 42
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 42
edge 18 left 16
edge 18 right 5
edge 36 up 10
edge 36 down item:ButtonBar/0//
edge 36 left same
edge 36 right 38
edge 38 up 12
edge 38 down item:ButtonBar/0//
edge 38 left 36
edge 38 right 40
edge 40 up 14
edge 40 down item:ButtonBar/0//
edge 40 left 38
edge 40 right 42
edge 42 up 16
edge 42 down item:ButtonBar/0//
edge 42 left 40
edge 42 right 44
edge 44 up 5
edge 44 down item:ButtonBar/0//
edge 44 left 42
edge 44 right 46
edge 46 up 5
edge 46 down item:ButtonBar/0//
edge 46 left 44
edge 46 right same
edge item:ButtonBar/0// up 5
edge item:ButtonBar/0// down 46
edge item:ButtonBar/0// left 42
edge item:ButtonBar/0// right same
press pageup focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press pagedown focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
