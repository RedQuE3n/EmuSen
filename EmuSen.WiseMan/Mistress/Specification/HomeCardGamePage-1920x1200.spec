# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardGamePage-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case HomeCardGamePage
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Home limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1920,1161 by=panel heard="Cobalt Harbor (Synthetic)" limit=0,0,1920,1161 name=AdaptiveCard role=menu
          4 HintBar at=491,1075,938,71 by=panel heard="A Select, B Back, Menu Close Menu, Y Favourite, D-pad left and right Change" limit=0,0,1920,1161 role=group
          5 Border at=36,386,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
            6 Border at=36,386,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
              7 LunaList at=36,386,768,567 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=36,386,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,386,768,567
                  9 ListBoxItem at=36,386,768,81 by=fixed-scroll drawn focus heard="Edit This Game's Metadata" in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Edit This Game's Metadata"
                  10 ListBoxItem at=36,467,768,81 by=fixed-scroll drawn focus heard="Crop This Game..." in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Crop This Game..."
                  11 ListBoxItem at=36,548,768,81 by=fixed-scroll drawn focus heard="Television Standard" in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Television Standard"
                  12 ListBoxItem at=36,629,768,81 by=fixed-scroll drawn focus heard="Use Another Game's Cover..." in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Use Another Game's Cover..."
                  13 ListBoxItem at=36,710,768,81 by=fixed-scroll drawn focus heard="Game Package..." in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Game Package..."
                  14 ListBoxItem at=36,791,768,81 by=fixed-scroll drawn focus heard="Scrape This Game..." in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Scrape This Game..."
                  15 ListBoxItem at=36,872,768,81 by=fixed-scroll drawn focus heard="Find by Name..." in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Find by Name..."
          16 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
            17 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
focus none
press down menu="open at 1: Crop This Game..." top=none
press up menu="open at 0: Edit This Game's Metadata" top=none
press back menu="open at 1: Cobalt Harbor (Synthetic)" top=none
