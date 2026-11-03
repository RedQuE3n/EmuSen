# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktopToolsPage-1280x800
size 1280 800
kind menu
root Border#PadMenuPanel
unit 1
slack 1.5
floor 16
frame 0,0,1280,768
case HomeCardDesktopToolsPage
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 role=none
    1 AdaptiveLayout at=0,0,1280,768 by=panel heard=Home limit=0,0,1280,768 name=PadCard role=pane
      2 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 name=AdaptiveLayers role=none
        3 Panel at=0,0,1280,768 by=panel limit=0,0,1280,768 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1280,768 by=panel heard=Tools limit=0,0,1280,768 name=AdaptiveCard role=menu
            5 HintBar at=64,704,350,30 by=panel heard="A Choose, B Back, Menu Close" limit=0,0,1280,768 role=group
            6 Border at=64,156,424,324 by=panel limit=0,0,1280,768 name=AdaptiveCardRows role=none
              7 Border at=64,156,424,324 by=panel limit=0,0,1280,768 name=AdaptiveContent role=none
                8 LunaList at=64,156,424,324 by=panel heard=Menu limit=0,0,1280,768 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=64,156,424,324 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=64,156,424,324
                    10 ListBoxItem at=64,156,424,54 by=fixed-scroll drawn focus heard=Scraper in=9 limit=64,156,424,324 op paints=#333337FF,#D4D4D4FF role=listitem text=Scraper
                    11 ListBoxItem at=64,210,424,54 by=fixed-scroll drawn focus heard=Packages in=9 limit=64,156,424,324 op paints=#333337FF,#D4D4D4FF role=listitem text=Packages
                    12 ListBoxItem at=64,264,424,54 by=fixed-scroll drawn focus heard="Library Manager" in=9 limit=64,156,424,324 op paints=#333337FF,#D4D4D4FF role=listitem text="Library Manager"
                    13 ListBoxItem at=64,318,424,54 by=fixed-scroll drawn focus heard=Cores in=9 limit=64,156,424,324 op paints=#333337FF,#D4D4D4FF role=listitem text=Cores
                    14 ListBoxItem at=64,372,424,54 by=fixed-scroll drawn focus heard="Compressed Discs" in=9 limit=64,156,424,324 op paints=#333337FF,#D4D4D4FF role=listitem text="Compressed Discs"
                    15 ListBoxItem at=64,426,424,54 by=fixed-scroll drawn focus heard=Shell in=9 limit=64,156,424,324 op paints=#333337FF,#D4D4D4FF role=listitem text=Shell
            16 Border at=576,156,640,498 by=panel limit=0,0,1280,768 name=AdaptiveCardPane paints=#26262BFF role=none
              17 Border at=600,180,592,450 by=panel limit=0,0,1280,768 name=AdaptiveDetail role=none
                18 PauseCardPane at=600,180,592,53 by=panel limit=0,0,1280,768 name=PauseCardPane role=none
                  19 StackPanel at=600,180,592,53 by=panel limit=0,0,1280,768 name=PaneLines role=none
                    20 StackPanel at=600,180,592,53 by=panel limit=0,0,1280,768 role=none
                      21 TextBlock at=600,180,592,27 box=592,27 by=panel caps=15.3989 drawn face=Barlow heard=Scraper layout=73.26,26.4 limit=0,0,1280,768 paints=#EEEEF0FF role=text size=22 text=Scraper
                      22 TextBlock at=600,209,592,24 box=592,24 by=panel caps=13.999 drawn face=Barlow heard="41 games missing media" layout=203.44,24 limit=0,0,1280,768 paints=#8D8D91FF role=text size=20 text="41 games missing media"
focus none
press down menu="open at 1: Packages" top=none
press up menu="open at 0: Scraper" top=none
press back menu="open at 4: Tools" top=none
