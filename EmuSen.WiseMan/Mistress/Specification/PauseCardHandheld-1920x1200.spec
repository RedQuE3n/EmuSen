# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state PauseCardHandheld-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case PauseCardHandheld
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Pause limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1161 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1161 name=AdaptiveCard role=menu
            5 HintBar at=476,1075,967,71 by=panel heard="A Select, B Back, Menu Close Menu, X Screenshot, D-pad left and right Change" limit=0,0,1920,1161 role=group
            6 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
              7 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                8 LunaList at=36,357,768,567 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=36,357,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,357,768,567
                    10 ListBoxItem at=36,357,768,81 by=fixed-scroll drawn focus heard=Resume in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Resume
                    11 ListBoxItem at=36,438,768,81 by=fixed-scroll drawn focus heard=Rewind in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Rewind
                    12 ListBoxItem at=36,519,768,81 by=fixed-scroll drawn focus heard=Save in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Save      <  Slot 1 (Empty)  >"
                    13 ListBoxItem at=36,600,768,81 by=fixed-scroll drawn focus heard=Load in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Load      <  Slot 1 (Empty)  >"
                    14 ListBoxItem at=36,681,768,81 by=fixed-scroll drawn focus heard=Speed in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Speed      <  100%  >"
                    15 ListBoxItem at=36,762,768,81 by=fixed-scroll drawn focus heard="Game Options" in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Options  ›"
                    16 ListBoxItem at=36,843,768,81 by=fixed-scroll drawn focus heard=Leave in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Leave  ›"
            17 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
              18 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Rewind" top=none
press up menu="open at 0: Resume" top=none
press back top=none
