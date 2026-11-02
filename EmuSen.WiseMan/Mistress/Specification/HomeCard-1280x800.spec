# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCard-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,774
case HomeCard
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 role=none
    1 AdaptiveLayout at=0,0,1280,774 by=panel heard=Home limit=0,0,1280,774 name=PadCard role=pane
      2 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,774 by=panel limit=0,0,1280,774 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,774 by=panel heard=Home limit=0,0,1280,774 name=AdaptiveCard role=menu
            5 HintBar at=386,716,508,48 by=panel heard="A Select, B Back, Menu Close Menu, Y Favourite" limit=0,0,1280,774 role=group
            6 Border at=24,238,512,378 by=panel limit=0,0,1280,774 name=AdaptiveCardRows role=none
              7 Border at=24,238,512,378 by=panel limit=0,0,1280,774 name=AdaptiveContent role=none
                8 LunaList at=24,238,512,378 by=panel heard=Menu limit=0,0,1280,774 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=24,238,512,378 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=24,238,512,378
                    10 ListBoxItem at=24,238,512,54 by=fixed-scroll drawn focus heard=Continue in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Continue
                    11 ListBoxItem at=24,292,512,54 by=fixed-scroll drawn focus heard="Cobalt Harbor (Synthetic)" in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Cobalt Harbor (Synthetic)  ›"
                    12 ListBoxItem at=24,346,512,54 by=fixed-scroll drawn focus heard=Search in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                    13 ListBoxItem at=24,400,512,54 by=fixed-scroll drawn focus heard="Start a Console" in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Start a Console  ›"
                    14 ListBoxItem at=24,454,512,54 by=fixed-scroll drawn focus heard=Tools in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                    15 ListBoxItem at=24,508,512,54 by=fixed-scroll drawn focus heard=Settings in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                    16 ListBoxItem at=24,562,512,54 by=fixed-scroll drawn focus heard=Power in=9 limit=24,238,512,378 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
            17 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveCardPane role=none
              18 Border at=560,24,696,668 by=panel limit=0,0,1280,774 name=AdaptiveDetail role=none
                19 PauseCardPane at=736,174,343,367 by=panel limit=0,0,1280,774 name=PauseCardPane role=none
                  20 StackPanel at=736,174,343,367 by=panel limit=0,0,1280,774 role=none
                    21 Image at=736,174,342.86,300 by=panel drawn limit=0,0,1280,774 name=PanePicture role=image
                    22 TextBlock at=736,482,343,27 box=343,27 by=panel caps=15.3989 drawn face=Barlow heard="Cobalt Harbor (Synthetic)" layout=231.55,26.4 limit=0,0,1280,774 name=PaneTitle paints=#EEEEF0FF role=text size=22 text="Cobalt Harbor (Synthetic)"
                    23 TextBlock at=736,517,343,24 box=343,24 by=panel caps=13.999 drawn face=Barlow heard="Paused just now" layout=137.08,24 limit=0,0,1280,774 name=PaneCaption paints=#8D8D91FF role=text size=20 text="Paused just now"
focus none
press down menu="open at 1: Cobalt Harbor (Synthetic)" top=none
press up menu="open at 0: Continue" top=none
press back top=none
