# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsDesktopConsoles-1280x800
size 1280 800
kind window
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 0
frame 0,0,960,640
case SettingsDesktopConsoles
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
window 960 640
tree
  0 Panel at=0,0,960,640 by=panel limit=0,0,960,640 role=none
    1 DockPanel at=0,0,960,640 by=panel limit=0,0,960,640 paints=#1B1B1EFF role=none
      2 Border at=0,0,201,640 by=panel limit=0,0,960,640 paints=#333337FF role=none
        3 SourceList at=0,12,200,628 by=panel focus limit=0,0,960,640 name=SettingsSourceList paints=#1B1B1EFF,#FFFFFFFF role=list
          4 ScrollViewer at=0,12,200,628 name=PART_Scroll overflow=0 paints=#FFFFFFFF part role=scrollviewer viewport=0,12,200,628
            5 SourceListGroupView at=0,12,200,144 by=fixed-scroll in=4 limit=0,12,200,628 role=group
              6 SourceListRow at=6,12,188,24 by=fixed-scroll heard=Controllers in=4 limit=0,12,200,628 role=listitem
                7 Grid at=26,12,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  8 TextBlock at=26,16,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Controllers in=4 layout=59.77,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Controllers
              9 SourceListRow at=6,36,188,24 by=fixed-scroll heard=Display in=4 limit=0,12,200,628 role=listitem
                10 Grid at=26,36,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  11 TextBlock at=26,40,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Display in=4 layout=39.94,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Display
              12 SourceListRow at=6,60,188,24 by=fixed-scroll heard=Sound in=4 limit=0,12,200,628 role=listitem
                13 Grid at=26,60,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  14 TextBlock at=26,64,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Sound in=4 layout=34.92,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Sound
              15 SourceListRow at=6,84,188,24 by=fixed-scroll heard=Consoles in=4 limit=0,12,200,628 paints=#4C9AE8FF role=listitem
                16 Grid at=26,84,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  17 TextBlock at=26,88,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Consoles in=4 layout=49.32,15.6 limit=0,12,200,628 paints=#0E0E10FF role=text size=13 text=Consoles
              18 SourceListRow at=6,108,188,24 by=fixed-scroll heard=Library in=4 limit=0,12,200,628 role=listitem
                19 Grid at=26,108,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  20 TextBlock at=26,112,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=Library in=4 layout=39.23,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=Library
              21 SourceListRow at=6,132,188,24 by=fixed-scroll heard=EmuSen in=4 limit=0,12,200,628 role=listitem
                22 Grid at=26,132,158,24 by=fixed-scroll in=4 limit=0,12,200,628 role=none
                  23 TextBlock at=26,136,158,16 box=158,16 by=fixed-scroll caps=9.0994 drawn face=Barlow heard=EmuSen in=4 layout=45.84,15.6 limit=0,12,200,628 paints=#D4D4D4FF role=text size=13 text=EmuSen
      24 DockPanel at=225,8,711,608 by=panel limit=0,0,960,640 role=none
        25 DockPanel at=225,8,711,48 by=panel limit=0,0,960,640 name=SettingsHeader role=none
          26 TextBox at=716,16,220,32 by=panel drawn focus heard="Search settings" limit=0,0,960,640 name=SettingsSearch op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=edit
            27 ScrollViewer at=727,23,202,19 name=PART_ScrollViewer paints=#D4D4D4FF part role=scrollviewer
              28 TextBlock at=727,23,202,19 box=202,19 by=fixed-scroll caps=9.0994 drawn face=Barlow heard="Search settings" in=27 layout=85.19,15.6 limit=727,23,202,19 name=PART_Placeholder paints=#8D8D91FF part role=text size=13 text="Search settings"
          29 TextBlock at=225,22,491,20 box=491,20 by=panel caps=11.1992 drawn face="Barlow Condensed" heard=CONSOLES layout=53.18,19.2 limit=0,0,960,640 name=SettingsTitle paints=#EEEEF0FF role=text size=16 text=CONSOLES
        30 ScrollViewer at=225,56,711,560 by=panel fades=b limit=0,0,960,640 overflow=112 paints=#707070FF,#FFFFFFFF role=scrollviewer viewport=225,56,711,560
          31 StackPanel at=225,56,711,672 by=scroll heard=Consoles in=30 limit=225,56,711,560 name=SettingsRows paints=#0E0E10FF,#333337FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=225,56,711,560
            32 Border at=225,56,711,48 by=scroll focus heard=NES in=30 limit=225,56,711,560 name=SettingsRow.console:NES paints=#26262BFF,#333337FF role=none
              33 DockPanel at=233,60,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                34 Button at=863,65,57,29 by=scroll drawn focus heard=NES in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  35 AccessText at=872,71,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,65,57,29 owner=34 paints=#D4D4D4FF role=none size=13 text="Open…"
                36 StackPanel at=233,64,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  37 TextBlock at=233,64,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard=NES in=30 layout=23.24,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text=NES
                  38 TextBlock at=233,80,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="2 games · MoonRT (Rust)" in=30 layout=111.86,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="2 games · MoonRT (Rust)"
            39 Border at=225,104,711,48 by=scroll focus heard=SG-1000 in=30 limit=225,56,711,560 name=SettingsRow.console:SG-1000 paints=#333337FF role=none
              40 DockPanel at=233,108,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                41 Button at=863,113,57,29 by=scroll drawn focus heard=SG-1000 in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  42 AccessText at=872,119,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,113,57,29 owner=41 paints=#D4D4D4FF role=none size=13 text="Open…"
                43 StackPanel at=233,112,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  44 TextBlock at=233,112,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard=SG-1000 in=30 layout=45.19,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text=SG-1000
                  45 TextBlock at=233,128,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · Endou" in=30 layout=75.9,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · Endou"
            46 Border at=225,152,711,48 by=scroll focus heard="Master System" in=30 limit=225,56,711,560 name="SettingsRow.console:Master System" paints=#333337FF role=none
              47 DockPanel at=233,156,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                48 Button at=863,161,57,29 by=scroll drawn focus heard="Master System" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  49 AccessText at=872,167,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,161,57,29 owner=48 paints=#D4D4D4FF role=none size=13 text="Open…"
                50 StackPanel at=233,160,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  51 TextBlock at=233,160,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="Master System" in=30 layout=80.37,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="Master System"
                  52 TextBlock at=233,176,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · Endou · open replacement" in=30 layout=166.89,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · Endou · open replacement"
            53 Border at=225,200,711,48 by=scroll focus heard=Genesis in=30 limit=225,56,711,560 name=SettingsRow.console:Genesis paints=#333337FF role=none
              54 DockPanel at=233,204,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                55 Button at=863,209,57,29 by=scroll drawn focus heard=Genesis in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  56 AccessText at=872,215,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,209,57,29 owner=55 paints=#D4D4D4FF role=none size=13 text="Open…"
                57 StackPanel at=233,208,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  58 TextBlock at=233,208,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard=Genesis in=30 layout=42.78,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text=Genesis
                  59 TextBlock at=233,224,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · Nephrite" in=30 layout=86.66,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · Nephrite"
            60 Border at=225,248,711,48 by=scroll focus heard="Game Boy" in=30 limit=225,56,711,560 name=SettingsRow.console:GB paints=#333337FF role=none
              61 DockPanel at=233,252,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                62 Button at=863,257,57,29 by=scroll drawn focus heard="Game Boy" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  63 AccessText at=872,263,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,257,57,29 owner=62 paints=#D4D4D4FF role=none size=13 text="Open…"
                64 StackPanel at=233,256,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  65 TextBlock at=233,256,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="Game Boy" in=30 layout=54.68,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="Game Boy"
                  66 TextBlock at=233,272,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="1 game · MercuryRT (Rust)" in=30 layout=117.39,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="1 game · MercuryRT (Rust)"
            67 Border at=225,296,711,48 by=scroll focus heard="Super Nintendo" in=30 limit=225,56,711,560 name=SettingsRow.console:SNES paints=#333337FF role=none
              68 DockPanel at=233,300,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                69 Button at=863,305,57,29 by=scroll drawn focus heard="Super Nintendo" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  70 AccessText at=872,311,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,305,57,29 owner=69 paints=#D4D4D4FF role=none size=13 text="Open…"
                71 StackPanel at=233,304,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  72 TextBlock at=233,304,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="Super Nintendo" in=30 layout=85.53,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="Super Nintendo"
                  73 TextBlock at=233,320,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="5 games · VenusRT (Rust)" in=30 layout=114.61,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="5 games · VenusRT (Rust)"
            74 Border at=225,344,711,48 by=scroll focus heard="Sega CD" in=30 limit=225,56,711,560 name="SettingsRow.console:Sega CD" paints=#333337FF role=none
              75 DockPanel at=233,348,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                76 Button at=863,353,57,29 by=scroll drawn focus heard="Sega CD" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  77 AccessText at=872,359,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,353,57,29 owner=76 paints=#D4D4D4FF role=none size=13 text="Open…"
                78 StackPanel at=233,352,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  79 TextBlock at=233,352,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="Sega CD" in=30 layout=45.11,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="Sega CD"
                  80 TextBlock at=233,368,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · Nephrite" in=30 layout=86.66,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · Nephrite"
            81 Border at=225,392,711,48 by=scroll focus heard=PlayStation in=30 limit=225,56,711,560 name=SettingsRow.console:PlayStation paints=#333337FF role=none
              82 DockPanel at=233,396,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                83 Button at=863,401,57,29 by=scroll drawn focus heard=PlayStation in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  84 AccessText at=872,407,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,401,57,29 owner=83 paints=#D4D4D4FF role=none size=13 text="Open…"
                85 StackPanel at=233,400,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  86 TextBlock at=233,400,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard=PlayStation in=30 layout=62.45,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text=PlayStation
                  87 TextBlock at=233,416,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · Diamond · open replacement" in=30 layout=178.77,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · Diamond · open replacement"
            88 Border at=225,440,711,48 by=scroll focus heard="Nintendo 64" in=30 limit=225,56,711,560 name=SettingsRow.console:N64 paints=#333337FF role=none
              89 DockPanel at=233,444,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                90 Button at=863,449,57,29 by=scroll drawn focus heard="Nintendo 64" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  91 AccessText at=872,455,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,449,57,29 owner=90 paints=#D4D4D4FF role=none size=13 text="Open…"
                92 StackPanel at=233,448,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  93 TextBlock at=233,448,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="Nintendo 64" in=30 layout=66.05,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="Nintendo 64"
                  94 TextBlock at=233,464,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · MarsRT (Rust)" in=30 layout=109.47,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · MarsRT (Rust)"
            95 Border at=225,488,711,48 by=scroll focus heard="Game Boy Color" in=30 limit=225,56,711,560 name=SettingsRow.console:GBC paints=#333337FF role=none
              96 DockPanel at=233,492,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                97 Button at=863,497,57,29 by=scroll drawn focus heard="Game Boy Color" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  98 AccessText at=872,503,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,497,57,29 owner=97 paints=#D4D4D4FF role=none size=13 text="Open…"
                99 StackPanel at=233,496,614,30 by=scroll in=30 limit=225,56,711,560 role=none
                  100 TextBlock at=233,496,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="Game Boy Color" in=30 layout=85.85,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="Game Boy Color"
                  101 TextBlock at=233,512,614,14 box=614,14 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · MercuryRT (Rust)" in=30 layout=124.99,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · MercuryRT (Rust)"
            102 Border at=225,536,711,48 by=scroll focus heard="PlayStation 2" in=30 limit=225,56,711,560 name="SettingsRow.console:PlayStation 2" paints=#333337FF role=none
              103 DockPanel at=233,540,695,39 by=scroll in=30 limit=225,56,711,560 role=none
                104 Button at=863,545,57,29 by=scroll drawn focus heard="PlayStation 2" in=30 limit=225,56,711,560 op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button text="Open…"
                  105 AccessText at=872,551,39,16 box=39,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=38.73,15.6 limit=863,545,57,29 owner=104 paints=#D4D4D4FF role=none size=13 text="Open…"
                106 StackPanel at=233,544,614,31 by=scroll in=30 limit=225,56,711,560 role=none
                  107 TextBlock at=233,544,614,16 box=614,16 by=scroll caps=9.0994 drawn face=Barlow heard="PlayStation 2" in=30 layout=71.64,15.6 limit=225,56,711,560 paints=#D4D4D4FF role=text size=13 text="PlayStation 2"
                  108 TextBlock at=233,560,614,15 box=614,15 by=scroll caps=7.6995 drawn face=Barlow heard="0 games · Sapphire · ⚠ BIOS not found" in=30 layout=176.2,14.14 limit=225,56,711,560 paints=#8D8D91FF role=text size=11 text="0 games · Sapphire · ⚠ BIOS not found"
            109 Border at=225,584,711,48 by=scroll focus heard="32X, Not supported yet" in=30 limit=225,56,711,560 name=SettingsRow.console:32X paints=#333337FF role=none seen=225,584,711,32
              110 DockPanel at=233,588,695,39 by=scroll in=30 limit=225,56,711,560 role=none seen=233,588,695,28
                111 Button at=796,593,124,29 by=scroll drawn heard=32X in=30 limit=225,56,711,560 off paints=#0E0E10FF,#707070FF,#D4D4D4FF role=button seen=796,593,124,23 text="Not supported yet ›"
                  112 AccessText at=805,599,106,16 box=106,16 by=clip caps=9.0994 drawn face=Barlow in=30 layout=105.3,15.6 limit=796,593,124,29 off owner=111 paints=#8D8D91FF role=none size=13 text="Not supported yet ›"
                113 StackPanel at=233,592,547,30 by=scroll in=30 limit=225,56,711,560 role=none seen=233,592,547,24
                  114 TextBlock at=233,592,547,16 box=547,16 by=scroll caps=9.0994 drawn face=Barlow heard=32X in=30 layout=20.37,15.6 limit=225,56,711,560 paints=#8D8D91FF role=text size=13 text=32X
                  115 TextBlock at=233,608,547,14 box=547,14 by=scroll caps=7.6995 drawn face=Barlow heard="EmuSen has no engine for the 32X yet." in=30 layout=176.34,13.2 limit=225,56,711,560 paints=#8D8D91FF role=text seen=233,608,547,8 size=11 text="EmuSen has no engine for the 32X yet."
focus none
