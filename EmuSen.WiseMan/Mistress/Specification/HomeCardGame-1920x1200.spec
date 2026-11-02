# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardGame-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case HomeCardGame
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Home limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1161 by=panel heard=Home limit=0,0,1920,1161 name=AdaptiveCard role=menu
            5 HintBar at=579,1075,762,71 by=panel heard="A Select, B Back, Menu Close Menu, Y Favourite" limit=0,0,1920,1161 role=group
            6 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
              7 Border at=36,357,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
                8 LunaList at=36,357,768,567 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=36,357,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,357,768,567
                    10 ListBoxItem at=36,357,768,81 by=fixed-scroll drawn focus heard=Continue in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Continue
                    11 ListBoxItem at=36,438,768,81 by=fixed-scroll drawn focus heard="Cobalt Harbor (Synthetic)" in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Cobalt Harbor (Synthetic)"
                    12 ListBoxItem at=36,519,768,81 by=fixed-scroll drawn focus heard=Search in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                    13 ListBoxItem at=36,600,768,81 by=fixed-scroll drawn focus heard="Start a Console" in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Start a Console  ›"
                    14 ListBoxItem at=36,681,768,81 by=fixed-scroll drawn focus heard=Tools in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                    15 ListBoxItem at=36,762,768,81 by=fixed-scroll drawn focus heard=Settings in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                    16 ListBoxItem at=36,843,768,81 by=fixed-scroll drawn focus heard=Power in=9 limit=36,357,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
            17 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
              18 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
                19 PauseCardPane at=1188,494,348,88 by=panel limit=0,0,1920,1161 name=PauseCardPane role=none
                  20 StackPanel at=1188,494,348,88 by=panel limit=0,0,1920,1161 role=none
                    21 TextBlock at=1188,494,348,40 box=348,40 by=panel caps=15.3989 drawn face=Barlow heard="Cobalt Harbor (Synthetic)" layout=347.33,39.6 limit=0,0,1920,1161 name=PaneTitle paints=#EEEEF0FF role=text size=22 text="Cobalt Harbor (Synthetic)"
                    22 TextBlock at=1188,546,348,36 box=348,36 by=panel caps=13.999 drawn face=Barlow heard="SNES (Venus)" layout=168.48,36 limit=0,0,1920,1161 name=PaneCaption paints=#8D8D91FF role=text size=20 text="SNES (Venus)"
focus none
press down menu="open at 2: Search" top=none
press up menu="open at 1: Cobalt Harbor (Synthetic)" top=none
press back top=none
