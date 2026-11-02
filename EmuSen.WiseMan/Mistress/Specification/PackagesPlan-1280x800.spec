# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PackagesPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 ContentControl at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#D4D4D4FF role=none
      2 PackagesPane at=48,28,1184,764 by=panel heard=Packages limit=0,0,1280,800 role=pane
        3 Panel at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
          4 Grid at=48,28,1184,764 by=panel limit=0,0,1280,800 role=none
            5 Border at=48,28,1184,51 by=panel heard=PACKAGES limit=0,0,1280,800 name=AdaptiveHeader role=none
              6 DockPanel at=64,40,1152,39 by=panel limit=0,0,1280,800 role=none
                7 TextBlock at=991,48,225,24 box=225,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="6 games · 3 packaged · 1 problems" layout=224.8,24 limit=0,0,1280,800 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problems"
                8 StackPanel at=64,40,915,39 by=panel limit=0,0,1280,800 role=none
                  9 TextBlock at=64,40,915,39 box=915,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=64,91,1152,46 by=panel limit=0,0,1280,800 role=none
              11 Border at=1036,91,180,46 by=panel limit=0,0,1280,800 name=AdaptiveSearch role=none
                12 TextBox at=1036,91,180,46 by=panel drawn focus heard=Search limit=0,0,1280,800 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1047,98,158,32 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1047,98,158,32 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=13 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              15 Border at=64,91,972,46 by=panel limit=0,0,1280,800 name=AdaptiveNavigation role=none
                16 WholeRow at=64,91,972,46 by=panel limit=0,0,1280,800 role=none
                  17 ToggleButton at=64,92,66,44 by=panel drawn focus heard=All limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=80,100,34,24 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=64,92,66,44 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=138,92,68,44 by=panel drawn focus heard=NES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=154,100,36,24 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=138,92,68,44 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=214,92,146,44 by=panel drawn focus heard=PlayStation limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=230,100,114,24 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=214,92,146,44 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=368,92,106,44 by=panel drawn focus heard="Sega CD" limit=0,0,1280,800 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=384,100,74,24 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=368,92,106,44 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=482,92,79,44 by=panel drawn focus heard=SNES limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=498,100,47,24 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=482,92,79,44 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=64,149,1152,529 by=panel limit=0,0,1280,800 role=none
              28 Border at=64,149,1152,529 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                29 Grid at=64,149,1152,529 by=panel limit=0,0,1280,800 role=none
                  30 Border at=284,218,713,391 by=panel limit=0,0,1280,800 paints=#1B1B1EFF,#707070FF role=none
                    31 DockPanel at=301,235,679,357 by=panel limit=0,0,1280,800 role=none
                      32 TextBlock at=301,235,679,39 box=679,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                      33 StackPanel at=301,500,679,92 by=panel limit=0,0,1280,800 role=none
                        34 Button at=301,500,679,44 by=panel drawn focus heard=No limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=No
                          35 TextBlock at=318,506,24,32 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=301,500,679,44 owner=34 paints=#EEEEF0FF role=text size=26 text=NO
                        36 Button at=301,548,679,44 by=panel drawn focus heard=Yes limit=0,0,1280,800 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                          37 TextBlock at=318,554,31,32 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=301,548,679,44 owner=36 paints=#D4D4D4FF role=text size=26 text=YES
                      38 ScrollViewer at=301,286,679,202 by=panel limit=0,0,1280,800 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=301,286,679,202
                        39 TextBlock at=301,286,679,202 box=679,202 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=38 layout=673.85,201.6 limit=301,286,679,202 paints=#D4D4D4FF role=text size=24 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
            40 Border at=48,678,1184,74 by=panel limit=0,0,1280,800 name=AdaptiveStatus role=none
              41 Grid at=52,682,1176,66 by=panel limit=0,0,1280,800 role=none
                42 Grid at=52,682,1176,29 by=panel limit=0,0,1280,800 role=none
                  43 TextBlock at=60,682,1168,29 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: make, linked 1, convert to CHD 1" layout=326.38,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="2 marked: make, linked 1, convert to CHD 1"
                44 StackPanel at=767,719,461,29 by=panel limit=0,0,1280,800 role=none
                  45 StackPanel at=767,719,461,29 by=panel limit=0,0,1280,800 role=none
                    46 ProgressBar at=767,727,200,14 by=panel drawn heard=Space limit=0,0,1280,800 paints=#333337FF,#4C9AE8FF role=progressbar
                    47 TextBlock at=975,719,253,29 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=0,0,1280,800 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
            48 HintBar at=64,760,1152,24 by=panel heard="Open, Back, Console, List, Apply…, Mark" limit=0,0,1280,800 role=group
focus 34
edge 12 up same
edge 12 down 34
edge 12 left 25
edge 12 right same
edge 17 up same
edge 17 down 34
edge 17 left same
edge 17 right 19
edge 19 up same
edge 19 down 34
edge 19 left 17
edge 19 right 21
edge 21 up same
edge 21 down 34
edge 21 left 19
edge 21 right 23
edge 23 up same
edge 23 down 34
edge 23 left 21
edge 23 right 25
edge 25 up same
edge 25 down 34
edge 25 left 23
edge 25 right 12
edge 34 up 21
edge 34 down 36
edge 34 left 19
edge 34 right 12
edge 36 up 34
edge 36 down same
edge 36 left 19
edge 36 right 12
press pageup focus="Button 'No' (No)" top=PackagesWindow
press pagedown focus="Button 'No' (No)" top=PackagesWindow
press first focus="Button 'No' (No)" top=PackagesWindow
press last focus="Button 'No' (No)" top=PackagesWindow
press back focus="ListBoxItem  (Mark: not marked, Game: Granite Choir - The Very Long Packaged Edition With Every Subtitle (USA) (Rev 1) (Synthetic), State: ✓ OK)" top=PackagesWindow
