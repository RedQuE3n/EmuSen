# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardToolsPage-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1161
case HomeCardToolsPage
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 role=none
    1 AdaptiveLayout at=0,0,1920,1161 by=panel heard=Home limit=0,0,1920,1161 name=PadCard role=pane
      2 Panel at=0,0,1920,1161 by=panel limit=0,0,1920,1161 name=AdaptiveCardScrim paints=#0000008C role=none
        3 MenuPanel at=0,0,1920,1161 by=panel heard=Tools limit=0,0,1920,1161 name=AdaptiveCard role=menu
          4 HintBar at=682,1075,556,71 by=panel heard="A Select, B Back, Menu Close Menu" limit=0,0,1920,1161 role=group
          5 Border at=36,386,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveCardRows role=none
            6 Border at=36,386,768,567 by=panel limit=0,0,1920,1161 name=AdaptiveContent role=none
              7 LunaList at=36,386,768,567 by=panel heard=Menu limit=0,0,1920,1161 name=PadMenuList paints=#FFFFFFFF role=list
                8 ScrollViewer at=36,386,768,567 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=36,386,768,567
                  9 ListBoxItem at=36,386,768,81 by=fixed-scroll drawn focus heard=Scraper in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Scraper
                  10 ListBoxItem at=36,467,768,81 by=fixed-scroll drawn focus heard=Packages in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Packages
                  11 ListBoxItem at=36,548,768,81 by=fixed-scroll drawn focus heard="Library Manager" in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Library Manager"
                  12 ListBoxItem at=36,629,768,81 by=fixed-scroll drawn focus heard=Cores in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Cores
                  13 ListBoxItem at=36,710,768,81 by=fixed-scroll drawn focus heard="Compressed Discs" in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Compressed Discs"
                  14 ListBoxItem at=36,791,768,81 by=fixed-scroll drawn focus heard="Theme Settings" in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text="Theme Settings"
                  15 ListBoxItem at=36,872,768,81 by=fixed-scroll drawn focus heard=Shell in=8 limit=36,386,768,567 op paints=#333337FF,#D4D4D4FF role=listitem text=Shell
          16 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveCardPane role=none
            17 Border at=840,36,1044,1003 by=panel limit=0,0,1920,1161 name=AdaptiveDetail role=none
              18 PauseCardPane at=1002,498,720,79 by=panel limit=0,0,1920,1161 name=PauseCardPane role=none
                19 StackPanel at=1002,498,720,79 by=panel limit=0,0,1920,1161 name=PaneLines role=none
                  20 StackPanel at=1002,498,720,79 by=panel limit=0,0,1920,1161 role=none
                    21 TextBlock at=1002,498,720,40 box=720,40 by=panel caps=15.3989 drawn face=Barlow heard=Scraper layout=109.89,39.6 limit=0,0,1920,1161 paints=#EEEEF0FF role=text size=22 text=Scraper
                    22 TextBlock at=1002,541,720,36 box=720,36 by=panel caps=13.999 drawn face=Barlow heard="41 games missing media" layout=305.16,36 limit=0,0,1920,1161 paints=#8D8D91FF role=text size=20 text="41 games missing media"
focus none
press down menu="open at 1: Packages" top=none
press up menu="open at 0: Scraper" top=none
press back menu="open at 4: Tools" top=none
