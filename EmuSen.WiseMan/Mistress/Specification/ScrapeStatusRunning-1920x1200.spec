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
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=96,218,1728,792 by=panel limit=96,218,1728,791 paints=#D4D4D4FF role=none seen=96,218,1728,791
    1 Grid at=132,236,1656,750 by=panel limit=96,218,1728,791 role=none
      2 Grid at=132,236,1656,666 by=panel limit=96,218,1728,791 role=none
        3 ScrollViewer at=132,236,964.5,666 by=panel limit=96,218,1728,791 overflow=0 paints=#D4D4D4FF role=scrollviewer viewport=132,236,964.5,666
          4 StackPanel at=132,236,964.5,666 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
            5 StackPanel at=132,236,964.5,114 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              6 TextBlock at=132,236,964.5,54 box=643,36 by=fixed-scroll caps=20.9985 drawn face="Barlow Condensed" heard="Scraping 3 of 6 games" in=3 layout=216.69,36 limit=132,236,964.5,666 name=ScrapeStatusHeading paints=#D4D4D4FF role=text size=30 text="Scraping 3 of 6 games"
              7 ProgressBar at=132,299,964.5,6 by=fixed-scroll drawn in=3 limit=132,236,964.5,666 name=ScrapeStatusProgress paints=#333337FF,#4C9AE8FF role=progressbar
              8 HintText at=132,314,964.5,36 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="Elapsed 0:18 · about 0:18 left at this run's pace" in=3 layout=300.24,24 limit=132,236,964.5,666 name=ScrapeStatusTiming paints=#8D8D91FF role=text size=20 text="Elapsed 0:18 · about 0:18 left at this run's pace"
            9 Grid at=132,368,964.5,142.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              10 StackPanel at=150,368,946.5,142.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
                11 TextBlock at=150,368,946.5,51 box=631,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard="Cobalt Harbor (USA)" in=3 layout=176.43,33.6 limit=132,236,964.5,666 name=ScrapeStatusGame paints=#D4D4D4FF role=text size=28 text="Cobalt Harbor (USA)"
                12 HintText at=150,425,946.5,36 box=631,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="SNES (Venus)" in=3 layout=84.4,24 limit=132,236,964.5,666 name=ScrapeStatusConsole paints=#8D8D91FF role=text size=20 text="SNES (Venus)"
                13 TextBlock at=150,467,946.5,43.5 box=631,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Downloading its cover" in=3 layout=170.33,28.8 limit=132,236,964.5,666 name=ScrapeStatusStep paints=#D4D4D4FF role=text size=24 text="Downloading its cover"
            14 StackPanel at=132,528.5,964.5,106.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              15 SectionHeader at=132,528.5,964.5,51 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Results in=3 layout=77.08,33.6 limit=132,236,964.5,666 paints=#D4D4D4FF role=text size=28 text=Results
              16 StackPanel at=132,591.5,964.5,43.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
                17 TextBlock at=132,591.5,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0" in=3 layout=506.23,28.8 limit=132,236,964.5,666 name=ScrapeStatusTallies paints=#D4D4D4FF role=text size=24 text="Found 3 · Not found 0 · Failed 0 · Skipped 0 · Filled by OpenEmu 0"
            18 StackPanel at=132,653,964.5,247.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
              19 SectionHeader at=132,653,964.5,51 box=643,34 by=fixed-scroll caps=19.5986 drawn face="Barlow Condensed" heard=Quota in=3 layout=56.95,33.6 limit=132,236,964.5,666 paints=#D4D4D4FF role=text size=28 text=Quota
              20 StackPanel at=132,716,964.5,184.5 by=fixed-scroll in=3 limit=132,236,964.5,666 role=none
                21 TextBlock at=132,716,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="No member account: EmuSen's developer credentials alone, with their limits." in=3 layout=594.58,28.8 limit=132,236,964.5,666 name=ScrapeStatusMember paints=#D4D4D4FF role=text size=24 text="No member account: EmuSen's developer credentials alone, with their limits."
                22 MeterRow at=132,765.5,964.5,43.5 by=fixed-scroll drawn heard="Requests today" in=3 limit=132,236,964.5,666 name=ScrapeStatusQuota paints=#D4D4D4FF role=progressbar
                  23 TextBlock at=132,765.5,210,43.5 box=140,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Requests today" in=3 layout=120.1,28.8 limit=132,765.5,964.5,43.5 paints=#DCDCDCFF part role=text size=24 text="Requests today"
                  24 ProgressBar at=342,777.5,672,21 drawn in=3 name=PART_Bar paints=#32CD32FF,#333337FF part role=progressbar
                  25 TextBlock at=1026,765.5,70.5,43.5 box=47,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=0% in=3 layout=26.9,28.8 limit=132,765.5,964.5,43.5 paints=#DCDCDCFF part role=text size=24 text=0%
                26 TextBlock at=132,815,964.5,43.5 box=643,29 by=fixed-scroll caps=16.7988 drawn face="Barlow Condensed" heard="11 of 20,000 requests used today · 19,989 left" in=3 layout=342.65,28.8 limit=132,236,964.5,666 name=ScrapeStatusRequests paints=#D4D4D4FF role=text size=24 text="11 of 20,000 requests used today · 19,989 left"
                27 HintText at=132,864.5,964.5,36 box=643,24 by=fixed-scroll caps=13.999 drawn face="Barlow Condensed" heard="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute" in=3 layout=463.94,24 limit=132,236,964.5,666 name=ScrapeStatusLimits paints=#8D8D91FF role=text size=20 text="1 thread · download limit 128 KB/s · 20,000 requests a day · 60 a minute"
        28 Grid at=1144.5,236,643.5,666 by=panel limit=96,218,1728,791 role=none
          29 SectionHeader at=1144.5,236,643.5,51 box=429,34 by=panel caps=19.5986 drawn face="Barlow Condensed" heard="Recent games" layout=134.74,33.6 limit=96,218,1728,791 paints=#D4D4D4FF role=text size=28 text="Recent games"
          30 LunaList at=1144.5,287,643.5,615 by=panel limit=96,218,1728,791 name=ScrapeStatusRecent paints=#FFFFFFFF role=list
            31 ScrollViewer at=1144.5,287,643.5,615 name=PART_ScrollViewer overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=1144.5,287,643.5,615
              32 ListBoxItem at=1144.5,287,643.5,63 by=fixed-scroll drawn focus heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=1144.5,287,643.5,615 op paints=#333337FF,#D4D4D4FF role=listitem text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                33 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,296,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=405.34,28.8 owner=32 paints=#D4D4D4FF role=text size=24 text="Brass Lantern (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              34 ListBoxItem at=1144.5,350,643.5,63 by=fixed-scroll drawn focus heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=1144.5,287,643.5,615 op paints=#333337FF,#D4D4D4FF role=listitem text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                35 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,359,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=404.4,28.8 owner=34 paints=#D4D4D4FF role=text size=24 text="Aurora Drift (USA) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
              36 ListBoxItem at=1144.5,413,643.5,63 by=fixed-scroll drawn focus heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 limit=1144.5,287,643.5,615 op paints=#333337FF,#D4D4D4FF role=listitem text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
                37 TextBlock allow="a recent game's line cut in the list is whole in the panel's footer while its row is chosen" at=1159.5,422,613.5,43.5 box=409,29 caps=16.7988 drawn ellipsis face="Barlow Condensed" heard="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image" in=31 layout=405.5,28.8 owner=36 paints=#D4D4D4FF role=text size=24 text="A Game With A Very Long Name Indeed, Special Edition (USA) (Rev 1) (SNES (Venus)): found · cover, screenshot, marquee, mix image"
      38 ButtonBar at=742.5,920,435,66 by=panel limit=96,218,1728,791 paints=#D4D4D4FF role=toolbar
        39 Button at=742.5,920,130.5,66 by=clip drawn focus heard=PAUSE limit=742.5,920,435,66 name=ScrapeStatusPauseButton op paints=#707070FF,#D4D4D4FF role=button text=Pause
          40 TextBlock at=768,929,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=PAUSE layout=52.65,31.2 limit=742.5,920,130.5,66 owner=39 paints=#D4D4D4FF role=text size=26 text=PAUSE
        41 Button at=885,920,171,66 by=clip drawn focus heard=CANCEL... limit=742.5,920,435,66 name=ScrapeStatusCancelButton op paints=#707070FF,#D4D4D4FF role=button text=Cancel...
          42 TextBlock at=910.5,929,120,48 box=80,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CANCEL... layout=79.56,31.2 limit=885,920,171,66 owner=41 paints=#D4D4D4FF role=text size=26 text=CANCEL...
        43 Button at=1068,920,109.5,66 by=clip drawn focus heard=HIDE limit=742.5,920,435,66 name=ScrapeStatusHideButton op paints=#707070FF,#D4D4D4FF role=button text=Hide
          44 TextBlock at=1093.5,929,58.5,48 box=39,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=HIDE layout=38.01,31.2 limit=1068,920,109.5,66 owner=43 paints=#EEEEF0FF role=text size=26 text=HIDE
focus 43
edge 32 up same
edge 32 down 34
edge 32 left 43
edge 32 right same
edge 34 up 32
edge 34 down 36
edge 34 left 43
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
edge 43 right 36
press pageup focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press pagedown focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press first focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press last focus="Button 'Hide'" title=Scraping top=ScrapeStatusWindow
press back top=none
