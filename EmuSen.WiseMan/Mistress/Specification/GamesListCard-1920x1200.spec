# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GamesListCard-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case GamesListCard
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard="All · 8 games" limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard="All · 8 games" limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,525,45 by=panel heard="A Choose, B Back, Menu Close" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,567 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,567 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,567 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,567
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard=Search in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard=Sort in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Sort
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard=Filter in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Filter  ›"
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard="Jump to" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Jump to"
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard="Random game" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Random game"
                    15 ListBoxItem at=96,639,636,81 by=fixed-scroll drawn focus heard="Start Console" in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Start Console"
                    16 ListBoxItem at=96,720,636,81 by=fixed-scroll drawn focus heard=Collections in=9 limit=96,234,636,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Collections
            17 Border at=864,234,960,795 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane paints=#26262BFF role=none
              18 Border at=900,270,888,723 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                19 PauseCardPane at=900,270,888,79 by=panel limit=0,0,1920,1200 name=PauseCardPane role=none
                  20 StackPanel at=900,270,888,79 by=panel limit=0,0,1920,1200 name=PaneLines role=none
                    21 StackPanel at=900,270,888,79 by=panel limit=0,0,1920,1200 role=none
                      22 TextBlock at=900,270,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard="Type part of a name; the covers narrow as you type." layout=710.52,39.6 limit=0,0,1920,1200 paints=#EEEEF0FF role=text size=22 text="Type part of a name; the covers narrow as you type."
                      23 TextBlock at=900,313,888,36 box=888,36 by=panel caps=13.999 drawn face=Barlow heard="The keyboard docks at the foot of the screen." layout=564.45,36 limit=0,0,1920,1200 paints=#8D8D91FF role=text size=20 text="The keyboard docks at the foot of the screen."
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 1: Sort" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 0: Search" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
