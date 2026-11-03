# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktopGame-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,768
case HomeCardDesktopGame
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 role=none
    1 AdaptiveLayout at=0,0,1280,768 by=panel heard=Home limit=0,0,1280,768 name=PadCard role=pane
      2 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,768 by=panel heard=Home limit=0,0,1280,768 name=AdaptiveCard role=menu
            5 HintBar at=64,704,509,30 by=panel heard="A Choose, B Back, Y Favourite, Menu Close" limit=0,0,1280,768 role=group
            6 Border at=64,156,424,378 by=panel limit=0,0,1280,768 name=AdaptiveCardRows role=none
              7 Border at=64,156,424,378 by=panel limit=0,0,1280,768 name=AdaptiveContent role=none
                8 LunaList at=64,156,424,378 by=panel heard=Menu limit=0,0,1280,768 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=64,156,424,378 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,156,424,378
                    10 ListBoxItem at=64,156,424,54 by=fixed-scroll drawn focus heard=Continue in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Continue
                    11 ListBoxItem at=64,210,424,54 by=fixed-scroll drawn focus heard="Selected game" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Cobalt Harbor (Synthetic)"
                    12 ListBoxItem at=64,264,424,54 by=fixed-scroll drawn focus heard=Search in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                    13 ListBoxItem at=64,318,424,54 by=fixed-scroll drawn focus heard="Start Console" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Start Console  ›"
                    14 ListBoxItem at=64,372,424,54 by=fixed-scroll drawn focus heard=Tools in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                    15 ListBoxItem at=64,426,424,54 by=fixed-scroll drawn focus heard=Settings in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                    16 ListBoxItem at=64,480,424,54 by=fixed-scroll drawn focus heard=Power in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
            17 Border at=576,156,640,498 by=panel limit=0,0,1280,768 name=AdaptiveCardPane paints=#26262BFF role=none
              18 Border at=600,180,592,450 by=panel limit=0,0,1280,768 name=AdaptiveDetail role=none
                19 PauseCardPane at=600,180,592,109 by=panel limit=0,0,1280,768 name=PauseCardPane role=none
                  20 Grid at=600,180,592,109 by=panel limit=0,0,1280,768 role=none
                    21 StackPanel at=600,180,592,109 by=panel limit=0,0,1280,768 role=none
                      22 TextBlock at=600,180,592,39 box=592,39 by=panel caps=22.3984 drawn face="Barlow Condensed" heard="Cobalt Harbor (Synthetic)" layout=256.38,38.4 limit=0,0,1280,768 name=PaneTitle paints=#EEEEF0FF role=text size=32 text="Cobalt Harbor (Synthetic)"
                      23 TextBlock at=600,227,592,27 box=592,27 by=panel caps=15.3989 drawn face=Barlow heard="SNES (Venus)" layout=123.55,26.4 limit=0,0,1280,768 name=PaneCaption paints=#8D8D91FF role=text size=22 text="SNES (Venus)"
                      24 TextBlock at=600,262,592,27 box=592,27 by=panel caps=15.3989 drawn face=Barlow heard="A opens its page: states, options, media and details." layout=477.36,26.4 limit=0,0,1280,768 name=PaneHint paints=#8D8D91FF role=text size=22 text="A opens its page: states, options, media and details."
focus none
press down menu="open at 2: Search" top=none
press up menu="open at 1: Cobalt Harbor (Synthetic)" top=none
press back top=none
