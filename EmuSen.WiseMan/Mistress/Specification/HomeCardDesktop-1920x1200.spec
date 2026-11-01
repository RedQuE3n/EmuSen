# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktop-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1146
case HomeCardDesktop
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1146 by=panel limit=0,0,1920,1146 role=none
    1 AdaptiveLayout at=0,0,1920,1146 by=panel heard=Home limit=0,0,1920,1146 name=PadCard role=pane
      2 Panel at=0,0,1920,1146 by=panel limit=0,0,1920,1146 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1920,1146 by=panel heard=Home limit=0,0,1920,1146 name=AdaptiveCard role=menu
          4 HintBar at=579,1060,762,71 by=panel heard="A Select, B Back, Menu Close Menu, Y Favourite" limit=0,0,1920,1146 role=group
          5 Border at=36,350,768,567 by=panel limit=0,0,1920,1146 name=AdaptiveCardRows role=none
            6 Border at=36,350,768,567 by=panel limit=0,0,1920,1146 name=AdaptiveContent role=none
              7 LunaList at=36,350,768,567 by=panel heard=Menu limit=0,0,1920,1146 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=36,350,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,350,768,567
                  9 ListBoxItem at=36,350,768,81 by=fixed-scroll drawn focus heard=Continue in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Continue
                  10 ListBoxItem at=36,431,768,81 by=fixed-scroll drawn focus heard="Cobalt Harbor (Synthetic)" in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Cobalt Harbor (Synthetic)  ›"
                  11 ListBoxItem at=36,512,768,81 by=fixed-scroll drawn focus heard=Search in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Search
                  12 ListBoxItem at=36,593,768,81 by=fixed-scroll drawn focus heard="Start a Console" in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Start a Console  ›"
                  13 ListBoxItem at=36,674,768,81 by=fixed-scroll drawn focus heard=Tools in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Tools  ›"
                  14 ListBoxItem at=36,755,768,81 by=fixed-scroll drawn focus heard=Settings in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Settings  ›"
                  15 ListBoxItem at=36,836,768,81 by=fixed-scroll drawn focus heard=Power in=8 limit=36,350,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Power  ›"
          16 Border at=840,36,1044,988 by=panel limit=0,0,1920,1146 name=AdaptiveCardPane role=none
            17 Border at=840,36,1044,988 by=panel limit=0,0,1920,1146 name=AdaptiveDetail role=none
              18 PauseCardPane at=1104,255,515,550 by=panel limit=0,0,1920,1146 name=PauseCardPane role=none
                19 StackPanel at=1104,255,515,550 by=panel limit=0,0,1920,1146 role=none
                  20 Image at=1104,255,514.29,450 by=panel drawn limit=0,0,1920,1146 name=PanePicture role=image
                  21 TextBlock at=1104,717,515,40 box=515,40 by=panel caps=15.3989 drawn face=Barlow heard="Cobalt Harbor (Synthetic)" layout=347.33,39.6 limit=0,0,1920,1146 name=PaneTitle paints=#EEEEF0FF role=text size=22 text="Cobalt Harbor (Synthetic)"
                  22 TextBlock at=1104,769,515,36 box=515,36 by=panel caps=13.999 drawn face=Barlow heard="Paused just now" layout=205.62,36 limit=0,0,1920,1146 name=PaneCaption paints=#8D8D91FF role=text size=20 text="Paused just now"
focus none
press down menu="open at 1: Cobalt Harbor (Synthetic)" top=none
press up menu="open at 0: Continue" top=none
press back top=none
