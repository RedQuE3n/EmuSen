# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardTools-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case HomeCardTools
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Home limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1920,1161 by=panel heard=Home limit=0,0,1920,1161 name=AdaptiveCard role=menu
          4 HintBar at=579,1075,762,71 by=panel heard="A Select, B Back, Menu Close Menu, Y Favourite" limit=0,0,1920,1161 role=group
          5 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
            6 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
              7 LunaList at=36,357,768,567 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=36,357,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,357,768,567
                  9 ListBoxItem at=36,357,768,81 by=fixed-scroll drawn focus heard=Continue in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Continue
                  10 ListBoxItem at=36,438,768,81 by=fixed-scroll drawn focus heard="Cobalt Harbor (Synthetic)" in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Cobalt Harbor (Synthetic)  ›"
                  11 ListBoxItem at=36,519,768,81 by=fixed-scroll drawn focus heard=Search in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                  12 ListBoxItem at=36,600,768,81 by=fixed-scroll drawn focus heard="Start a Console" in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Start a Console  ›"
                  13 ListBoxItem at=36,681,768,81 by=fixed-scroll drawn focus heard=Tools in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                  14 ListBoxItem at=36,762,768,81 by=fixed-scroll drawn focus heard=Settings in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                  15 ListBoxItem at=36,843,768,81 by=fixed-scroll drawn focus heard=Power in=8 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
          16 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
            17 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
              18 PauseCardPane at=1002,352,720,370 by=panel limit=0,0,1920,1161 name=PauseCardPane role=none
                19 StackPanel at=1002,352,720,370 by=panel limit=0,0,1920,1161 name=PaneLines role=none
                  20 StackPanel at=1002,352,720,79 by=panel limit=0,0,1920,1161 role=none
                    21 TextBlock at=1002,352,720,40 box=720,40 by=panel caps=15.3989 drawn face=Barlow heard=Scraper layout=109.89,39.6 limit=0,0,1920,1161 paints=#EEEEF0FF role=text size=22 text=Scraper
                    22 TextBlock at=1002,395,720,36 box=720,36 by=panel caps=13.999 drawn face=Barlow heard="41 games missing media" layout=305.16,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="41 games missing media"
                  23 StackPanel at=1002,449,720,79 by=panel limit=0,0,1920,1161 role=none
                    24 TextBlock at=1002,449,720,40 box=720,40 by=panel caps=15.3989 drawn face=Barlow heard=Packages layout=134.64,39.6 limit=0,0,1920,1161 paints=#EEEEF0FF role=text size=22 text=Packages
                    25 TextBlock at=1002,492,720,36 box=720,36 by=panel caps=13.999 drawn face=Barlow heard="Makes, checks and zips game packages" layout=497.85,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Makes, checks and zips game packages"
                  26 StackPanel at=1002,546,720,79 by=panel limit=0,0,1920,1161 role=none
                    27 TextBlock at=1002,546,720,40 box=720,40 by=panel caps=15.3989 drawn face=Barlow heard="Library Manager" layout=226.81,39.6 limit=0,0,1920,1161 paints=#EEEEF0FF role=text size=22 text="Library Manager"
                    28 TextBlock at=1002,589,720,36 box=720,36 by=panel caps=13.999 drawn face=Barlow heard="Finds duplicate and damaged dumps" layout=467.1,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="Finds duplicate and damaged dumps"
                  29 StackPanel at=1002,643,720,79 by=panel limit=0,0,1920,1161 role=none
                    30 TextBlock at=1002,643,720,40 box=720,40 by=panel caps=15.3989 drawn face=Barlow heard=Cores layout=79.73,39.6 limit=0,0,1920,1161 paints=#EEEEF0FF role=text size=22 text=Cores
                    31 TextBlock at=1002,686,720,36 box=720,36 by=panel caps=13.999 drawn face=Barlow heard="1 core waiting for approval" layout=330.06,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="1 core waiting for approval"
focus none
press down menu="open at 5: Settings" top=none
press up menu="open at 4: Tools" top=none
press back top=none
