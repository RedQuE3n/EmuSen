# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardRewind-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case PauseCardRewind
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 AdaptiveLayout at=0,0,1280,774 by=panel heard=Pause limit=0,0,1280,774 name=PadCard role=pane
      2 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1280,774 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1280,774 name=AdaptiveCard role=menu
          4 HintBar at=318,716,645,48 by=panel heard="A Select, B Back, Menu Close Menu, X Screenshot, D-pad left and right Change" limit=0,0,1280,774 role=group
          5 Border at=24,238,512,378 by=panel limit=0,0,1280,774 name=AdaptiveCardRows role=none
            6 Border at=24,238,512,378 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
              7 LunaList at=24,238,512,378 by=panel heard=Menu limit=0,0,1280,774 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=24,238,512,378 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=24,238,512,378
                  9 ListBoxItem at=24,238,512,54 by=fixed-scroll drawn focus heard=Resume in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                  10 ListBoxItem at=24,292,512,54 by=fixed-scroll drawn focus heard=Rewind in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                  11 ListBoxItem at=24,346,512,54 by=fixed-scroll drawn focus heard=Save in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                  12 ListBoxItem at=24,400,512,54 by=fixed-scroll drawn focus heard=Load in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                  13 ListBoxItem at=24,454,512,54 by=fixed-scroll drawn focus heard=Speed in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                  14 ListBoxItem at=24,508,512,54 by=fixed-scroll drawn focus heard="Game Options" in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                  15 ListBoxItem at=24,562,512,54 by=fixed-scroll drawn focus heard=Leave in=8 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
          16 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveCardPane role=none
            17 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
              18 PauseCardPane at=746,338,323,39 by=panel limit=0,0,1280,774 name=PauseCardPane role=none
                19 StackPanel at=746,338,323,39 by=panel limit=0,0,1280,774 role=none
                  20 TextBlock name=PaneReelCaption paints=#8D8D91FF role=text volatile="the reel's newest moments in the Pause Card's pane, taken as the emulation thread runs against the wall clock"
focus none
press down menu="open at 2: Save      <  Slot 1 (Empty)  >" top=none
press up menu="open at 1: Rewind" top=none
press back top=none
