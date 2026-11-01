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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=234,124,813,548 by=panel limit=234,124,812.35,548 paints=#D4D4D4FF role=none seen=234,124,812.35,548
    1 PackagesPane at=258,136,765,536 by=panel heard=Packages limit=234,124,812.35,548 role=pane
      2 Grid at=258,136,765,536 by=panel limit=234,124,812.35,548 role=none
        3 DockPanel at=274,148,733,46 by=panel limit=234,124,812.35,548 role=none
          4 Border at=827,148,180,46 by=panel limit=234,124,812.35,548 name=AdaptiveSearch role=none
            5 TextBox at=827,148,180,46 by=panel drawn focus heard=Search limit=234,124,812.35,548 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
              6 ScrollViewer allow="a text box scrolls its own text" at=838,155,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                7 TextBlock allow="a text box scrolls its own text" at=838,155,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=6 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
          8 Border at=274,148,553,46 by=panel limit=234,124,812.35,548 name=AdaptiveNavigation role=none
            9 WholeRow at=274,148,553,46 by=panel limit=234,124,812.35,548 paints=#FFFFFFFF role=none
              10 ToggleButton at=274,149,65,44 by=panel drawn focus heard=All limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=All
                11 TextBlock at=291,155,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=274,149,65,44 owner=10 paints=#EEEEF0FF role=text size=26 text=ALL
              12 ToggleButton at=347,149,67,44 by=panel drawn focus heard=NES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=NES
                13 TextBlock at=364,155,33,32 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=347,149,67,44 owner=12 paints=#FFFFFFFF role=text size=26 text=NES
              14 ToggleButton at=422,149,141,44 by=panel drawn focus heard=PlayStation limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=PlayStation
                15 TextBlock at=439,155,107,32 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=422,149,141,44 owner=14 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
              16 ToggleButton at=571,149,104,44 by=panel drawn focus heard="Sega CD" limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text="Sega CD"
                17 TextBlock at=588,155,70,32 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=571,149,104,44 owner=16 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
              18 ToggleButton at=683,149,77,44 by=panel drawn focus heard=SNES limit=234,124,812.35,548 op paints=#FFFFFFFF role=button text=SNES
                19 TextBlock at=700,155,43,32 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=683,149,77,44 owner=18 paints=#FFFFFFFF role=text size=26 text=SNES
        20 Grid at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
          21 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 name=AdaptiveDetail role=none
            22 Border at=274,206,733,279 by=panel limit=234,124,812.35,548 role=none
              23 ScrollViewer at=282,214,717,263 by=panel fades=t limit=234,124,812.35,548 overflow=488 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=282,214,717,263
                24 PackageDetail at=282,-143,717,751 by=scroll in=23 limit=282,214,717,263 name=PackagesDetail paints=#D4D4D4FF role=none seen=282,214,717,263
                  25 FieldRow at=282,203,717,62 by=scroll heard=Format in=23 limit=282,214,717,263 paints=#D4D4D4FF role=group seen=282,214,717,51
                    26 TextBlock at=282,203,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Format in=23 layout=54.29,28.8 limit=282,203,717,62 name=PART_Label paints=#D4D4D4FF part role=text seen=282,214,717,18 size=24 text=Format
                    27 TextBlock at=282,236,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="CUE/BIN, which can be converted to CHD" in=23 layout=312.96,28.8 limit=282,203,717,62 paints=#D4D4D4FF role=text size=24 text="CUE/BIN, which can be converted to CHD"
                  28 GamePackageDetail at=282,275,717,333 by=scroll in=23 limit=282,214,717,263 name=GamePackageDetail paints=#8D8D91FF,#D4D4D4FF role=none seen=282,275,717,202
                    29 HintText name=GamePackageAbout paints=#8D8D91FF role=text volatile="its words name the run's own folders"
                    30 FieldRow at=282,363,717,114 by=scroll heard=Package in=23 limit=282,214,717,263 paints=#D4D4D4FF role=group
                      31 TextBlock at=282,363,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Package in=23 layout=65.81,28.8 limit=282,363,717,114 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Package
                      32 StackPanel at=282,396,717,81 by=clip heard=Package in=23 limit=282,363,717,114 role=none
                        33 TextBlock at=282,396,717,29 box=717,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Not a package yet." in=23 layout=144.72,28.8 limit=282,363,717,114 name=GamePackagePlace paints=#D4D4D4FF role=text size=24 text="Not a package yet."
                        34 ButtonBar at=839,433,160,44 by=clip in=23 limit=282,363,717,114 paints=#D4D4D4FF role=toolbar
                          35 Button at=839,433,160,44 by=clip drawn focus heard="MAKE PACKAGE" in=23 limit=839,433,160,44 name=GamePackageMake op paints=#707070FF,#D4D4D4FF role=button text="Make Package"
                            36 TextBlock at=856,439,126,32 box=126,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="MAKE PACKAGE" in=23 layout=125.29,31.2 limit=839,433,160,44 owner=35 paints=#EEEEF0FF role=text size=26 text="MAKE PACKAGE"
        37 Border at=274,493,733,44 by=panel limit=234,124,812.35,548 name=AdaptiveActions role=none
          38 WrapPanel at=274,493,733,44 by=panel limit=234,124,812.35,548 role=none
            39 ActionButton at=274,493,80,44 by=panel drawn focus heard=MAKE limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Make
              40 TextBlock at=291,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MAKE layout=45.21,31.2 limit=274,493,80,44 owner=39 paints=#D4D4D4FF role=text size=26 text=MAKE
            41 ActionButton at=366,493,89,44 by=panel drawn focus heard=CHECK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Check
              42 TextBlock at=383,499,55,32 box=55,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CHECK layout=54.55,31.2 limit=366,493,89,44 owner=41 paints=#D4D4D4FF role=text size=26 text=CHECK
            43 ActionButton at=467,493,60,44 by=panel drawn focus heard=ZIP limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Zip
              44 TextBlock at=484,499,26,32 box=26,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ZIP layout=25.45,31.2 limit=467,493,60,44 owner=43 paints=#D4D4D4FF role=text size=26 text=ZIP
            45 ActionButton at=539,493,176,44 by=panel drawn focus heard="CONVERT TO CHD" limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text="Convert to CHD"
              46 TextBlock at=556,499,142,32 box=142,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="CONVERT TO CHD" layout=141.08,31.2 limit=539,493,176,44 owner=45 paints=#D4D4D4FF role=text size=26 text="CONVERT TO CHD"
            47 ActionButton at=727,493,109,44 by=panel drawn focus heard=REFRESH limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Refresh
              48 TextBlock at=744,499,75,32 box=75,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=REFRESH layout=74.7,31.2 limit=727,493,109,44 owner=47 paints=#D4D4D4FF role=text size=26 text=REFRESH
            49 ActionButton at=848,493,80,44 by=panel drawn focus heard=MARK limit=234,124,812.35,548 op paints=#707070FF,#D4D4D4FF role=button text=Mark
              50 TextBlock at=865,499,46,32 box=46,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=MARK layout=45.97,31.2 limit=848,493,80,44 owner=49 paints=#D4D4D4FF role=text size=26 text=MARK
        51 Border at=258,545,765,89 by=panel limit=234,124,812.35,548 name=AdaptiveStatus role=none
          52 Grid at=262,549,757,81 by=panel limit=234,124,812.35,548 role=none
            53 Grid at=262,549,757,29 by=panel limit=234,124,812.35,548 role=none
              54 TextBlock at=270,549,749,29 box=749,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="Nothing marked" layout=124.03,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="Nothing marked"
            55 StackPanel at=444,586,575,44 by=panel limit=234,124,812.35,548 role=none
              56 Button at=444,586,102,44 by=panel drawn heard="Apply the marked changes" limit=234,124,812.35,548 off paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                57 TextBlock at=461,592,68,32 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=444,586,102,44 off owner=56 paints=#707070FF role=text size=26 text="APPLY…"
              58 StackPanel at=558,586,461,44 by=panel limit=234,124,812.35,548 role=none
                59 ProgressBar at=558,601,200,14 by=panel drawn heard=Space limit=234,124,812.35,548 paints=#333337FF,#4C9AE8FF role=progressbar
                60 TextBlock at=766,594,253,29 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=234,124,812.35,548 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
        61 HintBar at=274,642,733,22 by=panel heard="Select, Back, Section, Apply…, Refresh, Mark" limit=234,124,812.35,548 role=group
focus 35
edge 5 up same
edge 5 down 35
edge 5 left 18
edge 5 right same
edge 10 up same
edge 10 down 39
edge 10 left same
edge 10 right 12
edge 12 up same
edge 12 down 41
edge 12 left 10
edge 12 right 14
edge 14 up same
edge 14 down 41
edge 14 left 12
edge 14 right 16
edge 16 up same
edge 16 down 45
edge 16 left 14
edge 16 right 18
edge 18 up same
edge 18 down 45
edge 18 left 16
edge 18 right 5
edge 35 up 5
edge 35 down 49
edge 35 left 47
edge 35 right same
edge 39 up 10
edge 39 down same
edge 39 left same
edge 39 right 41
edge 41 up 12
edge 41 down same
edge 41 left 39
edge 41 right 43
edge 43 up 14
edge 43 down same
edge 43 left 41
edge 43 right 45
edge 45 up 16
edge 45 down same
edge 45 left 43
edge 45 right 47
edge 47 up 18
edge 47 down same
edge 47 left 45
edge 47 right 49
edge 49 up 35
edge 49 down same
edge 49 left 47
edge 49 right same
press pageup title=Packages top=PackagesWindow
press pagedown title=Packages top=PackagesWindow
press first focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press last focus="TextBox Search games (Search)" title=Packages top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" title=Packages top=PackagesWindow
