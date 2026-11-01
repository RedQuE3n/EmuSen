# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case PackagesPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 ContentControl at=0,0,1920,1161 by=panel limit=0,0,1920,1161 paints=#D4D4D4FF role=none
      2 PackagesPane at=72,42,1776,1107 by=panel heard=Packages limit=0,0,1920,1161 role=pane
        3 Panel at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
          4 Grid at=72,42,1776,1107 by=panel limit=0,0,1920,1161 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=PACKAGES limit=0,0,1920,1161 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1161 role=none
                7 TextBlock at=1486.5,72,337.5,36 box=225,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="6 games · 3 packaged · 1 problems" layout=224.8,24 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problems"
                8 StackPanel at=96,60,1372.5,58.5 by=panel limit=0,0,1920,1161 role=none
                  9 TextBlock at=96,60,1372.5,58.5 box=915,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1161 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1161 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1161 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=13 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              15 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1161 role=none
                  17 ToggleButton at=96,138,97.5,66 by=panel drawn focus heard=All limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=All
                    18 TextBlock at=121.5,147,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=ALL layout=30.97,31.2 limit=96,138,97.5,66 owner=17 paints=#EEEEF0FF role=text size=26 text=ALL
                  19 ToggleButton at=205.5,138,100.5,66 by=panel drawn focus heard=NES limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=NES
                    20 TextBlock at=231,147,49.5,48 box=33,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NES layout=32.68,31.2 limit=205.5,138,100.5,66 owner=19 paints=#FFFFFFFF role=text size=26 text=NES
                  21 ToggleButton at=318,138,211.5,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=PlayStation
                    22 TextBlock at=343.5,147,160.5,48 box=107,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PLAYSTATION layout=106.31,31.2 limit=318,138,211.5,66 owner=21 paints=#FFFFFFFF role=text size=26 text=PLAYSTATION
                  23 ToggleButton at=541.5,138,156,66 by=panel drawn focus heard="Sega CD" limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text="Sega CD"
                    24 TextBlock at=567,147,105,48 box=70,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="SEGA CD" layout=69.71,31.2 limit=541.5,138,156,66 owner=23 paints=#FFFFFFFF role=text size=26 text="SEGA CD"
                  25 ToggleButton at=709.5,138,115.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1161 op paints=#FFFFFFFF role=button text=SNES
                    26 TextBlock at=735,147,64.5,48 box=43,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SNES layout=42.93,31.2 limit=709.5,138,115.5,66 owner=25 paints=#FFFFFFFF role=text size=26 text=SNES
            27 Grid at=96,223.5,1728,735 by=panel limit=0,0,1920,1161 role=none
              28 Border at=96,223.5,1728,735 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                29 Grid at=96,223.5,1728,735 by=panel limit=0,0,1920,1161 role=none
                  30 Border at=426,298.5,1069.5,586.5 by=panel limit=0,0,1920,1161 paints=#1B1B1EFF,#707070FF role=none
                    31 DockPanel at=451.5,324,1018.5,535.5 by=panel limit=0,0,1920,1161 role=none
                      32 TextBlock at=451.5,324,1018.5,58.5 box=679,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                      33 StackPanel at=451.5,721.5,1018.5,138 by=panel limit=0,0,1920,1161 role=none
                        34 Button at=451.5,721.5,1018.5,66 by=panel drawn focus heard=No limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=No
                          35 TextBlock at=477,730.5,36,48 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=451.5,721.5,1018.5,66 owner=34 paints=#EEEEF0FF role=text size=26 text=NO
                        36 Button at=451.5,793.5,1018.5,66 by=panel drawn focus heard=Yes limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                          37 TextBlock at=477,802.5,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=451.5,793.5,1018.5,66 owner=36 paints=#D4D4D4FF role=text size=26 text=YES
                      38 ScrollViewer at=451.5,400.5,1018.5,303 by=panel limit=0,0,1920,1161 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=451.5,400.5,1018.5,303
                        39 TextBlock at=451.5,400.5,1018.5,303 box=679,202 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=38 layout=673.85,201.6 limit=451.5,400.5,1018.5,303 paints=#D4D4D4FF role=text size=24 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
            40 Border at=72,958.5,1776,133.5 by=panel limit=0,0,1920,1161 name=AdaptiveStatus role=none
              41 Grid at=78,964.5,1764,121.5 by=panel limit=0,0,1920,1161 role=none
                42 Grid at=78,964.5,1764,43.5 by=panel limit=0,0,1920,1161 role=none
                  43 TextBlock at=90,964.5,1752,43.5 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: make, linked 1, convert to CHD 1" layout=326.38,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="2 marked: make, linked 1, convert to CHD 1"
                44 StackPanel at=979.5,1020,862.5,66 by=panel limit=0,0,1920,1161 role=none
                  45 Button at=979.5,1020,153,66 by=panel drawn focus heard="Apply the marked changes" limit=0,0,1920,1161 op paints=#707070FF,#D4D4D4FF role=button text="Apply…"
                    46 TextBlock at=1005,1029,102,48 box=68,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="APPLY…" layout=67.24,31.2 limit=979.5,1020,153,66 owner=45 paints=#D4D4D4FF role=text size=26 text="APPLY…"
                  47 StackPanel at=1150.5,1020,691.5,66 by=panel limit=0,0,1920,1161 role=none
                    48 ProgressBar at=1150.5,1042.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1161 paints=#333337FF,#4C9AE8FF role=progressbar
                    49 TextBlock at=1462.5,1032,379.5,43.5 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=0,0,1920,1161 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
            50 HintBar at=96,1104,1728,33 by=panel heard="Open, Back, Section, List, Apply…, Mark" limit=0,0,1920,1161 role=group
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
edge 36 down 45
edge 36 left 19
edge 36 right 12
edge 45 up 36
edge 45 down same
edge 45 left 17
edge 45 right 12
press pageup focus="Button 'No' (No)" top=PackagesWindow
press pagedown focus="Button 'No' (No)" top=PackagesWindow
press first focus="Button 'No' (No)" top=PackagesWindow
press last focus="Button 'No' (No)" top=PackagesWindow
press back focus="GridSplitter  (Resize Mark)" top=PackagesWindow
