# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusIdle-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,311,1728,578
title Scraping
case ScrapeStatusIdle
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,461,1728,399 by=panel limit=96,461,1728,398 paints=#9C9CA0FF role=none seen=96,461,1728,398
    1 Grid at=132,479,1656,357 by=panel limit=96,461,1728,398 role=none
      2 Grid at=132,479,1656,273 by=panel limit=96,461,1728,398 role=none
        3 ScrollViewer at=132,479,1656,273 by=panel limit=96,461,1728,398 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=132,479,1656,273
          4 StackPanel at=132,479,1656,273 by=fixed-scroll in=3 limit=132,479,1656,273 role=none
            5 StackPanel at=132,479,1656,105 by=fixed-scroll in=3 limit=132,479,1656,273 role=none
              6 TextBlock at=132,479,1656,54 box=1104,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="No scraping in progress." in=3 layout=247.26,36 limit=132,479,1656,273 name=ScrapeStatusHeading paints=#9C9CA0FF role=text size=30 text="No scraping in progress."
              7 HintText at=132,542,1656,42 box=1104,28 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping." in=3 layout=679.68,27.24 limit=132,479,1656,273 name=ScrapeStatusTiming paints=#86868AFF role=text size=20 text="Nothing is sent from this window. A run starts from Scrape This Game... or Preferences ▸ Scraping."
            8 StackPanel at=132,602,1656,148.5 by=fixed-scroll in=3 limit=132,479,1656,273 role=none
              9 SectionHeader at=132,602,1656,51 box=1104,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=60.68,33.6 limit=132,479,1656,273 paints=#B4B4B8FF role=text size=28 text=Quota
              10 StackPanel at=132,665,1656,85.5 by=fixed-scroll in=3 limit=132,479,1656,273 role=none
                11 TextBlock at=132,665,1656,43.5 box=1104,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=623.06,28.8 limit=132,479,1656,273 name=ScrapeStatusMember paints=#9C9CA0FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                12 HintText at=132,714.5,1656,36 box=1104,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute." in=3 layout=668.22,24 limit=132,479,1656,273 name=ScrapeStatusLimits paints=#86868AFF role=text size=20 text="The limits are read from ScreenScraper's first answer: until then one thread, 30 requests a minute."
      13 ButtonBar at=892.5,770,135,66 by=panel limit=96,461,1728,398 paints=#9C9CA0FF role=toolbar
        14 Button at=892.5,770,135,66 by=clip drawn focus heard=CLOSE limit=892.5,770,135,66 name=ScrapeStatusHideButton op paints=#55555AFF,#9C9CA0FF role=button text=Close
          15 TextBlock at=918,779,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=892.5,770,135,66 owner=14 paints=#EEEEF0FF role=text size=26 text=CLOSE
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
