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
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard="Start Console" limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard="Start Console" limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=96,1104,494,45 by=panel heard="A Start, B Back, Menu Close" limit=0,0,1920,1200 role=group
            6 Border at=96,234,636,243 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,243 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,243 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,243 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,243
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard="Master System" in=9 limit=96,234,636,243 op paints=#333337FF,#D4D4D4FF role=listitem text="Master System"
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard=PlayStation in=9 limit=96,234,636,243 op paints=#333337FF,#D4D4D4FF role=listitem text=PlayStation
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard="PlayStation 2" in=9 limit=96,234,636,243 op paints=#333337FF,#D4D4D4FF role=listitem text="PlayStation 2"
            13 Border at=864,234,960,795 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane paints=#26262BFF role=none
              14 Border at=900,270,888,723 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
                15 PauseCardPane at=900,270,888,129 by=panel limit=0,0,1920,1200 name=PauseCardPane role=none
                  16 StackPanel at=900,270,888,129 by=panel limit=0,0,1920,1200 name=PaneSentences role=none
                    17 TextBlock at=900,270,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard="Starts a Master System with no disc in it, on Endou." layout=702.37,39.6 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Starts a Master System with no disc in it, on Endou."
                    18 TextBlock at=900,319,888,80 box=888,80 by=panel caps=15.3989 drawn face=Barlow heard="Its BIOS screen comes up; Change Disc on the Pause Card puts a game in." layout=883.71,79.2 limit=0,0,1920,1200 paints=#D4D4D4FF role=text size=22 text="Its BIOS screen comes up; Change Disc on the Pause Card puts a game in."
focus none
press down focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 1: PlayStation" top=none
press up focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" menu="open at 0: Master System" top=none
press back focus="GalleryLayout GamesScreen" games="all <run>/Roms/Aurora Drift (Synthetic).sfc" top=none
