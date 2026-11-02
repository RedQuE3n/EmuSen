# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PackagesPlan-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case PackagesPlan
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 ContentControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#D4D4D4FF role=none
      2 PackagesPane at=72,42,1776,1146 by=panel heard=Packages limit=0,0,1920,1200 role=pane
        3 Panel at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
          4 Grid at=72,42,1776,1146 by=panel limit=0,0,1920,1200 role=none
            5 Border at=72,42,1776,76.5 by=panel heard=PACKAGES limit=0,0,1920,1200 name=AdaptiveHeader role=none
              6 DockPanel at=96,60,1728,58.5 by=panel limit=0,0,1920,1200 role=none
                7 TextBlock at=1486.5,72,337.5,36 box=225,24 by=panel caps=13.999 drawn face="Barlow Condensed" heard="6 games · 3 packaged · 1 problems" layout=224.8,24 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="6 games · 3 packaged · 1 problems"
                8 StackPanel at=96,60,1372.5,58.5 by=panel limit=0,0,1920,1200 role=none
                  9 TextBlock at=96,60,1372.5,58.5 box=915,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard=PACKAGES layout=104.77,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text=PACKAGES
            10 DockPanel at=96,136.5,1728,69 by=panel limit=0,0,1920,1200 role=none
              11 Border at=1554,136.5,270,69 by=panel limit=0,0,1920,1200 name=AdaptiveSearch role=none
                12 TextBox at=1554,136.5,270,69 by=panel drawn focus heard=Search limit=0,0,1920,1200 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
                  13 ScrollViewer allow="a text box scrolls its own text" at=1570.5,147,237,48 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
                    14 TextBlock allow="a text box scrolls its own text" at=1570.5,147,237,48 box=158,32 caps=18.1987 drawn face="Barlow Condensed" heard="Search games" in=13 layout=119.99,31.2 name=PART_Placeholder paints=#8D8D91FF part role=text size=26 text="Search games"
              15 Border at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 name=AdaptiveNavigation role=none
                16 WholeRow at=96,136.5,1458,69 by=panel limit=0,0,1920,1200 role=none
                  17 ToggleButton at=96,138,99,66 by=panel drawn focus heard=All limit=0,0,1920,1200 op paints=#4C9AE8FF,#EEEEF0FF role=button text=All
                    18 TextBlock at=120,150,51,36 box=34,24 by=clip caps=13.999 drawn face=Barlow heard=ALL layout=33.56,24 limit=96,138,99,66 owner=17 paints=#EEEEF0FF role=text size=20 text=ALL
                  19 ToggleButton at=207,138,102,66 by=panel drawn focus heard=NES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=NES
                    20 TextBlock at=231,150,54,36 box=36,24 by=clip caps=13.999 drawn face=Barlow heard=NES layout=35.76,24 limit=207,138,102,66 owner=19 paints=#D4D4D4FF role=text size=20 text=NES
                  21 ToggleButton at=321,138,219,66 by=panel drawn focus heard=PlayStation limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=PlayStation
                    22 TextBlock at=345,150,171,36 box=114,24 by=clip caps=13.999 drawn face=Barlow heard=PLAYSTATION layout=113.56,24 limit=321,138,219,66 owner=21 paints=#D4D4D4FF role=text size=20 text=PLAYSTATION
                  23 ToggleButton at=552,138,159,66 by=panel drawn focus heard="Sega CD" limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text="Sega CD"
                    24 TextBlock at=576,150,111,36 box=74,24 by=clip caps=13.999 drawn face=Barlow heard="SEGA CD" layout=73.12,24 limit=552,138,159,66 owner=23 paints=#D4D4D4FF role=text size=20 text="SEGA CD"
                  25 ToggleButton at=723,138,118.5,66 by=panel drawn focus heard=SNES limit=0,0,1920,1200 op paints=#D4D4D4FF role=button text=SNES
                    26 TextBlock at=747,150,70.5,36 box=47,24 by=clip caps=13.999 drawn face=Barlow heard=SNES layout=46.76,24 limit=723,138,118.5,66 owner=25 paints=#D4D4D4FF role=text size=20 text=SNES
            27 Grid at=96,223.5,1728,793.5 by=panel limit=0,0,1920,1200 role=none
              28 Border at=96,223.5,1728,793.5 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                29 Grid at=96,223.5,1728,793.5 by=panel limit=0,0,1920,1200 role=none
                  30 Border at=426,327,1069.5,586.5 by=panel limit=0,0,1920,1200 paints=#1B1B1EFF,#707070FF role=none
                    31 DockPanel at=451.5,352.5,1018.5,535.5 by=panel limit=0,0,1920,1200 role=none
                      32 TextBlock at=451.5,352.5,1018.5,58.5 box=679,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Apply these changes?" layout=225.5,38.4 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=32 text="Apply these changes?"
                      33 StackPanel at=451.5,750,1018.5,138 by=panel limit=0,0,1920,1200 role=none
                        34 Button at=451.5,750,1018.5,66 by=panel drawn focus heard=No limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=No
                          35 TextBlock at=477,759,36,48 box=24,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=NO layout=23.43,31.2 limit=451.5,750,1018.5,66 owner=34 paints=#EEEEF0FF role=text size=26 text=NO
                        36 Button at=451.5,822,1018.5,66 by=panel drawn focus heard=Yes limit=0,0,1920,1200 op paints=#707070FF,#D4D4D4FF role=button text=Yes
                          37 TextBlock at=477,831,46.5,48 box=31,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=YES layout=30.76,31.2 limit=451.5,822,1018.5,66 owner=36 paints=#D4D4D4FF role=text size=26 text=YES
                      38 ScrollViewer at=451.5,429,1018.5,303 by=panel limit=0,0,1920,1200 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=451.5,429,1018.5,303
                        39 TextBlock at=451.5,429,1018.5,303 box=679,202 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed." in=38 layout=673.85,201.6 limit=451.5,429,1018.5,303 paints=#D4D4D4FF role=text size=24 text="Make 1 package, linked: each game stays where it is in the library.\n    Amber Saga (USA) (SNES)\nConvert 1 disc to CHD, each checked three ways once it is written and given a package. The originals are never removed.\n    Basalt Run - The Long Way Round Through the Mountains of the North (Europe) (Disc 2 of 3) (PlayStation)\nNothing in the library is changed, moved or removed."
            40 Border at=72,1017,1776,111 by=panel limit=0,0,1920,1200 name=AdaptiveStatus role=none
              41 Grid at=78,1023,1764,99 by=panel limit=0,0,1920,1200 role=none
                42 Grid at=78,1023,1764,43.5 by=panel limit=0,0,1920,1200 role=none
                  43 TextBlock at=90,1023,1752,43.5 box=1168,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="2 marked: make, linked 1, convert to CHD 1" layout=326.38,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="2 marked: make, linked 1, convert to CHD 1"
                44 StackPanel at=1150.5,1078.5,691.5,43.5 by=panel limit=0,0,1920,1200 role=none
                  45 StackPanel at=1150.5,1078.5,691.5,43.5 by=panel limit=0,0,1920,1200 role=none
                    46 ProgressBar at=1150.5,1090.5,300,21 by=panel drawn heard=Space limit=0,0,1920,1200 paints=#333337FF,#4C9AE8FF role=progressbar
                    47 TextBlock at=1462.5,1078.5,379.5,43.5 box=253,29 by=panel caps=16.7988 drawn face="Barlow Condensed" heard="263 KB in packages · 412 GB free" layout=252.31,28.8 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=24 text="263 KB in packages · 412 GB free"
            48 HintBar at=96,1140,1728,36 by=panel heard="Open, Back, Console, List, Apply…, Mark" limit=0,0,1920,1200 role=group
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
