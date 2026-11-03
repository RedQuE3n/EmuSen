# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsLibrary-1280x800
size 1280 800
kind menu
root SettingsLayout#SettingsScreen
unit 1
slack 1.5
floor 16
frame 0,0,1280,800
case SettingsLibrary
palette #0000008C #000000AA #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
    1 LayoutTransformControl at=0,0,1280,800 by=panel limit=0,0,1280,800 role=none
      2 Panel at=0,0,1280,800 by=panel limit=0,0,1280,800 paints=#1B1B1EFF role=none
        3 WholeRow at=64,40,1152,44 by=panel heard=Sections limit=0,0,1280,800 name=SettingsSections role=none
          4 ToggleButton at=64,40,159,44 by=panel drawn focus heard=Controllers limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=CONTROLLERS
            5 AccessText at=80,48,127,25 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=64,40,159,44 owner=4 paints=#D4D4D4FF role=none size=20 text=CONTROLLERS
          6 ToggleButton at=231,40,104,44 by=panel drawn focus heard=Display limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=DISPLAY
            7 AccessText at=247,48,72,25 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=231,40,104,44 owner=6 paints=#D4D4D4FF role=none size=20 text=DISPLAY
          8 ToggleButton at=343,40,93,44 by=panel drawn focus heard=Sound limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=SOUND
            9 AccessText at=359,48,61,25 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=343,40,93,44 owner=8 paints=#D4D4D4FF role=none size=20 text=SOUND
          10 ToggleButton at=444,40,125,44 by=panel drawn focus heard=Consoles limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=CONSOLES
            11 AccessText at=460,48,93,25 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=444,40,125,44 owner=10 paints=#D4D4D4FF role=none size=20 text=CONSOLES
          12 ToggleButton at=577,40,107,44 by=panel drawn focus heard=Library limit=0,0,1280,800 op paints=#4C9AE8FF,#EEEEF0FF role=button text=LIBRARY
            13 AccessText at=593,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=577,40,107,44 owner=12 paints=#EEEEF0FF role=none size=20 text=LIBRARY
          14 ToggleButton at=692,40,107,44 by=panel drawn focus heard=EmuSen limit=0,0,1280,800 op paints=#D4D4D4FF role=button text=EMUSEN
            15 AccessText at=708,48,75,25 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=692,40,107,44 owner=14 paints=#D4D4D4FF role=none size=20 text=EMUSEN
        16 StackPanel at=64,108,704,486 by=panel heard=Library limit=0,0,1280,800 name=SettingsRows role=none
          17 MenuRow at=64,108,704,54 by=panel heard="ROM folder, <run>/Roms" limit=0,0,1280,800 name=SettingsRow.rom role=listitem
          18 MenuRow at=64,162,704,54 by=panel heard="Artwork folder, beside the games" limit=0,0,1280,800 name=SettingsRow.art role=listitem
          19 MenuRow at=64,216,704,54 by=panel heard="Themes, EmuSen" limit=0,0,1280,800 name=SettingsRow.themes role=listitem
          20 MenuRow at=64,270,704,54 by=panel heard="Scraping, ScreenScraper" limit=0,0,1280,800 name=SettingsRow.scraping role=listitem
          21 MenuRow at=64,324,704,54 by=panel heard="Collections, 1" limit=0,0,1280,800 name=SettingsRow.collections role=listitem
          22 MenuRow at=64,378,704,54 by=panel heard="Hidden games, Not listed" limit=0,0,1280,800 name=SettingsRow.hidden role=listitem
          23 MenuRow at=64,432,704,54 by=panel heard="Compressed discs" limit=0,0,1280,800 name=SettingsRow.discs role=listitem
          24 MenuRow at=64,486,704,54 by=panel heard="ES-DE media folder, <temp>/EmuSenOwnLookMedia" limit=0,0,1280,800 name=SettingsRow.media role=listitem
          25 MenuRow at=64,540,704,54 by=panel heard="ES-DE theme folder, None" limit=0,0,1280,800 name=SettingsRow.themefolder role=listitem
        26 Border at=800,108,416,578 by=panel limit=0,0,1280,800 name=SettingsPane paints=#26262BFF role=none
          27 StackPanel at=824,132,368,530 by=clip limit=800,108,416,578 role=none
            28 TextBlock name=SettingsLine paints=#D4D4D4FF role=text volatile="its words name the run's own folders"
            29 TextBlock at=824,220,368,80 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="A opens the folder picker; new games are found on the next start or with Check Again in the Library Manager." layout=344.74,79.2 limit=800,108,416,578 name=SettingsLine paints=#8D8D91FF role=text size=22 text="A opens the folder picker; new games are found on the next start or with Check Again in the Library Manager."
        30 Border at=0,702,1280,98 by=panel limit=0,0,1280,800 name=SettingsHelpBar paints=#1B1B1EFF,#707070FF role=none
          31 HintBar at=64,716,1152,30 by=panel heard="Open, Value, Section, Back, Home" limit=0,0,1280,800 role=group
focus none
press down focus="SettingsLayout SettingsScreen" settings="library  art" top=none
press up focus="SettingsLayout SettingsScreen" settings="library  rom" top=none
press back focus="HubLayout HomeScreen" top=none
