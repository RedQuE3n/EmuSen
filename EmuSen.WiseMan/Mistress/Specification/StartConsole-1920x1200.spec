# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state StartConsole-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case StartConsole
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a ring or a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard="All · 8 games" limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard="Start Console" limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,525,45 by=panel heard="A Choose, B Back, Menu Close" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,243 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,243 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,243 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,243 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,243
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard="Start Master System" in=9 limit=96,234,636,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Start Master System"
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard="Start PlayStation" in=9 limit=96,234,636,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Start PlayStation"
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard="Start PlayStation 2" in=9 limit=96,234,636,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Start PlayStation 2"
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 1: Start PlayStation" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 0: Start Master System" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 5: Start Console" top=none
