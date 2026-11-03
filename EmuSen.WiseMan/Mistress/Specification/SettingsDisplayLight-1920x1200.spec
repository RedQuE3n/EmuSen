# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state SettingsDisplayLight-1920x1200
size 1920 1200
kind menu
root SettingsLayout#SettingsScreen
unit 1.5
slack 2
floor 16
frame 0,0,1920,1200
case SettingsDisplayLight
palette #0000008C #000000AA #000000FF #005FB8FF #0A5A96FF #0E0E10FF #1B1B1EFF #1B6E3CFF #1B7A1BFF #1C1C1CE6 #1F1F1FFF #26262BFF #2A2A2AFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #5F5F5FFF #707070FF #7A5B00FF #8A6300FF #8C8C8CFF #8D8D91FF #9CDCFEFF #A34B1EFF #B3261EFF #B32D12FF #D08770FF #D37070FF #D4D4D4FF #D6D6D6FF #DAA520FF #DCDCDCFF #EEEEF0FF #F3F3F3FF #FF4500FF #FFD700FF #FFFFFFFF
walk none: the screen moves its own focus, drawn as a bar
tree
  0 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
    1 LayoutTransformControl at=0,0,1920,1200 by=panel limit=0,0,1920,1200 role=none
      2 Panel at=0,0,1920,1200 by=panel limit=0,0,1920,1200 paints=#F3F3F3FF role=none
        3 WholeRow at=96,60,1728,66 by=panel heard=Sections limit=0,0,1920,1200 name=SettingsSections role=tab
          4 SectionTab at=96,60,238.5,66 by=panel drawn focus heard=Controllers limit=0,0,1920,1200 op paints=#1F1F1FFF role=tabitem text=CONTROLLERS
            5 AccessText at=120,72,190.5,37.5 box=127,25 by=clip caps=13.999 drawn face=Barlow layout=126.94,24 limit=96,60,238.5,66 owner=4 paints=#1F1F1FFF role=none size=20 text=CONTROLLERS
          6 SectionTab at=346.5,60,156,66 by=panel drawn focus heard=Display limit=0,0,1920,1200 op paints=#000000FF,#005FB8FF role=tabitem text=DISPLAY
            7 AccessText at=370.5,72,108,37.5 box=72,25 by=clip caps=13.999 drawn face=Barlow layout=71.9,24 limit=346.5,60,156,66 owner=6 paints=#000000FF role=none size=20 text=DISPLAY
          8 SectionTab at=514.5,60,139.5,66 by=panel drawn focus heard=Sound limit=0,0,1920,1200 op paints=#1F1F1FFF role=tabitem text=SOUND
            9 AccessText at=538.5,72,91.5,37.5 box=61,25 by=clip caps=13.999 drawn face=Barlow layout=60.84,24 limit=514.5,60,139.5,66 owner=8 paints=#1F1F1FFF role=none size=20 text=SOUND
          10 SectionTab at=666,60,187.5,66 by=panel drawn focus heard=Consoles limit=0,0,1920,1200 op paints=#1F1F1FFF role=tabitem text=CONSOLES
            11 AccessText at=690,72,139.5,37.5 box=93,25 by=clip caps=13.999 drawn face=Barlow layout=92.68,24 limit=666,60,187.5,66 owner=10 paints=#1F1F1FFF role=none size=20 text=CONSOLES
          12 SectionTab at=865.5,60,160.5,66 by=panel drawn focus heard=Library limit=0,0,1920,1200 op paints=#1F1F1FFF role=tabitem text=LIBRARY
            13 AccessText at=889.5,72,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.16,24 limit=865.5,60,160.5,66 owner=12 paints=#1F1F1FFF role=none size=20 text=LIBRARY
          14 SectionTab at=1038,60,160.5,66 by=panel drawn focus heard=EmuSen limit=0,0,1920,1200 op paints=#1F1F1FFF role=tabitem text=EMUSEN
            15 AccessText at=1062,72,112.5,37.5 box=75,25 by=clip caps=13.999 drawn face=Barlow layout=74.04,24 limit=1038,60,160.5,66 owner=14 paints=#1F1F1FFF role=none size=20 text=EMUSEN
        16 SelectionStack at=96,162,1056,810 by=panel heard=Display limit=0,0,1920,1200 name=SettingsRows role=list
          17 MenuRow at=96,162,1056,81 by=panel focus heard="Picture shape, TV shape" limit=0,0,1920,1200 name=SettingsRow.shape role=listitem
          18 MenuRow at=96,243,1056,81 by=panel focus heard="CRT shader, Off" limit=0,0,1920,1200 name=SettingsRow.crt role=listitem
          19 MenuRow at=96,324,1056,81 by=panel focus heard="Shader levers" limit=0,0,1920,1200 name=SettingsRow.levers role=listitem
          20 MenuRow at=96,405,1056,81 by=panel focus heard=Shaders limit=0,0,1920,1200 name=SettingsRow.shaders role=listitem
          21 MenuRow at=96,486,1056,81 by=panel focus heard="Crop overscan, Whole picture" limit=0,0,1920,1200 name=SettingsRow.crop role=listitem
          22 MenuRow at=96,567,1056,81 by=panel focus heard="Text size, Default" limit=0,0,1920,1200 name=SettingsRow.textsize role=listitem
          23 MenuRow at=96,648,1056,81 by=panel focus heard="Background art on Home, On" limit=0,0,1920,1200 name=SettingsRow.art role=listitem
          24 MenuRow at=96,729,1056,81 by=panel focus heard="Reduced motion, System" limit=0,0,1920,1200 name=SettingsRow.motion role=listitem
          25 MenuRow at=96,810,1056,81 by=panel focus heard="Light or dark, Light" limit=0,0,1920,1200 name=SettingsRow.variant role=listitem
          26 MenuRow at=96,891,1056,81 by=panel focus heard="Play videos, On" limit=0,0,1920,1200 name=SettingsRow.videos role=listitem
        27 Border at=1200,162,624,867 by=panel limit=0,0,1920,1200 name=SettingsPane paints=#FFFFFFFF role=none
          28 StackPanel at=1236,198,552,795 by=clip limit=1200,162,624,867 role=none
            29 StackPanel at=1236,198,552,240 by=clip limit=1200,162,624,867 name=SettingsVariants role=none
              30 ThemeVariantScope at=1236,198,264,240 by=clip limit=1200,162,624,867 name=SettingsVariant.Dark role=none
                31 Border at=1236,198,264,240 by=clip limit=1200,162,624,867 paints=#1B1B1EFF,#707070FF role=none
                  32 TextBlock at=1255.5,217.5,225,219 box=150,146 by=clip caps=13.999 drawn face=Barlow heard=Dark layout=39.44,24 limit=1200,162,624,867 paints=#D4D4D4FF role=text size=20 text=Dark
              33 ThemeVariantScope at=1524,198,264,240 by=clip limit=1200,162,624,867 name=SettingsVariant.Light role=none
                34 Border at=1524,198,264,240 by=clip limit=1200,162,624,867 paints=#8C8C8CFF,#F3F3F3FF role=none
                  35 TextBlock at=1543.5,217.5,225,219 box=150,146 by=clip caps=13.999 drawn face=Barlow heard=Light layout=43.72,24 limit=1200,162,624,867 paints=#1F1F1FFF role=text size=20 text=Light
            36 TextBlock at=1236,456,552,120 box=368,80 by=clip caps=15.3989 drawn face=Barlow heard="Dark is the big screen's default; Light follows the desktop's choice when you ask it to." layout=352.2,79.2 limit=1200,162,624,867 name=SettingsLine paints=#1F1F1FFF role=text size=22 text="Dark is the big screen's default; Light follows the desktop's choice when you ask it to."
        37 Border at=0,1053,1920,147 by=panel limit=0,0,1920,1200 name=SettingsHelpBar paints=#8C8C8CFF,#F3F3F3FF role=none
          38 HintBar at=96,1074,1728,45 by=panel heard="Change, Value, Section, Back, Home" limit=0,0,1920,1200 role=group
focus none
press down focus="MenuRow SettingsRow.videos" settings="display  videos" top=none
press up focus="MenuRow SettingsRow.variant" settings="display  variant" top=none
press back focus="HubLayout HomeScreen" top=none
