# EmuSen interface specification, recorded from Mistress; EMUSEN_INTERFACE_SPEC=record rewrites it - see EmuSen_InterfaceSpec.md.
spec 1
state GraphicsSettingsPlayStationNamedCards-1920x1200
size 1920 1200
kind sheet
root Border
unit 1.5
slack 2
floor 16
frame 350.48,37,1219.05,1002
title "Graphics Settings"
case GraphicsSettingsPlayStationNamedCards
entered shoulders
palette #0000008C #000000AA #000000B8 #000000FF #0E0E10FF #1B1B1EFF #1C1C1CE6 #26262BFF #32CD32FF #333337FF #35A064FF #4C9AE8FF #707070FF #8D8D91FF #9CDCFEFF #D08770FF #D37070FF #D4D4D4FF #DAA520FF #DCDCDCFF #EEEEF0FF #FF4500FF #FFD700FF #FFFFFFFF
tree
  0 ContentControl at=350,187,1221,822 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=none seen=350.48,187,1219.05,822
    1 DockPanel at=410,229,1101,756 by=panel limit=350.48,187,1219.05,822 role=none
      2 HintText at=410,229,1101,72 box=734,48 by=panel caps=13.999 drawn face="Barlow Condensed" heard="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too." layout=733.68,48 limit=350.48,187,1219.05,822 paints=#8D8D91FF role=text size=20 text="What each console's core does with its picture and its machine. A change is saved at once and reaches a running game between frames unless its hint says otherwise; a game loaded later reads these too."
      3 ButtonBar at=1053.5,919,457.5,66 by=panel limit=350.48,187,1219.05,822 paints=#D4D4D4FF role=toolbar
        4 Button at=1053.5,919,315,66 by=clip drawn focus heard="RESET THIS CONSOLE" limit=1053.5,919,457.5,66 op paints=#707070FF,#D4D4D4FF role=button text="Reset This Console"
          5 TextBlock at=1079,928,264,48 box=176,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="RESET THIS CONSOLE" layout=175.14,31.2 limit=1053.5,919,315,66 owner=4 paints=#D4D4D4FF role=text size=26 text="RESET THIS CONSOLE"
        6 Button at=1380.5,919,130.5,66 by=clip drawn focus heard=CLOSE limit=1053.5,919,457.5,66 op paints=#707070FF,#D4D4D4FF role=button text=Close
          7 TextBlock at=1406,928,79.5,48 box=53,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=CLOSE layout=52.75,31.2 limit=1380.5,919,130.5,66 owner=6 paints=#D4D4D4FF role=text size=26 text=CLOSE
      8 Tabs at=410,316,1101,588 by=panel limit=350.48,187,1219.05,822 name=ConsoleTabs paints=#D4D4D4FF role=tab
        9 TabItem at=410,316,93,72 by=clip drawn focus heard=NES limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text=NES
          10 TextBlock at=428,325,57,54 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=NES layout=37.71,36 limit=410,316,93,72 owner=9 paints=#8D8D91FF role=text size=30 text=NES
        11 TabItem at=503,316,75,72 by=clip drawn focus heard=GB limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text=GB
          12 TextBlock at=521,325,39,54 box=26,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GB layout=25.77,36 limit=503,316,75,72 owner=11 paints=#8D8D91FF role=text size=30 text=GB
        13 TabItem at=578,316,111,72 by=clip drawn focus heard=SNES limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text=SNES
          14 TextBlock at=596,325,75,54 box=50,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SNES layout=49.53,36 limit=578,316,111,72 owner=13 paints=#8D8D91FF role=text size=30 text=SNES
        15 TabItem at=689,316,93,72 by=clip drawn focus heard=N64 limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text=N64
          16 TextBlock at=707,325,57,54 box=38,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=N64 layout=37.98,36 limit=689,316,93,72 owner=15 paints=#8D8D91FF role=text size=30 text=N64
        17 TabItem at=782,316,156,72 by=clip drawn focus heard=SG-1000 limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text=SG-1000
          18 TextBlock at=800,325,120,54 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=SG-1000 layout=79.8,36 limit=782,316,156,72 owner=17 paints=#8D8D91FF role=text size=30 text=SG-1000
        19 TabItem at=938,316,267,72 by=clip drawn focus heard="MASTER SYSTEM" limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text="Master System"
          20 TextBlock at=956,325,231,54 box=154,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="MASTER SYSTEM" layout=153.69,36 limit=938,316,267,72 owner=19 paints=#8D8D91FF role=text size=30 text="MASTER SYSTEM"
        21 TabItem at=1205,316,156,72 by=clip drawn focus heard=GENESIS limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text=Genesis
          22 TextBlock at=1223,325,120,54 box=80,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=GENESIS layout=79.89,36 limit=1205,316,156,72 owner=21 paints=#8D8D91FF role=text size=30 text=GENESIS
        23 TabItem at=410,388,157.5,72 by=clip drawn focus heard="SEGA CD" limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text="Sega CD"
          24 TextBlock at=428,397,121.5,54 box=81,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="SEGA CD" layout=80.43,36 limit=410,388,157.5,72 owner=23 paints=#8D8D91FF role=text size=30 text="SEGA CD"
        25 TabItem at=567.5,388,220.5,72 by=clip drawn focus heard=PLAYSTATION limit=410,316,1101,588 op paints=#EEEEF0FF role=tabitem sideways text=PlayStation
          26 TextBlock at=585.5,397,184.5,54 box=123,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard=PLAYSTATION layout=122.67,36 limit=567.5,388,220.5,72 owner=25 paints=#EEEEF0FF role=text size=30 text=PLAYSTATION
        27 TabItem at=788,388,247.5,72 by=clip drawn focus heard="PLAYSTATION 2" limit=410,316,1101,588 paints=#8D8D91FF role=tabitem sideways text="PlayStation 2"
          28 TextBlock at=806,397,211.5,54 box=141,36 by=clip caps=20.9985 drawn face="Barlow Condensed" heard="PLAYSTATION 2" layout=140.19,36 limit=788,388,247.5,72 owner=27 paints=#8D8D91FF role=text size=30 text="PLAYSTATION 2"
        29 ScrollViewer at=428,463,1065,441 by=clip fades=b limit=410,463,1101,441 overflow=3943.5 paints=#707070FF,#D4D4D4FF,#FFFFFFFF role=scrollviewer viewport=428,463,1065,441
          30 StackPanel at=428,475,1065,4372.5 by=scroll in=29 limit=428,463,1065,441 paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=none seen=428,475,1065,429
            31 FieldRow at=428,475,1065,193.5 by=scroll heard="Shader and shape" in=29 limit=428,463,1065,441 paints=#D4D4D4FF role=group
              32 TextBlock at=428,475,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Shader and shape" in=29 layout=141.36,28.8 limit=428,475,1065,193.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text="Shader and shape"
              33 HintText at=428,524.5,1065,72 box=710,48 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square." in=29 layout=689.14,48 limit=428,475,1065,193.5 paints=#8D8D91FF part role=text size=20 text="The shader drawn over the picture, chosen in the Shaders window; and its shape, the console's screen's or each pixel square."
              34 DockPanel at=428,602.5,1065,66 by=clip heard="Shader and shape" in=29 limit=428,475,1065,193.5 role=none
                35 Button at=428,602.5,187.5,66 by=clip drawn focus heard=SHADERS... in=29 limit=428,475,1065,193.5 name=PlayStation.Shaders op paints=#707070FF,#D4D4D4FF role=button text=Shaders...
                  36 TextBlock at=453.5,611.5,136.5,48 box=91,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard=SHADERS... in=29 layout=90.48,31.2 limit=428,602.5,187.5,66 owner=35 paints=#EEEEF0FF role=text size=26 text=SHADERS...
                37 Dropdown at=1311.5,602.5,181.5,66 by=clip drawn focus heard="Shader and shape" in=29 limit=428,475,1065,193.5 name=PlayStation.PictureShape op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=combobox sideways
                  38 TextBlock at=1329.5,610,115.5,48 box=77,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="TV shape" in=29 layout=76.73,31.2 limit=1329.5,610,115.5,48 owner=37 paints=#D4D4D4FF role=text size=26 text="TV shape"
                  39 ViewboxContainer at=1460,630.6,18,9.81 by=clip in=29 limit=1460,630.6,18,9.81 nohit role=none
                40 TextBlock at=633.5,614.5,660,43.5 box=440,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=None in=29 layout=39.6,28.8 limit=428,475,1065,193.5 name=PlayStation.ShaderInUse paints=#D4D4D4FF role=text size=24 text=None
            41 FieldRow at=428,686.5,1065,157.5 by=scroll heard=Overscan in=29 limit=428,463,1065,441 paints=#D4D4D4FF role=group
              42 TextBlock at=428,686.5,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard=Overscan in=29 layout=71.98,28.8 limit=428,686.5,1065,157.5 name=PART_Label paints=#D4D4D4FF part role=text size=24 text=Overscan
              43 HintText at=428,736,1065,36 box=710,24 by=clip caps=13.999 drawn face="Barlow Condensed" heard="The whole picture, the part a television showed, or a crop of your own." in=29 layout=455.22,24 limit=428,686.5,1065,157.5 paints=#8D8D91FF part role=text size=20 text="The whole picture, the part a television showed, or a crop of your own."
              44 StackPanel at=428,778,1065,66 by=clip heard=Overscan in=29 limit=428,686.5,1065,157.5 role=none
                45 Dropdown at=428,778,240,66 by=clip drawn focus heard=Overscan in=29 limit=428,686.5,1065,157.5 name=PlayStation.Overscan op paints=#0E0E10FF,#707070FF,#D4D4D4FF role=combobox sideways
                  46 TextBlock at=446,785.5,174,48 box=116,32 by=clip caps=18.1987 drawn face="Barlow Condensed" heard="Whole picture" in=29 layout=115.73,31.2 limit=446,785.5,174,48 owner=45 paints=#D4D4D4FF role=text size=26 text="Whole picture"
                  47 ViewboxContainer at=635,806.1,18,9.81 by=clip in=29 limit=635,806.1,18,9.81 nohit role=none
            48 FieldRow at=428,862,1065,301.5 by=scroll heard="Memory Card" in=29 limit=428,463,1065,441 name=PlayStation.Cards.Row paints=#0E0E10FF,#707070FF,#8D8D91FF,#D4D4D4FF role=group seen=428,862,1065,42
              49 TextBlock at=428,862,1065,43.5 box=710,29 by=clip caps=16.7988 drawn face="Barlow Condensed" heard="Memory Card" in=29 layout=102.36,28.8 limit=428,862,1065,301.5 name=PART_Label paints=#D4D4D4FF part role=text seen=428,862,1065,42 size=24 text="Memory Card"
focus 35
edge 4 up 25
edge 4 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/16/0/2/0
edge 4 left path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0
edge 4 right 6
edge 6 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0
edge 6 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/16/0/2/0
edge 6 left 4
edge 6 right same
edge 25 up same
edge 25 down 35
edge 35 up 25
edge 35 down 45
edge 35 left same
edge 35 right 37
edge 37 up 25
edge 37 down 45
edge 45 up 35
edge 45 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/12/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/12/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/11/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/12/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/13/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/13/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/12/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/13/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/13/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/16/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0 right 4
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/16/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/15/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/16/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/17/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/17/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/16/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/17/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/18/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/17/0/2/0 left same
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/17/0/2/0 right 4
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 up 45
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/3/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/4/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/5/0/2/0/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/6/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/7/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0 up path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/8/0/2/0
edge path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/9/0/2/0 down path:0/3/0/0/2/3/0/0/1/0/0/0/2/0/0/1/1/0/0/0/0/10/0/2/0
press pageup focus="Button 'Shaders...'" tab="Sega CD" title="Graphics Settings" top=GraphicsSettingsWindow
press pagedown focus="Button 'Shaders...'" tab=PlayStation title="Graphics Settings" top=GraphicsSettingsWindow
press first focus="Button 'Shaders...'" tab=PlayStation title="Graphics Settings" top=GraphicsSettingsWindow
press last focus="Button 'Shaders...'" tab=PlayStation title="Graphics Settings" top=GraphicsSettingsWindow
press back top=none
