# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state BigPicture-MainMenu-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case MainMenu
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves this view's own selection, not the focus
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 AdaptiveLayout at=0,0,1920,1200 by=panel heard=Home limit=0,0,1920,1200 name=PadCard role=pane
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1200 by=panel heard=Home limit=0,0,1920,1200 name=AdaptiveCard role=menu
            5 HintBar at=682,1114,556,71 by=panel heard="A Select, B Back, Menu Close Menu" limit=0,0,1920,1200 role=group
            6 Border at=36,498,768,324 by=panel limit=0,0,1920,1200 name=AdaptiveCardRows role=none
              7 Border at=36,498,768,324 by=panel limit=0,0,1920,1200 name=AdaptiveContent role=none
                8 LunaList at=36,498,768,324 by=panel heard=Menu limit=0,0,1920,1200 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=36,498,768,324 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,498,768,324
                    10 ListBoxItem at=36,498,768,81 by=fixed-scroll drawn focus heard="Start a Console" in=9 limit=36,498,768,324 op paints=#333337FF,#D4D4D4FF role=listitem text="Start a Console  ›"
                    11 ListBoxItem at=36,579,768,81 by=fixed-scroll drawn focus heard=Tools in=9 limit=36,498,768,324 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                    12 ListBoxItem at=36,660,768,81 by=fixed-scroll drawn focus heard=Settings in=9 limit=36,498,768,324 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                    13 ListBoxItem at=36,741,768,81 by=fixed-scroll drawn focus heard=Power in=9 limit=36,498,768,324 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
            14 Border at=840,36,1044,1042 by=panel limit=0,0,1920,1200 name=AdaptiveCardPane role=none
              15 Border at=840,36,1044,1042 by=panel limit=0,0,1920,1200 name=AdaptiveDetail role=none
focus none
press pageup menu="open at 0: Start a Console" top=none
press pagedown menu="open at 0: Start a Console" top=none
press first menu="open at 0: Start a Console" top=none
press last menu="open at 0: Start a Console" top=none
press back top=none
