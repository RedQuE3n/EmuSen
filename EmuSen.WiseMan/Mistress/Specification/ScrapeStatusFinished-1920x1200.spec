# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusFinished-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 96,129,1728,903
title Scraping
case ScrapeStatusFinished
palette #000000FF #007ACCFF #050507FF #0E0E10FF #141416FF #161618F0 #161618FF #1B1B1EFF #1C1C1CE6 #1E1E1EFF #252526FF #2E8B57FF #32CD32FF #333337FF #4C9AE8FF #55555AFF #6E6E6EFF #808080FF #86868AFF #9C9CA0FF #9CDCFEFF #B4B4B8FF #CD5C5CFF #D08770FF #D2D2D6FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,279,1728,723 by=panel limit=96,279,1728,723 paints=#9C9CA0FF role=none
    1 Grid at=132,297,1656,681 by=panel limit=96,279,1728,723 role=none
      2 Grid at=132,297,1656,597 by=panel limit=96,279,1728,723 role=none
        3 ScrollViewer at=132,297,964.5,597 by=panel limit=96,279,1728,723 overflow=0 paints=#9C9CA0FF role=scrollviewer viewport=132,297,964.5,597
          4 StackPanel at=132,297,964.5,597 by=fixed-scroll in=3 limit=132,297,964.5,597 role=none
            5 StackPanel at=132,297,964.5,114 by=fixed-scroll in=3 limit=132,297,964.5,597 role=none
              6 TextBlock at=132,297,964.5,54 box=643,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="Finished: 6 of 6 games" in=3 layout=229.44,36 limit=132,297,964.5,597 name=ScrapeStatusHeading paints=#9C9CA0FF role=text size=30 text="Finished: 6 of 6 games"
              7 ProgressBar at=132,360,964.5,6 by=fixed-scroll drawn in=3 limit=132,297,964.5,597 name=ScrapeStatusProgress paints=#333337FF,#4C9AE8FF role=progressbar
              8 HintText at=132,375,964.5,36 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Took 0:28" in=3 layout=65.7,24 limit=132,297,964.5,597 name=ScrapeStatusTiming paints=#86868AFF role=text size=20 text="Took 0:28"
            9 StackPanel at=132,429,964.5,106.5 by=fixed-scroll in=3 limit=132,297,964.5,597 role=none
              10 SectionHeader at=132,429,964.5,51 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Results in=3 layout=82.21,33.6 limit=132,297,964.5,597 paints=#B4B4B8FF role=text size=28 text=Results
              11 StackPanel at=132,492,964.5,43.5 by=fixed-scroll in=3 limit=132,297,964.5,597 role=none
                12 TextBlock at=132,492,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Found 5 · Not found 1 · Failed 0 · Skipped 0 · Filled by OpenEmu 0" in=3 layout=521.52,28.8 limit=132,297,964.5,597 name=ScrapeStatusTallies paints=#9C9CA0FF role=text size=24 text="Found 5 · Not found 1 · Failed 0 · Skipped 0 · Filled by OpenEmu 0"
            13 StackPanel at=132,553.5,964.5,340.5 by=fixed-scroll in=3 limit=132,297,964.5,597 role=none
              14 SectionHeader at=132,553.5,964.5,51 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=60.68,33.6 limit=132,297,964.5,597 paints=#B4B4B8FF role=text size=28 text=Quota
              15 StackPanel at=132,616.5,964.5,277.5 by=fixed-scroll in=3 limit=132,297,964.5,597 role=none
                16 TextBlock at=132,616.5,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=623.06,28.8 limit=132,297,964.5,597 name=ScrapeStatusMember paints=#9C9CA0FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                17 MeterRow at=132,666,964.5,43.5 by=fixed-scroll drawn heard="Requests today" in=3 limit=132,297,964.5,597 name=ScrapeStatusQuota paints=#9C9CA0FF role=progressbar
                  18 TextBlock at=132,666,210,43.5 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Requests today" in=3 layout=124.85,28.8 limit=132,666,964.5,43.5 paints=#B4B4B8FF part role=text size=24 text="Requests today"
                  19 ProgressBar at=342,678,672,21 drawn in=3 name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                  20 TextBlock at=1026,666,70.5,43.5 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=0% in=3 layout=29.4,28.8 limit=132,666,964.5,43.5 paints=#B4B4B8FF part role=text size=24 text=0%
                21 TextBlock at=132,715.5,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="12 of 20,000 requests used today · 19,988 left" in=3 layout=363.67,28.8 limit=132,297,964.5,597 name=ScrapeStatusRequests paints=#9C9CA0FF role=text size=24 text="12 of 20,000 requests used today · 19,988 left"
                22 HintText at=132,765,964.5,36 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute" in=3 layout=480.38,24 limit=132,297,964.5,597 name=ScrapeStatusLimits paints=#86868AFF role=text size=20 text="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute"
                23 TextBlock at=132,807,964.5,87 box=643,58 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="5 found, 1 not found, 0 failed, 0 skipped, 0 covers from OpenEmu's sources. 26 requests sent to ScreenScraper; it counts 12 of 20,000 today." in=3 layout=632.83,57.6 limit=132,297,964.5,597 name=ScrapeStatusSummary paints=#9C9CA0FF role=text size=24 text="5 found, 1 not found, 0 failed, 0 skipped, 0 covers from OpenEmu's sources. 26 requests sent to ScreenScraper; it counts 12 of 20,000 today."
        24 Grid at=1144.5,297,643.5,597 by=panel limit=96,279,1728,723 role=none
          25 SectionHeader at=1144.5,297,643.5,51 box=429,34 by=panel caps=19.5986 drawn face="Barlow Condensed" heard="Recent games" layout=142.41,33.6 limit=96,279,1728,723 paints=#B4B4B8FF role=text size=28 text="Recent games"
          26 LunaList at=1144.5,348,643.5,546 by=panel limit=96,279,1728,723 name=ScrapeStatusRecent paints=#FFFFFFFF role=list
            27 ScrollViewer at=1144.5,348,643.5,546 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=1144.5,348,643.5,546
              28 ListBoxItem at=1144.5,348,643.5,63 by=fixed-scroll drawn focus heard="Zzz Unknown Dump (SNES (Venus)): not found" in=27 limit=1144.5,348,643.5,546 op paints=#333337FF,#9C9CA0FF role=listitem text="Zzz Unknown Dump (SNES (Venus)): not found"
                29 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,357,613.5,43.5 box=409,29 caps=16.7988 drawn face="Barlow Condensed" heard="Zzz Unknown Dump (SNES (Venus)): not found" in=27 layout=368.04,28.8 owner=28 paints=#9C9CA0FF role=text size=24 text="Zzz Unknown Dump (SNES (Venus)): not found"
              30 ListBoxItem at=1144.5,411,643.5,63 by=fixed-scroll drawn focus heard="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=1144.5,348,643.5,546 op paints=#333337FF,#9C9CA0FF role=listitem text="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                31 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,420,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=406.97,28.8 owner=30 paints=#9C9CA0FF role=text size=24 text="Dune Relay (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              32 ListBoxItem at=1144.5,474,643.5,63 by=fixed-scroll drawn focus heard="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=1144.5,348,643.5,546 op paints=#333337FF,#9C9CA0FF role=listitem text="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                33 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,483,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=403.87,28.8 owner=32 paints=#9C9CA0FF role=text size=24 text="Cobalt Harbor (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              34 ListBoxItem at=1144.5,537,643.5,63 by=fixed-scroll drawn focus heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=1144.5,348,643.5,546 op paints=#333337FF,#9C9CA0FF role=listitem text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                35 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,546,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=405.26,28.8 owner=34 paints=#9C9CA0FF role=text size=24 text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              36 ListBoxItem at=1144.5,600,643.5,63 by=fixed-scroll drawn focus heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=1144.5,348,643.5,546 op paints=#333337FF,#9C9CA0FF role=listitem text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                37 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,609,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=403.2,28.8 owner=36 paints=#9C9CA0FF role=text size=24 text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              38 ListBoxItem at=1144.5,663,643.5,63 by=fixed-scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 limit=1144.5,348,643.5,546 op paints=#333337FF,#9C9CA0FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                39 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,672,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=27 layout=400.97,28.8 owner=38 paints=#9C9CA0FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
      40 ButtonBar at=892.5,912,135,66 by=panel limit=96,279,1728,723 paints=#9C9CA0FF role=toolbar
        41 Button at=892.5,912,135,66 by=clip drawn focus heard=CLOSE limit=892.5,912,135,66 name=ScrapeStatusHideButton op paints=#55555AFF,#9C9CA0FF role=button text=Close
          42 TextBlock at=918,921,84,48 box=56,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=55.51,31.2 limit=892.5,912,135,66 owner=41 paints=#EEEEF0FF role=text size=26 text=CLOSE
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
