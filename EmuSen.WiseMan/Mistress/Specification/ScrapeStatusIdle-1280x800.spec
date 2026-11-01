# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusIdle-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,208.5,1152,383
title Scraping
case ScrapeStatusIdle
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,308,1152,263 by=panel limit=64,308,1152,263 paints=#D4D4D4FF role=none
    1 Grid at=88,320,1104,235 by=panel limit=64,308,1152,263 role=none
      2 Grid at=88,320,1104,179 by=panel limit=64,308,1152,263 role=none
        3 ScrollViewer at=88,320,1104,179 by=panel limit=64,308,1152,263 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=88,320,1104,179
          4 StackPanel at=88,320,1104,179 by=fixed-scroll in=3 limit=88,320,1104,179 role=none
            5 StackPanel at=88,320,1104,68 by=fixed-scroll in=3 limit=88,320,1104,179 role=none
              6 TextBlock at=88,320,1104,36 box=1104,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="No scraping in progress." in=3 layout=237.36,36 limit=88,320,1104,179 name=ScrapeStatusHeading paints=#D4D4D4FF role=text size=30 text="No scraping in progress."
              7 HintText at=88,362,1104,26 box=1104,26 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping." in=3 layout=649.7,25.7 limit=88,320,1104,179 name=ScrapeStatusTiming paints=#8D8D91FF role=text size=20 text="Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping."
            8 StackPanel at=88,400,1104,99 by=fixed-scroll in=3 limit=88,320,1104,179 role=none
              9 SectionHeader at=88,400,1104,34 box=1104,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=56.95,33.6 limit=88,320,1104,179 paints=#D4D4D4FF role=text size=28 text=Quota
              10 StackPanel at=88,442,1104,57 by=fixed-scroll in=3 limit=88,320,1104,179 role=none
                11 TextBlock at=88,442,1104,29 box=1104,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=594.58,28.8 limit=88,320,1104,179 name=ScrapeStatusMember paints=#D4D4D4FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                12 HintText at=88,475,1104,24 box=1104,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute." in=3 layout=639.34,24 limit=88,320,1104,179 name=ScrapeStatusLimits paints=#8D8D91FF role=text size=20 text="The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute."
      13 ButtonBar at=596,511,87,44 by=panel limit=64,308,1152,263 paints=#D4D4D4FF role=toolbar
        14 Button at=596,511,87,44 by=clip drawn focus heard=CLOSE limit=596,511,87,44 name=ScrapeStatusHideButton op paints=#707070FF,#D4D4D4FF role=button text=Close
          15 TextBlock at=613,517,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=596,511,87,44 owner=14 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 14
edge 14 up same
edge 14 down same
edge 14 left same
edge 14 right same
press pageup focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press pagedown focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press first focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press last focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press back top=none
