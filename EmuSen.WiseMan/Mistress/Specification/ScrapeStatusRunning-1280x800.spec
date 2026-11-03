# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state ScrapeStatusRunning-1280x800
size 1280 800
kind sheet
root Border
unit 1
slack 1.5
floor 16
frame 64,71,1152,647
title Scraping
case ScrapeStatusRunning
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=64,171,1152,527 by=panel limit=64,171,1152,527 paints=#D4D4D4FF role=none
    1 Grid at=88,183,1104,499 by=panel limit=64,171,1152,527 role=none
      2 Grid at=88,183,1104,443 by=panel limit=64,171,1152,527 role=none
        3 ScrollViewer at=88,183,643,443 by=panel limit=64,171,1152,527 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=88,183,643,443
          4 StackPanel at=88,183,643,443 by=fixed-scroll in=3 limit=88,183,643,443 role=none
            5 StackPanel at=88,183,643,76 by=fixed-scroll in=3 limit=88,183,643,443 role=none
              6 TextBlock at=88,183,643,36 box=643,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="Scraping 3 of 6 games" in=3 layout=216.69,36 limit=88,183,643,443 name=ScrapeStatusHeading paints=#D4D4D4FF role=text size=30 text="Scraping 3 of 6 games"
              7 ProgressBar at=88,225,643,4 by=fixed-scroll drawn in=3 limit=88,183,643,443 name=ScrapeStatusProgress paints=#333337FF,#4C9AE8FF role=progressbar
              8 HintText at=88,235,643,24 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Elapsed 0:18 · about 0:18 left at this run's pace" in=3 layout=300.24,24 limit=88,183,643,443 name=ScrapeStatusTiming paints=#8D8D91FF role=text size=20 text="Elapsed 0:18 · about 0:18 left at this run's pace"
            9 Grid at=88,271,643,95 by=fixed-scroll in=3 limit=88,183,643,443 role=none
              10 StackPanel at=100,271,631,95 by=fixed-scroll in=3 limit=88,183,643,443 role=none
                11 TextBlock at=100,271,631,34 box=631,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard="Cobalt Harbor (USA)" in=3 layout=176.43,33.6 limit=88,183,643,443 name=ScrapeStatusGame paints=#D4D4D4FF role=text size=28 text="Cobalt Harbor (USA)"
                12 HintText at=100,309,631,24 box=631,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="SNES (Venus)" in=3 layout=84.4,24 limit=88,183,643,443 name=ScrapeStatusConsole paints=#8D8D91FF role=text size=20 text="SNES (Venus)"
                13 TextBlock at=100,337,631,29 box=631,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Downloading its cover" in=3 layout=170.33,28.8 limit=88,183,643,443 name=ScrapeStatusStep paints=#D4D4D4FF role=text size=24 text="Downloading its cover"
            14 StackPanel at=88,378,643,71 by=fixed-scroll in=3 limit=88,183,643,443 role=none
              15 SectionHeader at=88,378,643,34 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Results in=3 layout=77.08,33.6 limit=88,183,643,443 paints=#D4D4D4FF role=text size=28 text=Results
              16 StackPanel at=88,420,643,29 by=fixed-scroll in=3 limit=88,183,643,443 role=none
                17 TextBlock at=88,420,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0" in=3 layout=506.23,28.8 limit=88,183,643,443 name=ScrapeStatusTallies paints=#D4D4D4FF role=text size=24 text="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0"
            18 StackPanel at=88,461,643,165 by=fixed-scroll in=3 limit=88,183,643,443 role=none
              19 SectionHeader at=88,461,643,34 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=56.95,33.6 limit=88,183,643,443 paints=#D4D4D4FF role=text size=28 text=Quota
              20 StackPanel at=88,503,643,123 by=fixed-scroll in=3 limit=88,183,643,443 role=none
                21 TextBlock at=88,503,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=594.58,28.8 limit=88,183,643,443 name=ScrapeStatusMember paints=#D4D4D4FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                22 MeterRow at=88,536,643,29 by=fixed-scroll drawn heard="Requests today" in=3 limit=88,183,643,443 name=ScrapeStatusQuota paints=#D4D4D4FF role=progressbar
                  23 TextBlock at=88,536,140,29 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Requests today" in=3 layout=120.1,28.8 limit=88,536,643,29 paints=#DCDCDCFF part role=text size=24 text="Requests today"
                  24 ProgressBar at=228,544,448,14 drawn in=3 name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                  25 TextBlock at=684,536,47,29 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=0% in=3 layout=26.9,28.8 limit=88,536,643,29 paints=#DCDCDCFF part role=text size=24 text=0%
                26 TextBlock at=88,569,643,29 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="11 of 20,000 requests used today · 19,989 left" in=3 layout=342.65,28.8 limit=88,183,643,443 name=ScrapeStatusRequests paints=#D4D4D4FF role=text size=24 text="11 of 20,000 requests used today · 19,989 left"
                27 HintText at=88,602,643,24 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute" in=3 layout=463.94,24 limit=88,183,643,443 name=ScrapeStatusLimits paints=#8D8D91FF role=text size=20 text="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute"
        28 Grid at=763,183,429,443 by=panel limit=64,171,1152,527 role=none
          29 SectionHeader at=763,183,429,34 box=429,34 by=panel caps=19.5986 drawn face="Barlow Condensed" heard="Recent games" layout=134.74,33.6 limit=64,171,1152,527 paints=#D4D4D4FF role=text size=28 text="Recent games"
          30 LunaList at=763,217,429,409 by=panel limit=64,171,1152,527 name=ScrapeStatusRecent paints=#FFFFFFFF role=list
            31 ScrollViewer at=763,217,429,409 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=763,217,429,409
              32 ListBoxItem at=763,217,429,42 by=fixed-scroll drawn focus heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=763,217,429,409 op paints=#333337FF,#D4D4D4FF role=listitem text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                33 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,223,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=405.34,28.8 owner=32 paints=#D4D4D4FF role=text size=24 text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              34 ListBoxItem at=763,259,429,42 by=fixed-scroll drawn focus heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=763,217,429,409 op paints=#333337FF,#D4D4D4FF role=listitem text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                35 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,265,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=404.4,28.8 owner=34 paints=#D4D4D4FF role=text size=24 text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              36 ListBoxItem at=763,301,429,42 by=fixed-scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=763,217,429,409 op paints=#333337FF,#D4D4D4FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                37 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=773,307,409,29 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=405.5,28.8 owner=36 paints=#D4D4D4FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
      38 ButtonBar at=495,638,290,44 by=panel limit=64,171,1152,527 paints=#D4D4D4FF role=toolbar
        39 Button at=495,638,87,44 by=clip drawn focus heard=PAUSE limit=495,638,290,44 name=ScrapeStatusPauseButton op paints=#707070FF,#D4D4D4FF role=button text=Pause
          40 TextBlock at=512,644,53,32 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PAUSE layout=52.65,31.2 limit=495,638,87,44 owner=39 paints=#D4D4D4FF role=text size=26 text=PAUSE
        41 Button at=590,638,114,44 by=clip drawn focus heard=CANCEL... limit=495,638,290,44 name=ScrapeStatusCancelButton op paints=#707070FF,#D4D4D4FF role=button text=Cancel...
          42 TextBlock at=607,644,80,32 box=80,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL... layout=79.56,31.2 limit=590,638,114,44 owner=41 paints=#D4D4D4FF role=text size=26 text=CANCEL...
        43 Button at=712,638,73,44 by=clip drawn focus heard=HIDE limit=495,638,290,44 name=ScrapeStatusHideButton op paints=#707070FF,#D4D4D4FF role=button text=Hide
          44 TextBlock at=729,644,39,32 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=712,638,73,44 owner=43 paints=#EEEEF0FF role=text size=26 text=HIDE
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
