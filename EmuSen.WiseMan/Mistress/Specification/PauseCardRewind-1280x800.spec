# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardRewind-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case PauseCardRewind
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 AdaptiveLayout at=0,0,1280,800 by=panel heard=Pause limit=0,0,1280,800 name=PadCard role=pane
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,800 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,800 name=AdaptiveCard role=menu
            5 HintBar at=64,736,565,30 by=panel heard="A Select, B Resume, X Screenshot, Menu Resume" limit=0,0,1280,800 role=group
            6 Border at=64,156,424,378 by=panel limit=0,0,1280,800 name=AdaptiveCardRows role=none
              7 Border at=64,156,424,378 by=panel limit=0,0,1280,800 name=AdaptiveContent role=none
                8 LunaList at=64,156,424,378 by=panel heard=Menu limit=0,0,1280,800 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=64,156,424,378 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,156,424,378
                    10 ListBoxItem at=64,156,424,54 by=fixed-scroll drawn focus heard=Resume in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                    11 ListBoxItem at=64,210,424,54 by=fixed-scroll drawn focus heard=Rewind in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                    12 ListBoxItem at=64,264,424,54 by=fixed-scroll drawn focus heard="Save State" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                    13 ListBoxItem at=64,318,424,54 by=fixed-scroll drawn focus heard="Load State" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                    14 ListBoxItem at=64,372,424,54 by=fixed-scroll drawn focus heard=Speed in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                    15 ListBoxItem at=64,426,424,54 by=fixed-scroll drawn focus heard="Game Options" in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                    16 ListBoxItem at=64,480,424,54 by=fixed-scroll drawn focus heard=Leave in=9 limit=64,156,424,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
            17 Border at=576,156,640,530 by=panel limit=0,0,1280,800 name=AdaptiveCardPane paints=#26262BFF role=none
              18 Border at=600,180,592,482 by=panel limit=0,0,1280,800 name=AdaptiveDetail role=none
                19 PauseCardPane at=600,180,592,65 by=panel limit=0,0,1280,800 name=PauseCardPane role=none
                  20 StackPanel at=600,180,592,65 by=panel limit=0,0,1280,800 role=none
                    21 TextBlock name=PaneReelCaption paints=#D4D4D4FF role=text volatile="the reel's newest moments in the Pause Card's pane, taken as the emulation thread runs against the wall clock"
focus none
press down menu="open at 2: Save      <  Slot 1 (Empty)  >" top=none
press up menu="open at 1: Rewind" top=none
press back top=none
