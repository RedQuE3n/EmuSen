# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusFinished-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,86,1152,602
title Scraping
case ScrapeStatusFinished
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,186,1152,482 by=panel limit=64,186,1152,482 paints=#9C9CA0FF role=none
    1 Grid at=88,198,1104,454 by=panel limit=64,186,1152,482 role=none
      2 Grid at=88,198,1104,398 by=panel limit=64,186,1152,482 role=none
        3 ScrollViewer at=88,198,643,398 by=panel limit=64,186,1152,482 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=88,198,643,398
          4 StackPanel at=88,198,643,398 by=fixed-scroll in=3 limit=88,198,643,398 role=none
            5 StackPanel at=88,198,643,76 by=fixed-scroll in=3 limit=88,198,643,398 role=none
              6 TextBlock at=88,198,643,36 box=643,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="Finished: 6 of 6 games" in=3 layout=229.44,36 limit=88,198,643,398 name=ScrapeStatusHeading paints=#9C9CA0FF role=text size=30 text="Finished: 6 of 6 games"
              7 ProgressBar at=88,240,643,4 by=fixed-scroll drawn in=3 limit=88,198,643,398 name=ScrapeStatusProgress paints=#333337FF,#4C9AE8FF role=progressbar
              8 HintText at=88,250,643,24 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Took 0:28" in=3 layout=65.7,24 limit=88,198,643,398 name=ScrapeStatusTiming paints=#86868AFF role=text size=20 text="Took 0:28"
            9 StackPanel at=88,286,643,71 by=fixed-scroll in=3 limit=88,198,643,398 role=none
              10 SectionHeader at=88,286,643,34 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Results in=3 layout=82.21,33.6 limit=88,198,643,398 paints=#B4B4B8FF role=text size=28 text=Results
              11 StackPanel at=88,328,643,29 by=fixed-scroll in=3 limit=88,198,643,398 role=none
                12 TextBlock at=88,328,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Found 5 · Not found 1 · Failed 0 · Skipped 0 · Filled by OpenEmu 0" in=3 layout=521.52,28.8 limit=88,198,643,398 name=ScrapeStatusTallies paints=#9C9CA0FF role=text size=24 text="Found 5 · Not found 1 · Failed 0 · Skipped 0 · Filled by OpenEmu 0"
            13 StackPanel at=88,369,643,227 by=fixed-scroll in=3 limit=88,198,643,398 role=none
              14 SectionHeader at=88,369,643,34 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=60.68,33.6 limit=88,198,643,398 paints=#B4B4B8FF role=text size=28 text=Quota
              15 StackPanel at=88,411,643,185 by=fixed-scroll in=3 limit=88,198,643,398 role=none
                16 TextBlock at=88,411,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=623.06,28.8 limit=88,198,643,398 name=ScrapeStatusMember paints=#9C9CA0FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                17 MeterRow at=88,444,643,29 by=fixed-scroll drawn heard="Requests today" in=3 limit=88,198,643,398 name=ScrapeStatusQuota paints=#9C9CA0FF role=progressbar
                  18 TextBlock at=88,444,140,29 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Requests today" in=3 layout=124.85,28.8 limit=88,444,643,29 paints=#B4B4B8FF part role=text size=24 text="Requests today"
                  19 ProgressBar at=228,452,448,14 drawn in=3 name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                  20 TextBlock at=684,444,47,29 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=0% in=3 layout=29.4,28.8 limit=88,444,643,29 paints=#B4B4B8FF part role=text size=24 text=0%
                21 TextBlock at=88,477,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="12 of 20,000 requests used today · 19,988 left" in=3 layout=363.67,28.8 limit=88,198,643,398 name=ScrapeStatusRequests paints=#9C9CA0FF role=text size=24 text="12 of 20,000 requests used today · 19,988 left"
                22 HintText at=88,510,643,24 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute" in=3 layout=480.38,24 limit=88,198,643,398 name=ScrapeStatusLimits paints=#86868AFF role=text size=20 text="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute"
                23 TextBlock at=88,538,643,58 box=643,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="5 found, 1 not found, 0 failed, 0 skipped, 0 covers from OpenEmu's sources. 26 requests sent to ScreenScraper; it counts 12 of 20,000 today." in=3 layout=632.83,57.6 limit=88,198,643,398 name=ScrapeStatusSummary paints=#9C9CA0FF role=text size=24 text="5 found, 1 not found, 0 failed, 0 skipped, 0 covers from OpenEmu's sources. 26 requests sent to ScreenScraper; it counts 12 of 20,000 today."
        24 Grid at=763,198,429,398 by=panel limit=64,186,1152,482 role=none
          25 SectionHeader at=763,198,429,34 box=429,34 by=panel caps=19.5986 drawn face="Barlow Condensed" heard="Recent games" layout=142.41,33.6 limit=64,186,1152,482 paints=#B4B4B8FF role=text size=28 text="Recent games"
          26 LunaList at=763,232,429,364 by=panel limit=64,186,1152,482 name=ScrapeStatusRecent paints=#FFFFFFFF role=list
            27 ScrollViewer at=763,232,429,364 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=763,232,429,364
              28 ListBoxItem at=763,232,429,42 by=fixed-scroll drawn focus heard="Zzz Unknown Dump (SNES (Venus)): not found" in=27 limit=763,232,429,364 op paints=#333337FF,#9C9CA0FF role=listitem text="Zzz Unknown Dump (SNES (Venus)): not found"
                29 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,238,409,29 box=409,29 caps=16.7988 drawn face="Barlow Condensed" heard="Zzz Unknown Dump (SNES (Venus)): not found" in=27 layout=368.04,28.8 owner=28 paints=#9C9CA0FF role=text size=24 text="Zzz Unknown Dump (SNES (Venus)): not found"
              30 ListBoxItem at=763,274,429,42 by=fixed-scroll drawn focus heard="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=763,232,429,364 op paints=#333337FF,#9C9CA0FF role=listitem text="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                31 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,280,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=406.97,28.8 owner=30 paints=#9C9CA0FF role=text size=24 text="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              32 ListBoxItem at=763,316,429,42 by=fixed-scroll drawn focus heard="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=763,232,429,364 op paints=#333337FF,#9C9CA0FF role=listitem text="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                33 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,322,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=403.87,28.8 owner=32 paints=#9C9CA0FF role=text size=24 text="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              34 ListBoxItem at=763,358,429,42 by=fixed-scroll drawn focus heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=763,232,429,364 op paints=#333337FF,#9C9CA0FF role=listitem text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                35 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,364,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=405.26,28.8 owner=34 paints=#9C9CA0FF role=text size=24 text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              36 ListBoxItem at=763,400,429,42 by=fixed-scroll drawn focus heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=763,232,429,364 op paints=#333337FF,#9C9CA0FF role=listitem text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                37 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,406,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=403.2,28.8 owner=36 paints=#9C9CA0FF role=text size=24 text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              38 ListBoxItem at=763,442,429,42 by=fixed-scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=763,232,429,364 op paints=#333337FF,#9C9CA0FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                39 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,448,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=400.97,28.8 owner=38 paints=#9C9CA0FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
      40 ButtonBar at=595,608,90,44 by=panel limit=64,186,1152,482 paints=#9C9CA0FF role=toolbar
        41 Button at=595,608,90,44 by=clip drawn focus heard=CLOSE limit=595,608,90,44 name=ScrapeStatusHideButton op paints=#55555AFF,#9C9CA0FF role=button text=Close
          42 TextBlock at=612,614,56,32 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=595,608,90,44 owner=41 paints=#EEEEF0FF role=text size=26 text=CLOSE
focus 41
edge 28 up same
edge 28 down 30
edge 28 left 41
edge 28 right same
edge 30 up 28
edge 30 down 32
edge 30 left 41
edge 30 right same
edge 32 up 30
edge 32 down 34
edge 32 left 41
edge 32 right same
edge 34 up 32
edge 34 down 36
edge 34 left 41
edge 34 right same
edge 36 up 34
edge 36 down 38
edge 36 left 41
edge 36 right same
edge 38 up 36
edge 38 down 41
edge 38 left 41
edge 38 right same
edge 41 up 38
edge 41 down same
edge 41 left same
edge 41 right 38
press pageup focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press pagedown focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press first focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press last focus="Button 'Close'" title=Scraping top=ScrapeStatusWindow
press back top=none
