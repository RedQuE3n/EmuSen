# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusIdle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,312.5,1728,575
title Scraping
case ScrapeStatusIdle
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,462,1728,396 by=panel limit=96,462,1728,395 paints=#D4D4D4FF role=none seen=96,462,1728,395
    1 Grid at=132,480,1656,354 by=panel limit=96,462,1728,395 role=none
      2 Grid at=132,480,1656,270 by=panel limit=96,462,1728,395 role=none
        3 ScrollViewer at=132,480,1656,270 by=panel limit=96,462,1728,395 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=132,480,1656,270
          4 StackPanel at=132,480,1656,270 by=fixed-scroll in=3 limit=132,480,1656,270 role=none
            5 StackPanel at=132,480,1656,102 by=fixed-scroll in=3 limit=132,480,1656,270 role=none
              6 TextBlock at=132,480,1656,54 box=1104,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="No scraping in progress." in=3 layout=237.36,36 limit=132,480,1656,270 name=ScrapeStatusHeading paints=#D4D4D4FF role=text size=30 text="No scraping in progress."
              7 HintText at=132,543,1656,39 box=1104,26 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping." in=3 layout=649.7,25.7 limit=132,480,1656,270 name=ScrapeStatusTiming paints=#8D8D91FF role=text size=20 text="Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping."
            8 StackPanel at=132,600,1656,148.5 by=fixed-scroll in=3 limit=132,480,1656,270 role=none
              9 SectionHeader at=132,600,1656,51 box=1104,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=56.95,33.6 limit=132,480,1656,270 paints=#D4D4D4FF role=text size=28 text=Quota
              10 StackPanel at=132,663,1656,85.5 by=fixed-scroll in=3 limit=132,480,1656,270 role=none
                11 TextBlock at=132,663,1656,43.5 box=1104,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=594.58,28.8 limit=132,480,1656,270 name=ScrapeStatusMember paints=#D4D4D4FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                12 HintText at=132,712.5,1656,36 box=1104,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute." in=3 layout=639.34,24 limit=132,480,1656,270 name=ScrapeStatusLimits paints=#8D8D91FF role=text size=20 text="The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute."
      13 ButtonBar at=894,768,130.5,66 by=panel limit=96,462,1728,395 paints=#D4D4D4FF role=toolbar
        14 Button at=894,768,130.5,66 by=clip drawn focus heard=CLOSE limit=894,768,130.5,66 name=ScrapeStatusHideButton op paints=#707070FF,#D4D4D4FF role=button text=Close
          15 TextBlock at=919.5,777,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=894,768,130.5,66 owner=14 paints=#EEEEF0FF role=text size=26 text=CLOSE
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
