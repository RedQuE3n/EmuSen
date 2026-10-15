# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,45,1152,647
title Scraping
case ScrapeStatusRunning
palette #000000AA #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,145,1152,527 by=panel limit=64,145,1152,527 paints=#9C9CA0FF role=none
    1 Grid at=88,157,1104,499 by=panel limit=64,145,1152,527 role=none
      2 Grid at=88,157,1104,443 by=panel limit=64,145,1152,527 role=none
        3 ScrollViewer at=88,157,643,443 by=panel limit=64,145,1152,527 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=88,157,643,443
          4 StackPanel at=88,157,643,443 by=fixed-scroll in=3 limit=88,157,643,443 role=none
            5 StackPanel at=88,157,643,76 by=fixed-scroll in=3 limit=88,157,643,443 role=none
              6 TextBlock at=88,157,643,36 box=643,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="Scraping 3 of 6 games" in=3 layout=226.71,36 limit=88,157,643,443 name=ScrapeStatusHeading paints=#9C9CA0FF role=text size=30 text="Scraping 3 of 6 games"
              7 ProgressBar at=88,199,643,4 by=fixed-scroll drawn in=3 limit=88,157,643,443 name=ScrapeStatusProgress paints=#333337FF,#4C9AE8FF role=progressbar
              8 HintText at=88,209,643,24 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Elapsed 0:18 · about 0:18 left at this run's pace" in=3 layout=311.9,24 limit=88,157,643,443 name=ScrapeStatusTiming paints=#86868AFF role=text size=20 text="Elapsed 0:18 · about 0:18 left at this run's pace"
            9 Grid at=88,245,643,95 by=fixed-scroll in=3 limit=88,157,643,443 role=none
              10 StackPanel at=100,245,631,95 by=fixed-scroll in=3 limit=88,157,643,443 role=none
                11 TextBlock at=100,245,631,34 box=631,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard="Cobalt Harbor (USA)" in=3 layout=184.41,33.6 limit=88,157,643,443 name=ScrapeStatusGame paints=#9C9CA0FF role=text size=28 text="Cobalt Harbor (USA)"
                12 HintText at=100,283,631,24 box=631,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="SNES (Venus)" in=3 layout=88.42,24 limit=88,157,643,443 name=ScrapeStatusConsole paints=#86868AFF role=text size=20 text="SNES (Venus)"
                13 TextBlock at=100,311,631,29 box=631,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Downloading its cover" in=3 layout=178.66,28.8 limit=88,157,643,443 name=ScrapeStatusStep paints=#9C9CA0FF role=text size=24 text="Downloading its cover"
            14 StackPanel at=88,352,643,71 by=fixed-scroll in=3 limit=88,157,643,443 role=none
              15 SectionHeader at=88,352,643,34 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Results in=3 layout=82.21,33.6 limit=88,157,643,443 paints=#B4B4B8FF role=text size=28 text=Results
              16 StackPanel at=88,394,643,29 by=fixed-scroll in=3 limit=88,157,643,443 role=none
                17 TextBlock at=88,394,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0" in=3 layout=526.01,28.8 limit=88,157,643,443 name=ScrapeStatusTallies paints=#9C9CA0FF role=text size=24 text="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0"
            18 StackPanel at=88,435,643,165 by=fixed-scroll in=3 limit=88,157,643,443 role=none
              19 SectionHeader at=88,435,643,34 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=60.68,33.6 limit=88,157,643,443 paints=#B4B4B8FF role=text size=28 text=Quota
              20 StackPanel at=88,477,643,123 by=fixed-scroll in=3 limit=88,157,643,443 role=none
                21 TextBlock at=88,477,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=623.06,28.8 limit=88,157,643,443 name=ScrapeStatusMember paints=#9C9CA0FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                22 MeterRow at=88,510,643,29 by=fixed-scroll drawn heard="Requests today" in=3 limit=88,157,643,443 name=ScrapeStatusQuota paints=#9C9CA0FF role=progressbar
                  23 TextBlock at=88,510,140,29 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Requests today" in=3 layout=124.85,28.8 limit=88,510,643,29 paints=#B4B4B8FF part role=text size=24 text="Requests today"
                  24 ProgressBar at=228,518,448,14 drawn in=3 name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                  25 TextBlock at=684,510,47,29 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=0% in=3 layout=29.4,28.8 limit=88,510,643,29 paints=#B4B4B8FF part role=text size=24 text=0%
                26 TextBlock at=88,543,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="11 of 20,000 requests used today · 19,989 left" in=3 layout=359.18,28.8 limit=88,157,643,443 name=ScrapeStatusRequests paints=#9C9CA0FF role=text size=24 text="11 of 20,000 requests used today · 19,989 left"
                27 HintText at=88,576,643,24 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute" in=3 layout=480.38,24 limit=88,157,643,443 name=ScrapeStatusLimits paints=#86868AFF role=text size=20 text="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute"
        28 Grid at=763,157,429,443 by=panel limit=64,145,1152,527 role=none
          29 SectionHeader at=763,157,429,34 box=429,34 by=panel caps=19.5986 drawn face="Barlow Condensed" heard="Recent games" layout=142.41,33.6 limit=64,145,1152,527 paints=#B4B4B8FF role=text size=28 text="Recent games"
          30 LunaList at=763,191,429,409 by=panel limit=64,145,1152,527 name=ScrapeStatusRecent paints=#FFFFFFFF role=list
            31 ScrollViewer at=763,191,429,409 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=763,191,429,409
              32 ListBoxItem at=763,191,429,42 by=fixed-scroll drawn focus heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=763,191,429,409 op paints=#333337FF,#9C9CA0FF role=listitem text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                33 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,197,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=405.26,28.8 owner=32 paints=#9C9CA0FF role=text size=24 text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              34 ListBoxItem at=763,233,429,42 by=fixed-scroll drawn focus heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=763,191,429,409 op paints=#333337FF,#9C9CA0FF role=listitem text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                35 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,239,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=403.2,28.8 owner=34 paints=#9C9CA0FF role=text size=24 text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              36 ListBoxItem at=763,275,429,42 by=fixed-scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=763,191,429,409 op paints=#333337FF,#9C9CA0FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                37 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,281,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=400.97,28.8 owner=36 paints=#9C9CA0FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
      38 ButtonBar at=490,612,299,44 by=panel limit=64,145,1152,527 paints=#9C9CA0FF role=toolbar
        39 Button at=490,612,90,44 by=clip drawn focus heard=PAUSE limit=490,612,299,44 name=ScrapeStatusPauseButton op paints=#55555AFF,#9C9CA0FF role=button text=Pause
          40 TextBlock at=507,618,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PAUSE layout=55.64,31.2 limit=490,612,90,44 owner=39 paints=#9C9CA0FF role=text size=26 text=PAUSE
        41 Button at=588,612,118,44 by=clip drawn focus heard=CANCEL... limit=490,612,299,44 name=ScrapeStatusCancelButton op paints=#55555AFF,#9C9CA0FF role=button text=Cancel...
          42 TextBlock at=605,618,84,32 box=84,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL... layout=83.2,31.2 limit=588,612,118,44 owner=41 paints=#9C9CA0FF role=text size=26 text=CANCEL...
        43 Button at=714,612,75,44 by=clip drawn focus heard=HIDE limit=490,612,299,44 name=ScrapeStatusHideButton op paints=#55555AFF,#9C9CA0FF role=button text=Hide
          44 TextBlock at=731,618,41,32 box=41,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=40.92,31.2 limit=714,612,75,44 owner=43 paints=#EEEEF0FF role=text size=26 text=HIDE
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
