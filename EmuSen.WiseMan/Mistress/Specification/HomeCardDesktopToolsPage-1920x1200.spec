# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state HomeCardDesktopToolsPage-1920x1200
size 1920 1200
kind menu
root Border#PadMenuPanel
unit 1.5
slack 2
floor 16
frame 0,0,1920,1168
case HomeCardDesktopToolsPage
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the pad moves the menu's own selection, not the focus
tree
  0 Panel at=0,0,1920,1168 by=panel limit=0,0,1920,1168 role=none
    1 AdaptiveLayout at=0,0,1920,1168 by=panel heard=Home limit=0,0,1920,1168 name=PadCard role=pane
      2 Panel at=0,0,1920,1168 by=panel limit=0,0,1920,1168 name=AdaptiveLayers role=none
        3 Panel at=0,0,1920,1168 by=panel limit=0,0,1920,1168 name=AdaptiveCardScrim paints=#0000008C role=none
          4 MenuPanel at=0,0,1920,1168 by=panel heard=Tools limit=0,0,1920,1168 name=AdaptiveCard role=menu
            5 HintBar at=96,1072,525,45 by=panel heard="A Choose, B Back, Menu Close" limit=0,0,1920,1168 role=group
            6 Border at=96,234,636,729 by=panel limit=0,0,1920,1168 name=AdaptiveCardRows role=none
              7 Border at=96,234,636,729 by=panel limit=0,0,1920,1168 name=AdaptiveContent role=none
                8 LunaList at=96,234,636,729 by=panel heard=Menu limit=0,0,1920,1168 name=PadMenuList paints=#FFFFFFFF role=list
                  9 ScrollViewer at=96,234,636,729 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=96,234,636,729
                    10 ListBoxItem at=96,234,636,81 by=fixed-scroll drawn focus heard=Scraper in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Scraper
                    11 ListBoxItem at=96,315,636,81 by=fixed-scroll drawn focus heard=Packages in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Packages
                    12 ListBoxItem at=96,396,636,81 by=fixed-scroll drawn focus heard="Library Manager" in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text="Library Manager"
                    13 ListBoxItem at=96,477,636,81 by=fixed-scroll drawn focus heard=Cores in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Cores
                    14 ListBoxItem at=96,558,636,81 by=fixed-scroll drawn focus heard=Cheats in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Cheats
                    15 ListBoxItem at=96,639,636,81 by=fixed-scroll drawn focus heard=Themes in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Themes
                    16 ListBoxItem at=96,720,636,81 by=fixed-scroll drawn focus heard=Collections in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Collections
                    17 ListBoxItem at=96,801,636,81 by=fixed-scroll drawn focus heard="Compressed Discs" in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text="Compressed Discs"
                    18 ListBoxItem at=96,882,636,81 by=fixed-scroll drawn focus heard=Shell in=9 limit=96,234,636,729 op paints=#333337FF,#D4D4D4FF role=listitem text=Shell
            19 Border at=864,234,960,763 by=panel limit=0,0,1920,1168 name=AdaptiveCardPane paints=#26262BFF role=none
              20 Border at=900,270,888,691 by=panel limit=0,0,1920,1168 name=AdaptiveDetail role=none
                21 PauseCardPane at=900,270,888,178 by=panel limit=0,0,1920,1168 name=PauseCardPane role=none
                  22 StackPanel at=900,270,888,178 by=panel limit=0,0,1920,1168 name=PaneSaid role=none
                    23 TextBlock at=900,270,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard=Scraper layout=109.89,39.6 limit=0,0,1920,1168 name=PaneTitle paints=#D4D4D4FF role=text size=22 text=Scraper
                    24 TextBlock at=900,319,888,80 box=888,80 by=panel caps=15.3989 drawn face=Barlow heard="41 games have no art yet; the Scraper fetches it from ScreenScraper." layout=721.15,79.2 limit=0,0,1920,1168 name=PaneSentence paints=#D4D4D4FF role=text size=22 text="41 games have no art yet; the Scraper fetches it from ScreenScraper."
                    25 TextBlock at=900,408,888,40 box=888,40 by=panel caps=15.3989 drawn face=Barlow heard="A opens Scraper; B comes back here." layout=513.05,39.6 limit=0,0,1920,1168 name=PaneHint paints=#8D8D91FF role=text size=22 text="A opens Scraper; B comes back here."
focus none
press down menu="open at 1: Packages" top=none
press up menu="open at 0: Scraper" top=none
press back menu="open at 4: Tools" top=none
