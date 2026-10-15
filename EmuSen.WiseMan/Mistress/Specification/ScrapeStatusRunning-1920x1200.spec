# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusRunning-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,68,1728,971
title Scraping
case ScrapeStatusRunning
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,218,1728,792 by=panel limit=96,218,1728,791 paints=#9C9CA0FF role=none seen=96,218,1728,791
    1 Grid at=132,236,1656,750 by=panel limit=96,218,1728,791 role=none
      2 Grid at=132,236,1656,666 by=panel limit=96,218,1728,791 role=none
        3 ScrollViewer at=132,236,964.5,666 by=panel limit=96,218,1728,791 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=132,236,964.5,666
          4 StackPanel at=132,236,964.5,666 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
            5 StackPanel at=132,236,964.5,114 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              6 TextBlock at=132,236,964.5,54 box=643,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="Scraping 3 of 6 games" in=3 layout=226.71,36 limit=132,236,964.5,666 name=ScrapeStatusHeading paints=#9C9CA0FF role=text size=30 text="Scraping 3 of 6 games"
              7 ProgressBar at=132,299,964.5,6 by=fixed-scroll drawn in=3 limit=132,236,964.5,666 name=ScrapeStatusProgress paints=#333337FF,#4C9AE8FF role=progressbar
              8 HintText at=132,314,964.5,36 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Elapsed 0:18 · about 0:18 left at this run's pace" in=3 layout=311.9,24 limit=132,236,964.5,666 name=ScrapeStatusTiming paints=#86868AFF role=text size=20 text="Elapsed 0:18 · about 0:18 left at this run's pace"
            9 Grid at=132,368,964.5,142.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              10 StackPanel at=150,368,946.5,142.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
                11 TextBlock at=150,368,946.5,51 box=631,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard="Cobalt Harbor (USA)" in=3 layout=184.41,33.6 limit=132,236,964.5,666 name=ScrapeStatusGame paints=#9C9CA0FF role=text size=28 text="Cobalt Harbor (USA)"
                12 HintText at=150,425,946.5,36 box=631,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="SNES (Venus)" in=3 layout=88.42,24 limit=132,236,964.5,666 name=ScrapeStatusConsole paints=#86868AFF role=text size=20 text="SNES (Venus)"
                13 TextBlock at=150,467,946.5,43.5 box=631,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Downloading its cover" in=3 layout=178.66,28.8 limit=132,236,964.5,666 name=ScrapeStatusStep paints=#9C9CA0FF role=text size=24 text="Downloading its cover"
            14 StackPanel at=132,528.5,964.5,106.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              15 SectionHeader at=132,528.5,964.5,51 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Results in=3 layout=82.21,33.6 limit=132,236,964.5,666 paints=#B4B4B8FF role=text size=28 text=Results
              16 StackPanel at=132,591.5,964.5,43.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
                17 TextBlock at=132,591.5,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0" in=3 layout=526.01,28.8 limit=132,236,964.5,666 name=ScrapeStatusTallies paints=#9C9CA0FF role=text size=24 text="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0"
            18 StackPanel at=132,653,964.5,247.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              19 SectionHeader at=132,653,964.5,51 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=60.68,33.6 limit=132,236,964.5,666 paints=#B4B4B8FF role=text size=28 text=Quota
              20 StackPanel at=132,716,964.5,184.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
                21 TextBlock at=132,716,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=623.06,28.8 limit=132,236,964.5,666 name=ScrapeStatusMember paints=#9C9CA0FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                22 MeterRow at=132,765.5,964.5,43.5 by=fixed-scroll drawn heard="Requests today" in=3 limit=132,236,964.5,666 name=ScrapeStatusQuota paints=#9C9CA0FF role=progressbar
                  23 TextBlock at=132,765.5,210,43.5 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Requests today" in=3 layout=124.85,28.8 limit=132,765.5,964.5,43.5 paints=#B4B4B8FF part role=text size=24 text="Requests today"
                  24 ProgressBar at=342,777.5,672,21 drawn in=3 name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                  25 TextBlock at=1026,765.5,70.5,43.5 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=0% in=3 layout=29.4,28.8 limit=132,765.5,964.5,43.5 paints=#B4B4B8FF part role=text size=24 text=0%
                26 TextBlock at=132,815,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="11 of 20,000 requests used today · 19,989 left" in=3 layout=359.18,28.8 limit=132,236,964.5,666 name=ScrapeStatusRequests paints=#9C9CA0FF role=text size=24 text="11 of 20,000 requests used today · 19,989 left"
                27 HintText at=132,864.5,964.5,36 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute" in=3 layout=480.38,24 limit=132,236,964.5,666 name=ScrapeStatusLimits paints=#86868AFF role=text size=20 text="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute"
        28 Grid at=1144.5,236,643.5,666 by=panel limit=96,218,1728,791 role=none
          29 SectionHeader at=1144.5,236,643.5,51 box=429,34 by=panel caps=19.5986 drawn face="Barlow Condensed" heard="Recent games" layout=142.41,33.6 limit=96,218,1728,791 paints=#B4B4B8FF role=text size=28 text="Recent games"
          30 LunaList at=1144.5,287,643.5,615 by=panel limit=96,218,1728,791 name=ScrapeStatusRecent paints=#FFFFFFFF role=list
            31 ScrollViewer at=1144.5,287,643.5,615 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=1144.5,287,643.5,615
              32 ListBoxItem at=1144.5,287,643.5,63 by=fixed-scroll drawn focus heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=1144.5,287,643.5,615 op paints=#333337FF,#9C9CA0FF role=listitem text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                33 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,296,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=405.26,28.8 owner=32 paints=#9C9CA0FF role=text size=24 text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              34 ListBoxItem at=1144.5,350,643.5,63 by=fixed-scroll drawn focus heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=1144.5,287,643.5,615 op paints=#333337FF,#9C9CA0FF role=listitem text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                35 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,359,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=403.2,28.8 owner=34 paints=#9C9CA0FF role=text size=24 text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              36 ListBoxItem at=1144.5,413,643.5,63 by=fixed-scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=1144.5,287,643.5,615 op paints=#333337FF,#9C9CA0FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                37 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,422,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=400.97,28.8 owner=36 paints=#9C9CA0FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
      38 ButtonBar at=735,920,448.5,66 by=panel limit=96,218,1728,791 paints=#9C9CA0FF role=toolbar
        39 Button at=735,920,135,66 by=clip drawn focus heard=PAUSE limit=735,920,448.5,66 name=ScrapeStatusPauseButton op paints=#55555AFF,#9C9CA0FF role=button text=Pause
          40 TextBlock at=760.5,929,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PAUSE layout=55.64,31.2 limit=735,920,135,66 owner=39 paints=#9C9CA0FF role=text size=26 text=PAUSE
        41 Button at=882,920,177,66 by=clip drawn focus heard=CANCEL... limit=735,920,448.5,66 name=ScrapeStatusCancelButton op paints=#55555AFF,#9C9CA0FF role=button text=Cancel...
          42 TextBlock at=907.5,929,126,48 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL... layout=83.2,31.2 limit=882,920,177,66 owner=41 paints=#9C9CA0FF role=text size=26 text=CANCEL...
        43 Button at=1071,920,112.5,66 by=clip drawn focus heard=HIDE limit=735,920,448.5,66 name=ScrapeStatusHideButton op paints=#55555AFF,#9C9CA0FF role=button text=Hide
          44 TextBlock at=1096.5,929,61.5,48 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=40.92,31.2 limit=1071,920,112.5,66 owner=43 paints=#EEEEF0FF role=text size=26 text=HIDE
focus 43
edge 32 up same
edge 32 down 34
edge 32 left 41
edge 32 right same
edge 34 up 32
edge 34 down 36
edge 34 left 41
edge 34 right same
edge 36 up 34
edge 36 down 43
edge 36 left 39
edge 36 right same
edge 39 up 36
edge 39 down same
edge 39 left same
edge 39 right 41
edge 41 up 36
edge 41 down same
edge 41 left 39
edge 41 right 43
edge 43 up 36
edge 43 down same
edge 43 left 41
edge 43 right same
press pageup focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press pagedown focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press first focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press last focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press back top=none
