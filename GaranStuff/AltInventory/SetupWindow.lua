-- changing Settings.showIcon from a boolean (checkbox) to a three state (radiobuttongroup) 0=never, 1=minimized, 3=always

if importPath==nil then importPath=string.gsub(getfenv(1)._.Name,"%.SetupWindow","").."." end
if resourcePath==nil then resourcePath=string.gsub(importPath,"%.","/").."Resources/" end
-- use a do block so that we have local variables that get discarded after initialization
do
	-- yes, all this initial size stuff is duplicated in .Refresh, but it has to be done here to handle Settings.SWTop==nil and Settings.SWLeft==nil
	local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
	fontMetric:SetFont(Settings.fontFace)
	local tmpText={28,28,30,31,33,34,35,55} -- all captions
	local captionWidth=0
	local controlHeight=fontSize
	-- we have controls that can not go below 20 pixel height
	if controlHeight<20 then controlHeight=20 end
	for k,v in ipairs(tmpText) do
		local width=fontMetric:GetTextWidth("Resource[language][v]:")
		if width>captionWidth then captionWidth=width end
	end
	if captionWidth>displayWidth/4 then captionWidth=displayWidth/4 end -- gotta have some sort of reasonable limit

	tmpText={24,25,26}
	local tabWidth=0
	for k,v in ipairs(tmpText) do
		local width=fontMetric:GetTextWidth("Resource[language][v]:")+6
		if width>tabWidth then tabWidth=width end
	end

	setupWindow = Turbine.UI.Lotro.Window()
	setupWindow.Loaded=false
	setupWindow:SetZOrder(1)
	local initHeight=150+fontSize+10*(controlHeight+5) -- 10 being the number of option rows per tab + 1 for tab controls
	local initWidth=captionWidth+450
	if initWidth<tabWidth*3+34 then initWidth=tabWidth*3+34 end

	if Settings.SWTop==nil then Settings.SWTop=(displayHeight - initHeight) / 2 end
	if Settings.SWLeft==nil then Settings.SWLeft=(displayWidth - initWidth) / 2 end

	if initHeight>displayHeight then initHeight=displayHeight end
	if initHeight+Settings.SWTop>displayHeight then Settings.SWTop=displayHeight-initHeight end
	if initWidth>displayWidth then initWidth=displayWidth end
	if initWidth+Settings.SWLeft>displayWidth then Settings.SWLeft=displayWidth-initWidth end

	setupWindow:SetPosition( Settings.SWLeft, Settings.SWTop )
	setupWindow:SetHeight(initHeight)
	setupWindow:SetWidth(initWidth)
	setupWindow:SetOpacity(Settings.opacity)
	setupWindow:SetVisible(false)

	setupWindow.TabTrim=Turbine.UI.Label()
	setupWindow.TabTrim:SetParent(setupWindow)
	setupWindow.TabTrim:SetBackColor(Tab1TabTrimColor)

	setupWindow.TabBack=Turbine.UI.Label()
	setupWindow.TabBack:SetParent(setupWindow.TabTrim)
	setupWindow.TabBack:SetPosition(1,1)
	setupWindow.TabBack:SetBackColor(Tab1TabTrimColor)

	-- list tab
	setupWindow.Tab1Tab=Turbine.UI.Label()
	setupWindow.Tab1Tab:SetParent(setupWindow.TabBack)
	setupWindow.Tab1Tab:SetPosition(1,1)
	setupWindow.Tab1Tab:SetBackColor(Tab1TabBackColor)

	-- defaults tab
	setupWindow.Tab2Tab=Turbine.UI.Label()
	setupWindow.Tab2Tab:SetParent(setupWindow.TabBack)
	setupWindow.Tab2Tab:SetPosition(1,1)
	setupWindow.Tab2Tab:SetBackColor(Tab2TabBackColor)
	setupWindow.Tab2Tab:SetVisible(false)

	setupWindow.Tab3Tab=Turbine.UI.Label()
	setupWindow.Tab3Tab:SetParent(setupWindow.TabBack)
	setupWindow.Tab3Tab:SetPosition(1,1)
	setupWindow.Tab3Tab:SetBackColor(Tab3TabBackColor)
	setupWindow.Tab3Tab:SetVisible(false)

	-- tab buttons
	setupWindow.Tab1Button=Turbine.UI.Control()
	setupWindow.Tab1Button:SetParent(setupWindow)
	setupWindow.Tab1Button:SetPosition(10,42)

	setupWindow.Tab1LeftCorner=Turbine.UI.Control()
	setupWindow.Tab1LeftCorner:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1LeftCorner:SetPosition(0,0)
	setupWindow.Tab1LeftCorner:SetSize(5,4)
	setupWindow.Tab1LeftCorner:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab1LeftCorner:SetBackground(resourcePath.."TabLeftCorner.tga")
	setupWindow.Tab1LeftCorner:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab1LeftCorner:SetBackColor(Tab1TabTrimColor)
	setupWindow.Tab1LeftCorner:SetMouseVisible(false)

	setupWindow.Tab1Top=Turbine.UI.Control()
	setupWindow.Tab1Top:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1Top:SetPosition(5,0)
	setupWindow.Tab1Top:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab1Top:SetBackground(resourcePath.."TabTop.tga")
	setupWindow.Tab1Top:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab1Top:SetBackColor(Tab1TabTrimColor)
	setupWindow.Tab1Top:SetMouseVisible(false)

	setupWindow.Tab1RightCorner=Turbine.UI.Control()
	setupWindow.Tab1RightCorner:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1RightCorner:SetSize(5,4)
	setupWindow.Tab1RightCorner:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab1RightCorner:SetBackground(resourcePath.."TabRightCorner.tga")
	setupWindow.Tab1RightCorner:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab1RightCorner:SetBackColor(Tab1TabTrimColor)
	setupWindow.Tab1RightCorner:SetMouseVisible(false)

	setupWindow.Tab1LeftSide=Turbine.UI.Control()
	setupWindow.Tab1LeftSide:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1LeftSide:SetPosition(0,4)
	setupWindow.Tab1LeftSide:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab1LeftSide:SetBackground(resourcePath.."TabLeftSide.tga")
	setupWindow.Tab1LeftSide:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab1LeftSide:SetBackColor(Tab1TabTrimColor)
	setupWindow.Tab1LeftSide:SetMouseVisible(false)

	setupWindow.Tab1Center=Turbine.UI.Control()
	setupWindow.Tab1Center:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1Center:SetPosition(3,4)
	setupWindow.Tab1Center:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab1Center:SetBackground(resourcePath.."TabCenter.tga")
	setupWindow.Tab1Center:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab1Center:SetBackColor(Tab1TabTrimColor)
	setupWindow.Tab1Center:SetMouseVisible(false)

	setupWindow.Tab1RightSide=Turbine.UI.Control()
	setupWindow.Tab1RightSide:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1RightSide:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab1RightSide:SetBackground(resourcePath.."TabRightSide.tga")
	setupWindow.Tab1RightSide:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab1RightSide:SetBackColor(Tab1TabTrimColor)
	setupWindow.Tab1RightSide:SetMouseVisible(false)

	setupWindow.Tab1Text=Turbine.UI.Label()
	setupWindow.Tab1Text:SetParent(setupWindow.Tab1Button)
	setupWindow.Tab1Text:SetPosition(3,4)
	setupWindow.Tab1Text:SetZOrder(1)
	setupWindow.Tab1Text:SetBackColor(Tab1TabBackColor)
	setupWindow.Tab1Text:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
	setupWindow.Tab1Text:SetText(Resource[language][24])
	setupWindow.Tab1Text:SetMouseVisible(false)
	setupWindow.Tab1Button.MouseClick=function()
		local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
		setupWindow.Tab1Tab:SetVisible(true)
		setupWindow.TabTrim:SetBackColor(Tab1TabTrimColor)
		setupWindow.TabBack:SetBackColor(Tab1TabTrimColor)
		setupWindow.Tab2Tab:SetVisible(false)
		setupWindow.Tab3Tab:SetVisible(false)
		setupWindow.Tab1Button:SetTop(42)
		setupWindow.Tab1Button:SetHeight(fontSize+6)
		setupWindow.Tab2Button:SetTop(46)
		setupWindow.Tab2Button:SetHeight(fontSize)
		setupWindow.Tab3Button:SetTop(46)
		setupWindow.Tab3Button:SetHeight(fontSize)
	end
	setupWindow.Tab2Button=Turbine.UI.Control()
	setupWindow.Tab2Button:SetParent(setupWindow)

	setupWindow.Tab2LeftCorner=Turbine.UI.Control()
	setupWindow.Tab2LeftCorner:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2LeftCorner:SetPosition(0,0)
	setupWindow.Tab2LeftCorner:SetSize(5,4)
	setupWindow.Tab2LeftCorner:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab2LeftCorner:SetBackground(resourcePath.."TabLeftCorner.tga")
	setupWindow.Tab2LeftCorner:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab2LeftCorner:SetBackColor(Tab2TabTrimColor)
	setupWindow.Tab2LeftCorner:SetMouseVisible(false)

	setupWindow.Tab2Top=Turbine.UI.Control()
	setupWindow.Tab2Top:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2Top:SetPosition(5,0)
	setupWindow.Tab2Top:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab2Top:SetBackground(resourcePath.."TabTop.tga")
	setupWindow.Tab2Top:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab2Top:SetBackColor(Tab2TabTrimColor)
	setupWindow.Tab2Top:SetMouseVisible(false)

	setupWindow.Tab2RightCorner=Turbine.UI.Control()
	setupWindow.Tab2RightCorner:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2RightCorner:SetSize(5,4)
	setupWindow.Tab2RightCorner:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab2RightCorner:SetBackground(resourcePath.."TabRightCorner.tga")
	setupWindow.Tab2RightCorner:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab2RightCorner:SetBackColor(Tab2TabTrimColor)
	setupWindow.Tab2RightCorner:SetMouseVisible(false)

	setupWindow.Tab2LeftSide=Turbine.UI.Control()
	setupWindow.Tab2LeftSide:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2LeftSide:SetPosition(0,4)
	setupWindow.Tab2LeftSide:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab2LeftSide:SetBackground(resourcePath.."TabLeftSide.tga")
	setupWindow.Tab2LeftSide:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab2LeftSide:SetBackColor(Tab2TabTrimColor)
	setupWindow.Tab2LeftSide:SetMouseVisible(false)

	setupWindow.Tab2Center=Turbine.UI.Control()
	setupWindow.Tab2Center:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2Center:SetPosition(3,4)
	setupWindow.Tab2Center:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab2Center:SetBackground(resourcePath.."TabCenter.tga")
	setupWindow.Tab2Center:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab2Center:SetBackColor(Tab2TabTrimColor)
	setupWindow.Tab2Center:SetMouseVisible(false)

	setupWindow.Tab2RightSide=Turbine.UI.Control()
	setupWindow.Tab2RightSide:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2RightSide:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab2RightSide:SetBackground(resourcePath.."TabRightSide.tga")
	setupWindow.Tab2RightSide:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab2RightSide:SetBackColor(Tab2TabTrimColor)
	setupWindow.Tab2RightSide:SetMouseVisible(false)

	setupWindow.Tab2Text=Turbine.UI.Label()
	setupWindow.Tab2Text:SetParent(setupWindow.Tab2Button)
	setupWindow.Tab2Text:SetPosition(3,4)
	setupWindow.Tab2Text:SetZOrder(1)
	setupWindow.Tab2Text:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
	setupWindow.Tab2Text:SetText(Resource[language][25])
	setupWindow.Tab2Text:SetMouseVisible(false)
	setupWindow.Tab2Button.MouseClick=function()
		local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
		setupWindow.Tab2Tab:SetVisible(true)
		setupWindow.TabTrim:SetBackColor(Tab2TabTrimColor)
		setupWindow.TabBack:SetBackColor(Tab2TabTrimColor)
		setupWindow.Tab1Tab:SetVisible(false)
		setupWindow.Tab3Tab:SetVisible(false)
		setupWindow.Tab2Button:SetTop(42)
		setupWindow.Tab2Button:SetHeight(fontSize+6)
		setupWindow.Tab1Button:SetTop(46)
		setupWindow.Tab1Button:SetHeight(fontSize)
		setupWindow.Tab3Button:SetTop(46)
		setupWindow.Tab3Button:SetHeight(fontSize)
	end

	setupWindow.Tab3Button=Turbine.UI.Control()
	setupWindow.Tab3Button:SetParent(setupWindow)

	setupWindow.Tab3LeftCorner=Turbine.UI.Control()
	setupWindow.Tab3LeftCorner:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3LeftCorner:SetPosition(0,0)
	setupWindow.Tab3LeftCorner:SetSize(5,4)
	setupWindow.Tab3LeftCorner:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab3LeftCorner:SetBackground(resourcePath.."TabLeftCorner.tga")
	setupWindow.Tab3LeftCorner:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab3LeftCorner:SetBackColor(Tab3TabTrimColor)
	setupWindow.Tab3LeftCorner:SetMouseVisible(false)

	setupWindow.Tab3Top=Turbine.UI.Control()
	setupWindow.Tab3Top:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3Top:SetPosition(5,0)
	setupWindow.Tab3Top:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab3Top:SetBackground(resourcePath.."TabTop.tga")
	setupWindow.Tab3Top:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab3Top:SetBackColor(Tab3TabTrimColor)
	setupWindow.Tab3Top:SetMouseVisible(false)

	setupWindow.Tab3RightCorner=Turbine.UI.Control()
	setupWindow.Tab3RightCorner:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3RightCorner:SetSize(5,4)
	setupWindow.Tab3RightCorner:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab3RightCorner:SetBackground(resourcePath.."TabRightCorner.tga")
	setupWindow.Tab3RightCorner:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab3RightCorner:SetBackColor(Tab3TabTrimColor)
	setupWindow.Tab3RightCorner:SetMouseVisible(false)

	setupWindow.Tab3LeftSide=Turbine.UI.Control()
	setupWindow.Tab3LeftSide:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3LeftSide:SetPosition(0,4)
	setupWindow.Tab3LeftSide:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab3LeftSide:SetBackground(resourcePath.."TabLeftSide.tga")
	setupWindow.Tab3LeftSide:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab3LeftSide:SetBackColor(Tab3TabTrimColor)
	setupWindow.Tab3LeftSide:SetMouseVisible(false)

	setupWindow.Tab3Center=Turbine.UI.Control()
	setupWindow.Tab3Center:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3Center:SetPosition(3,4)
	setupWindow.Tab3Center:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab3Center:SetBackground(resourcePath.."TabCenter.tga")
	setupWindow.Tab3Center:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab3Center:SetBackColor(Tab3TabTrimColor)
	setupWindow.Tab3Center:SetMouseVisible(false)

	setupWindow.Tab3RightSide=Turbine.UI.Control()
	setupWindow.Tab3RightSide:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3RightSide:SetBlendMode(Turbine.UI.BlendMode.AlphaBlend)
	setupWindow.Tab3RightSide:SetBackground(resourcePath.."TabRightSide.tga")
	setupWindow.Tab3RightSide:SetBackColorBlendMode(Turbine.UI.BlendMode.Color)
	setupWindow.Tab3RightSide:SetBackColor(Tab3TabTrimColor)
	setupWindow.Tab3RightSide:SetMouseVisible(false)

	setupWindow.Tab3Text=Turbine.UI.Label()
	setupWindow.Tab3Text:SetParent(setupWindow.Tab3Button)
	setupWindow.Tab3Text:SetPosition(3,4)
	setupWindow.Tab3Text:SetZOrder(1)
	setupWindow.Tab3Text:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
	setupWindow.Tab3Text:SetText(Resource[language][26])
	setupWindow.Tab3Text:SetMouseVisible(false)
	setupWindow.Tab3Button.MouseClick=function()
		local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
		setupWindow.Tab3Tab:SetVisible(true)
		setupWindow.TabTrim:SetBackColor(Tab3TabTrimColor)
		setupWindow.TabBack:SetBackColor(Tab3TabTrimColor)
		setupWindow.Tab1Tab:SetVisible(false)
		setupWindow.Tab2Tab:SetVisible(false)
		setupWindow.Tab3Button:SetTop(42)
		setupWindow.Tab3Button:SetHeight(fontSize+6)
		setupWindow.Tab1Button:SetTop(46)
		setupWindow.Tab1Button:SetHeight(fontSize)
		setupWindow.Tab2Button:SetTop(46)
		setupWindow.Tab2Button:SetHeight(fontSize)
	end
-- tab1 controls
	setupWindow.LanguageCaption=Turbine.UI.Label()
	setupWindow.LanguageCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.LanguageCaption:SetPosition(5,10)
	setupWindow.LanguageCaption:SetForeColor(Settings.fontColor)
	setupWindow.LanguageCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.LanguageCaption:SetText(Resource[language][28]..":")
	setupWindow.LanguageList=DropDownList()
	setupWindow.LanguageList:SetParent(setupWindow.Tab1Tab)
	setupWindow.LanguageList:SetBackColor(Settings.backColor)
	setupWindow.LanguageList:SetCurrentBackColor(Settings.backColor)
	setupWindow.LanguageList:SetBorderColor(Settings.trimColor)
	setupWindow.LanguageList:SetZOrder(1)
	setupWindow.LanguageList:SetTextColor(Settings.listTextColor)
	setupWindow.LanguageList:AddItem(Resource[clientLanguage][157],0)
	for k,v in pairs(Resource) do
		setupWindow.LanguageList:AddItem(v[1],k)
	end
	setupWindow.LanguageList:SetSelectedIndex(Settings.language+1)

	setupWindow.TrimCaption=Turbine.UI.Label()
	setupWindow.TrimCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.TrimCaption:SetForeColor(Settings.fontColor)
	setupWindow.TrimCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.TrimCaption:SetText(Resource[language][29]..":")
	setupWindow.TrimColor=ColorPicker()
	setupWindow.TrimColor:SetParent(setupWindow.Tab1Tab)
	setupWindow.BackCaption=Turbine.UI.Label()
	setupWindow.BackCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.BackCaption:SetForeColor(Settings.fontColor)
	setupWindow.BackCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.BackCaption:SetText(Resource[language][30]..":")
	setupWindow.BackColor=ColorPicker()
	setupWindow.BackColor:SetParent(setupWindow.Tab1Tab)
	setupWindow.TextCaption=Turbine.UI.Label()
	setupWindow.TextCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.TextCaption:SetForeColor(Settings.fontColor)
	setupWindow.TextCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.TextCaption:SetText(Resource[language][31]..":")
	setupWindow.TextColor=ColorPicker()
	setupWindow.TextColor:SetParent(setupWindow.Tab1Tab)
	setupWindow.HeadingsCaption=Turbine.UI.Label()
	setupWindow.HeadingsCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.HeadingsCaption:SetForeColor(Settings.fontColor)
	setupWindow.HeadingsCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.HeadingsCaption:SetText(Resource[language][33]..":")
	setupWindow.HeadingsColor=ColorPicker()
	setupWindow.HeadingsColor:SetParent(setupWindow.Tab1Tab)
	setupWindow.ListCaption=Turbine.UI.Label()
	setupWindow.ListCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.ListCaption:SetForeColor(Settings.fontColor)
	setupWindow.ListCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.ListCaption:SetText(Resource[language][34]..":")
	setupWindow.ListColor=ColorPicker()
	setupWindow.ListColor:SetParent(setupWindow.Tab1Tab)
	setupWindow.PanelCaption=Turbine.UI.Label()
	setupWindow.PanelCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.PanelCaption:SetForeColor(Settings.fontColor)
	setupWindow.PanelCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.PanelCaption:SetText(Resource[language][35]..":")
	setupWindow.PanelColor=ColorPicker()
	setupWindow.PanelColor:SetParent(setupWindow.Tab1Tab)
	setupWindow.DefaultColors=Turbine.UI.Lotro.Button()
	setupWindow.DefaultColors:SetParent(setupWindow.Tab1Tab)

	setupWindow.DefaultColors:SetSize(200,20)
	setupWindow.DefaultColors:SetFont(Turbine.UI.Lotro.Font.Verdana18)
	setupWindow.DefaultColors:SetText(Resource[language][32])
	setupWindow.DefaultColors.Click=function()
		setupWindow.TrimColor:SetColor(Turbine.UI.Color(.4,.4,.5))
		setupWindow.BackColor:SetColor(Turbine.UI.Color(.05,.05,.05))
		setupWindow.TextColor:SetColor(Turbine.UI.Color(1,.9,.5))
		setupWindow.HeadingsColor:SetColor(Turbine.UI.Color(1,1,1))
		setupWindow.ListColor:SetColor(Turbine.UI.Color(1,1,1))
		setupWindow.PanelColor:SetColor(Turbine.UI.Color(.11,.25,0)) -- 1b 40 0
	end
	setupWindow.ZoomCaption=Turbine.UI.Label()
	setupWindow.ZoomCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.ZoomCaption:SetForeColor(Settings.fontColor)
	setupWindow.ZoomCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.ZoomCaption:SetText(Resource[language][55]..":")
	setupWindow.Zoom=RadioButtonGroup()
	setupWindow.Zoom:SetParent(setupWindow.Tab1Tab)
	setupWindow.Zoom:AddChoice("1x",1,1)
	setupWindow.Zoom:AddChoice("2x",2,2)
	setupWindow.Zoom:AddChoice("4x",4,3)

	setupWindow.FontCaption=Turbine.UI.Label()
	setupWindow.FontCaption:SetParent(setupWindow.Tab1Tab)
	setupWindow.FontCaption:SetForeColor(Settings.fontColor)
	setupWindow.FontCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.FontCaption:SetText(Resource[language][131]..":")
	setupWindow.FontSelect=FontSelect()
	setupWindow.FontSelect:SetParent(setupWindow.Tab1Tab)
--	setupWindow.FontSelect:SetStretchMode(3) -- doesn't work with setup window
	setupWindow.FontSelect:SetBackground(resourcePath.."FontSelect.tga")
	setupWindow.FontSelect:SetZOrder(2)
	setupWindow.FontSelect.FontChanged=function()
		-- defer applying until "save" is clicked
	end

-- tab2 controls
	setupWindow.replaceBags=Turbine.UI.Lotro.CheckBox()
	setupWindow.replaceBags:SetParent(setupWindow.Tab2Tab)
	setupWindow.replaceBags:SetPosition(10,10)
	setupWindow.replaceBags:SetForeColor(Settings.fontColor)
	setupWindow.replaceBags:SetText(Resource[language][38])

	setupWindow.totalsOnly=Turbine.UI.Lotro.CheckBox()
	setupWindow.totalsOnly:SetParent(setupWindow.Tab2Tab)
	setupWindow.totalsOnly:SetForeColor(Settings.fontColor)
	setupWindow.totalsOnly:SetText(Resource[language][53])

	setupWindow.loadMinimized=Turbine.UI.Lotro.CheckBox()
	setupWindow.loadMinimized:SetParent(setupWindow.Tab2Tab)
	setupWindow.loadMinimized:SetForeColor(Settings.fontColor)
	setupWindow.loadMinimized:SetText(Resource[language][39])

	setupWindow.showIconCaption=Turbine.UI.Label()
	setupWindow.showIconCaption:SetParent(setupWindow.Tab2Tab)
	setupWindow.showIconCaption:SetForeColor(Settings.fontColor)
	setupWindow.showIconCaption:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleLeft)
	setupWindow.showIconCaption:SetText(Resource[language][40]..":")

	setupWindow.showIcon=RadioButtonGroup()
	setupWindow.showIcon:SetParent(setupWindow.Tab2Tab)
	setupWindow.showIcon:AddChoice(Resource[language][138],0,1)
	setupWindow.showIcon:AddChoice(Resource[language][139],1,2)
	setupWindow.showIcon:AddChoice(Resource[language][140],2,3)

	setupWindow.iconLeftCaption=Turbine.UI.Label()
	setupWindow.iconLeftCaption:SetParent(setupWindow.Tab2Tab)
	setupWindow.iconLeftCaption:SetForeColor(Settings.fontColor)
	setupWindow.iconLeftCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight)
	setupWindow.iconLeftCaption:SetText(Resource[language][41]..":")
	setupWindow.iconLeftBack=Turbine.UI.Control()
	setupWindow.iconLeftBack:SetParent(setupWindow.Tab2Tab)
	setupWindow.iconLeftBack:SetBackColor(Settings.trimColor)
	setupWindow.iconLeft=Turbine.UI.Lotro.TextBox()
	setupWindow.iconLeft:SetParent(setupWindow.iconLeftBack)
	setupWindow.iconLeft:SetBackColor(Settings.backColor)
	setupWindow.iconLeft:SetPosition(1,1)
	setupWindow.iconLeft:SetText("")

	setupWindow.iconTopCaption=Turbine.UI.Label()
	setupWindow.iconTopCaption:SetParent(setupWindow.Tab2Tab)
	setupWindow.iconTopCaption:SetForeColor(Settings.fontColor)
	setupWindow.iconTopCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight)
	setupWindow.iconTopCaption:SetText(Resource[language][42]..":")

	setupWindow.iconTopBack=Turbine.UI.Control()
	setupWindow.iconTopBack:SetParent(setupWindow.Tab2Tab)
	setupWindow.iconTopBack:SetBackColor(Settings.trimColor)

	setupWindow.iconTop=Turbine.UI.Lotro.TextBox()
	setupWindow.iconTop:SetParent(setupWindow.iconTopBack)
	setupWindow.iconTop:SetBackColor(Settings.backColor)
	setupWindow.iconTop:SetPosition(1,1)
	setupWindow.iconTop:SetText("")

	setupWindow.miniIcon=Turbine.UI.Lotro.CheckBox()
	setupWindow.miniIcon:SetParent(setupWindow.Tab2Tab)
	setupWindow.miniIcon:SetForeColor(Settings.fontColor)
	setupWindow.miniIcon:SetText(Resource[language][52])

	setupWindow.useMinimalHeader=Turbine.UI.Lotro.CheckBox()
	setupWindow.useMinimalHeader:SetParent(setupWindow.Tab2Tab)
	setupWindow.useMinimalHeader:SetForeColor(Settings.fontColor)
	setupWindow.useMinimalHeader:SetText(Resource[language][43])

	setupWindow.defaultToAll=Turbine.UI.Lotro.CheckBox()
	setupWindow.defaultToAll:SetParent(setupWindow.Tab2Tab)
	setupWindow.defaultToAll:SetForeColor(Settings.fontColor)
	setupWindow.defaultToAll:SetText(Resource[language][54])
	setupWindow.defaultToAll:SetVisible(true)

--*** deprecated now that we have Groups
	setupWindow.bagSeparator=Turbine.UI.Lotro.CheckBox()
	setupWindow.bagSeparator:SetParent(setupWindow.Tab2Tab)
	setupWindow.bagSeparator:SetSize(setupWindow.replaceBags:GetWidth(),controlHeight)
	setupWindow.bagSeparator:SetPosition(setupWindow.replaceBags:GetLeft(),setupWindow.defaultToAll:GetTop()+setupWindow.defaultToAll:GetHeight()+10)
	setupWindow.bagSeparator:SetFont(Settings.fontFace)
	setupWindow.bagSeparator:SetForeColor(Settings.fontColor)
	setupWindow.bagSeparator:SetText(Resource[language][44])
	setupWindow.bagSeparator:SetVisible(false)

-- tab3 controls
	setupWindow.Tab3ViewPane=Turbine.UI.Control()
	setupWindow.Tab3ViewPane:SetParent(setupWindow.Tab3Tab)
	setupWindow.Tab3ViewPane:SetLeft(0)
	setupWindow.Tab3VScroll=Turbine.UI.Lotro.ScrollBar()
	setupWindow.Tab3VScroll:SetParent(setupWindow.Tab3Tab)
	setupWindow.Tab3VScroll:SetMinimum(0)
	setupWindow.Tab3VScroll:SetWidth(10)
	setupWindow.Tab3VScroll:SetTop(0)
	setupWindow.Tab3VScroll:SetOrientation( Turbine.UI.Orientation.Vertical)
	setupWindow.Tab3VScroll.ValueChanged=function()
		setupWindow.Tab3ViewPane:SetTop(0-setupWindow.Tab3VScroll:GetValue())
	end

	setupWindow.delayCaption=Turbine.UI.Label()
	setupWindow.delayCaption:SetParent(setupWindow.Tab3ViewPane)
	setupWindow.delayCaption:SetPosition(5,5)
	setupWindow.delayCaption:SetText(Resource[language][134])

	setupWindow.sep1=Turbine.UI.Control()
	setupWindow.sep1:SetParent(setupWindow.Tab3ViewPane)
	setupWindow.sep1:SetLeft(5)
	setupWindow.sep1:SetHeight(1)

	setupWindow.cropDelayCaption=Turbine.UI.Label()
	setupWindow.cropDelayCaption:SetParent(setupWindow.Tab3ViewPane)
	setupWindow.cropDelayCaption:SetText(Resource[language][132])
	setupWindow.cropDelayBack=Turbine.UI.Control()
	setupWindow.cropDelayBack:SetParent(setupWindow.Tab3ViewPane)
	setupWindow.cropDelay=Turbine.UI.Lotro.TextBox()
	setupWindow.cropDelay:SetParent(setupWindow.cropDelayBack)
	setupWindow.cropDelayDesc=Turbine.UI.Label()
	setupWindow.cropDelayDesc:SetParent(setupWindow.Tab3ViewPane)
	setupWindow.cropDelayDesc:SetLeft(5)
	setupWindow.cropDelayDesc:SetText(Resource[language][133])

	setupWindow.sep2=Turbine.UI.Control()
	setupWindow.sep2:SetParent(setupWindow.Tab3ViewPane)
	setupWindow.sep2:SetLeft(5)
	setupWindow.sep2:SetHeight(1)

	setupWindow.Save=Turbine.UI.Lotro.Button()
	setupWindow.Save:SetParent(setupWindow)
	setupWindow.Save:SetFont(Turbine.UI.Lotro.Font.Verdana18)
	setupWindow.Save:SetSize(100,20)
	setupWindow.Save:SetText(Resource[language][36])
	setupWindow.Save.Click=function()
		if not inventoryWindow.settingFont then
			inventoryWindow.settingFont=true
			Settings.fontFace=setupWindow.FontSelect:GetFont()
			inventoryWindow.FontSelect:SetFont(Settings.fontFace)
			-- now need to apply new font... :p
			fontMetric:SetFont(Settings.fontFace)
			-- no need to set setupWindow fields as their font gets set in .Refresh next time window is opened
			getItemEntryLayout()
			applyItemEntryLayout()
			inventoryPanel:Layout()
			inventoryWindow.settingFont=false
		end
		if language~=setupWindow.LanguageList:GetValue() then
			Settings.language=setupWindow.LanguageList:GetValue()
			if Settings.language==0 then
				language=clientLanguage
			else
				language=Settings.language
			end

			-- set language
			setupWindow.Tab1Text:SetText(Resource[language][24])
			setupWindow.Tab2Text:SetText(Resource[language][25])
			setupWindow.Tab3Text:SetText(Resource[language][26])
			setupWindow.LanguageCaption:SetText(Resource[language][28]..":")
			setupWindow.TrimCaption:SetText(Resource[language][29]..":")
			setupWindow.BackCaption:SetText(Resource[language][30]..":")
			setupWindow.TextCaption:SetText(Resource[language][31]..":")
			setupWindow.HeadingsCaption:SetText(Resource[language][33]..":")
			setupWindow.ListCaption:SetText(Resource[language][34]..":")
			setupWindow.PanelCaption:SetText(Resource[language][35]..":")
			setupWindow.DefaultColors:SetText(Resource[language][32])
			setupWindow.ZoomCaption:SetText(Resource[language][55]..":")

			setupWindow.replaceBags:SetText(Resource[language][38])
			setupWindow.loadMinimized:SetText(Resource[language][39])
			setupWindow.showIconCaption:SetText(Resource[language][40])
			setupWindow.showIcon.Choices[1].Caption:SetText(Resource[language][138])
			setupWindow.showIcon.Choices[2].Caption:SetText(Resource[language][139])
			setupWindow.showIcon.Choices[3].Caption:SetText(Resource[language][140])

			setupWindow.iconLeftCaption:SetText(Resource[language][41]..":")
			setupWindow.iconTopCaption:SetText(Resource[language][42]..":")
			setupWindow.useMinimalHeader:SetText(Resource[language][43])
			setupWindow.defaultToAll:SetText(Resource[language][54])
			setupWindow.bagSeparator:SetText(Resource[language][44])
			setupWindow.delayCaption:SetText(Resource[language][134])
			setupWindow.cropDelayCaption:SetText(Resource[language][132])
			setupWindow.cropDelayDesc:SetText(Resource[language][133])

			setupWindow.Save:SetText(Resource[language][36])
			setupWindow.Cancel:SetText(Resource[language][37])
			setupWindow:SetText(Resource[language][27])
			if inventoryWindow.CharList:GetSelectedIndex()==1 then
				inventoryWindow.CapacityDisplay:SetText(Resource[language][10])
			elseif inventoryWindow.CharList:GetSelectedIndex()==2 then
				inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList["Shared Storage"].used).."/"..tostring(CharList["Shared Storage"].capacity)..")")
			elseif inventoryWindow.CharList:GetSelectedIndex()==3 then
				inventoryWindow.CapacityDisplay:SetText(Resource[language][10]) -- "N/A"
			elseif string.find(inventoryWindow.CharList:GetText(),"(EI)")~=nil then
				inventoryWindow.CapacityDisplay:SetText(Resource[language][10])
			else
				inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(CharList[inventoryWindow.CharList:GetText()].used).."/"..tostring(CharList[inventoryWindow.CharList:GetText()].capacity)..")")
			end
			inventoryWindow.CharList.ListData:GetItem(1):SetText(Resource[language][12])
			if inventoryWindow.CharList:GetSelectedIndex()==1 then
				inventoryWindow.CharList.CurrentValue:SetText(Resource[language][12])
			end
			inventoryWindow.CharList.ListData:GetItem(2):SetText(Resource[language][9])
			if inventoryWindow.CharList:GetSelectedIndex()==2 then
				inventoryWindow.CharList.CurrentValue:SetText(Resource[language][9])
			end

			inventoryWindow:SetText("      "..Resource[language][2].."      ")
			inventoryWindow.CapacityEmpty:SetText(Resource[language][4])
			inventoryWindow.CapacityFull:SetText(Resource[language][6])
			inventoryWindow.CharCaption:SetText(Resource[language][13]..":")
			inventoryPanel.FilterCaption:SetText(Resource[language][11]..":")
			inventoryPanel.SearchCaption:SetText(Resource[language][14]..":")
			inventoryPanel.ShowAllButton:SetText(Resource[language][23])
			inventoryPanel.SortCaption:SetText(Resource[language][15]..":")
			inventoryPanel.SortList.ListData:GetItem(1):SetText(Resource[language][17])
			if inventoryPanel.SortList:GetSelectedIndex()==1 then
				inventoryPanel.SortList.CurrentValue:SetText(Resource[language][17])
			end
			inventoryPanel.SortList.ListData:GetItem(2):SetText(Resource[language][18])
			if inventoryPanel.SortList:GetSelectedIndex()==2 then
				inventoryPanel.SortList.CurrentValue:SetText(Resource[language][18])
			end
			inventoryPanel.SortList.ListData:GetItem(3):SetText(Resource[language][19])
			if inventoryPanel.SortList:GetSelectedIndex()==3 then
				inventoryPanel.SortList.CurrentValue:SetText(Resource[language][19])
			end
			inventoryPanel.SortList.ListData:GetItem(4):SetText(Resource[language][20])
			if inventoryPanel.SortList:GetSelectedIndex()==4 then
				inventoryPanel.SortList.CurrentValue:SetText(Resource[language][20])
			end
			inventoryPanel.DisplayCaption:SetText(Resource[language][22]..":")

			-- filter list
			local tmpFilter=inventoryPanel.FilterList:GetValue()
			local newIndex=1
			inventoryPanel.FilterList:ClearList()
			inventoryPanel.FilterList:AddItem(Resource[language][12],-1)
			for tmpIndex=1,#ItemCategory do
				if ItemCategory[tmpIndex][1]==Turbine.Gameplay.ItemCategory.Undefined then
					undefinedCategoryIndex=tmpIndex+1; -- need to keep track of where the "undefined" category winds up in the list
				end
				categorySortOrder[ItemCategory[tmpIndex][1]]=tmpIndex
				if ItemCategory[tmpIndex][1]==tmpFilter then newIndex=tmpIndex+1 end
				inventoryPanel.FilterList:AddItem(ItemCategory[tmpIndex][2][language],ItemCategory[tmpIndex][1])
			end
			inventoryPanel.FilterList:SetSelectedIndex(newIndex)
			minimalWindow.CharText:SetText(Resource[language][13]..": "..inventoryWindow.CharList:GetText())
			minimalWindow.OptionsButton:SetText(Resource[language][48])
			updateDisplayTabXref()
			inventoryPanel:Refresh()
		end

		-- trimColor
		Settings.trimColor=setupWindow.TrimColor:GetColor()
		setupWindow.LanguageList:SetBorderColor(Settings.trimColor)
		inventoryPanel.FilterList:SetBorderColor(Settings.trimColor)
		inventoryPanel.SortList:SetBorderColor(Settings.trimColor)

		-- backColor
		Settings.backColor=setupWindow.BackColor:GetColor()
		setupWindow.LanguageList:SetBackColor(Settings.backColor)
		setupWindow.LanguageList:SetCurrentBackColor(Settings.backColor)
		inventoryPanel.FilterList:SetBackColor(Settings.backColor)
		inventoryPanel.FilterList:SetCurrentBackColor(Settings.backColor)
		inventoryPanel.SortList:SetBackColor(Settings.backColor)
		inventoryPanel.SortList:SetCurrentBackColor(Settings.backColor)

		-- fontColor
		Settings.fontColor=setupWindow.TextColor:GetColor()
		setupWindow.LanguageCaption:SetForeColor(Settings.fontColor)
		setupWindow.TrimCaption:SetForeColor(Settings.fontColor)
		setupWindow.BackCaption:SetForeColor(Settings.fontColor)
		setupWindow.TextCaption:SetForeColor(Settings.fontColor)
		setupWindow.HeadingsCaption:SetForeColor(Settings.fontColor)
		setupWindow.ListCaption:SetForeColor(Settings.fontColor)
		setupWindow.PanelCaption:SetForeColor(Settings.fontColor)
		setupWindow.replaceBags:SetForeColor(Settings.fontColor)
		setupWindow.loadMinimized:SetForeColor(Settings.fontColor)
		setupWindow.showIconCaption:SetForeColor(Settings.fontColor)
--		setupWindow.showIcon:SetForeColor(Settings.fontColor)
		setupWindow.iconLeftCaption:SetForeColor(Settings.fontColor)
		setupWindow.iconTopCaption:SetForeColor(Settings.fontColor)
		setupWindow.useMinimalHeader:SetForeColor(Settings.fontColor)
-- current bags display total
		setupWindow.totalsOnly:SetForeColor(Settings.fontColor)
		setupWindow.miniIcon:SetForeColor(Settings.fontColor)
-- always load with all
		setupWindow.defaultToAll:SetForeColor(Settings.fontColor)

		setupWindow.bagSeparator:SetForeColor(Settings.fontColor)
		setupWindow.ZoomCaption:SetForeColor(Settings.fontColor)
		inventoryWindow.CharCaption:SetForeColor(Settings.fontColor)
		inventoryPanel.FilterCaption:SetForeColor(Settings.fontColor)
		inventoryPanel.SearchCaption:SetForeColor(Settings.fontColor)
		inventoryPanel.SearchText:SetForeColor(Settings.fontColor)
		inventoryPanel.SortCaption:SetForeColor(Settings.fontColor)
		inventoryPanel.DisplayCaption:SetForeColor(Settings.fontColor)

		-- headingsColor
		Settings.headingsColor=setupWindow.HeadingsColor:GetColor()

		-- listTextColor
		Settings.listTextColor=setupWindow.ListColor:GetColor()
		setupWindow.LanguageList:SetTextColor(Settings.listTextColor)
		inventoryPanel.FilterList:SetTextColor(Settings.listTextColor)
		inventoryPanel.SortList:SetTextColor(Settings.listTextColor)

		-- panelBackColor
		Settings.panelBackColor=setupWindow.PanelColor:GetColor()
		inventoryWindow.TopPanelTiled:SetBackColor(Settings.panelBackColor)
		inventoryWindow.TopPanelLeft:SetBackColor(Settings.panelBackColor)
		inventoryWindow.TopPanelRight:SetBackColor(Settings.panelBackColor)
		inventoryWindow.BottomPanelTiled:SetBackColor(Settings.panelBackColor)
		inventoryWindow.BottomPanelLeft:SetBackColor(Settings.panelBackColor)
		inventoryWindow.BottomPanelRight:SetBackColor(Settings.panelBackColor)

		itemExplorer:SetFont(Settings.fontFace) -- will also handle changes to language as the text has to be reapplied when the font changes
		itemExplorer:SetTrimColor(Settings.trimColor)
		itemExplorer:SetBackColor(Settings.backColor)
		itemExplorer:SetFontColor(Settings.fontColor)
		itemExplorer:SetHeadingsColor(Settings.headingsColor)
		itemExplorer:SetListTextColor(Settings.listTextColor)
		itemExplorer:SetPanelBackColor(Settings.panelBackColor)
		itemExplorer:Layout()

		Settings.zoom=setupWindow.Zoom:GetValue()

		Settings.replaceBags=setupWindow.replaceBags:IsChecked()
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack1, not Settings.replaceBags )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack2, not Settings.replaceBags )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack3, not Settings.replaceBags )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack4, not Settings.replaceBags )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack5, not Settings.replaceBags )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack6, not Settings.replaceBags )

		Settings.totalsOnly=setupWindow.totalsOnly:IsChecked()

		Settings.loadMinimized=setupWindow.loadMinimized:IsChecked()
		Settings.showIcon=setupWindow.showIcon:GetValue()
		if Settings.showIcon==2 then
			iconWindow:SetVisible(true)
		elseif Settings.showIcon==0 then
			iconWindow:SetVisible(false)
		else
			iconWindow:SetVisible(not(inventoryWindow:IsVisible() or minimalWindow:IsVisible()))
		end
		
		local val=tonumber(setupWindow.iconLeft:GetText())
		if val~=nil then
			val=math.floor(val)
			if val<0 then val=0 end
			if val>displayWidth-32 then
				val=displayWidth-32
			end
			iconWindow:SetLeft(val)
		end
		val=tonumber(setupWindow.iconTop:GetText())
		if val~=nil then
			val=math.floor(val)
			if val<0 then val=0 end
			if val>displayHeight-32 then
				val=displayHeight-32
			end
			iconWindow:SetTop(val)
		end
		-- preserve the old size so that we can set whichever window is now visible to that size
		local oldWidth,oldHeight
		if Settings.useMinimalHeader then
			oldWidth,oldHeight=minimalWindow:GetSize()
		else
			oldWidth,oldHeight=inventoryWindow:GetSize()
		end
		Settings.useMinimalHeader=setupWindow.useMinimalHeader:IsChecked()
		if Settings.useMinimalHeader then
			inventoryPanel:SetParent(minimalWindow)
			inventoryPanel:SetPosition(minimalWindow.LeftMargin,minimalWindow.TopMargin)
			equipmentPanel:SetParent(minimalWindow)
			equipmentPanel:SetPosition(minimalWindow.LeftMargin,minimalWindow.TopMargin)
			allEIPanel:SetParent(minimalWindow)
			allEIPanel:SetPosition(minimalWindow.LeftMargin,minimalWindow.TopMargin)
			minimalWindow:SetPosition(setupWindow.PanelLeft-minimalWindow.LeftMargin,setupWindow.PanelTop-minimalWindow.TopMargin)
			minimalWindow:SetSize(oldWidth,oldHeight)
			inventoryWindow:SetVisible(false)
			minimalWindow:SetVisible(true)
		else
			inventoryPanel:SetParent(inventoryWindow)
			inventoryPanel:SetPosition(inventoryWindow.LeftMargin,inventoryWindow.TopMargin)
			equipmentPanel:SetParent(inventoryWindow)
			equipmentPanel:SetPosition(inventoryWindow.LeftMargin,inventoryWindow.TopMargin)
			allEIPanel:SetParent(inventoryWindow)
			allEIPanel:SetPosition(inventoryWindow.LeftMargin,inventoryWindow.TopMargin)
			inventoryWindow:SetPosition(setupWindow.PanelLeft-inventoryWindow.LeftMargin,setupWindow.PanelTop-inventoryWindow.TopMargin)
			inventoryWindow:SetSize(oldWidth,oldHeight)
			if inventoryWindow:GetLeft()<0 then inventoryWindow:SetLeft(0) end
			if inventoryWindow:GetTop()<0 then inventoryWindow:SetTop(0) end
			if inventoryWindow:GetHeight()>displayHeight then inventoryWindow:SetHeight(displayHeight) end
			if inventoryWindow:GetWidth()>displayWidth then inventoryWindow:SetWidth(displayWidth) end
			inventoryWindow:SetVisible(true)
			minimalWindow:SetVisible(false)
		end

		Settings.useMiniIcon=setupWindow.miniIcon:IsChecked()
		if Settings.useMiniIcon then
			iconWindow.UpImage=resourcePath.."mini_icon.tga"
			iconWindow.HoverImage=resourcePath.."mini_hover.tga"
			iconWindow.MoveImage=resourcePath.."mini_drag.tga"
			iconWindow.FlashImage=resourcePath.."mini_border.tga"
		else
			iconWindow.UpImage=resourcePath.."bags_up.jpg"
			iconWindow.HoverImage=resourcePath.."bags_down.jpg"
			iconWindow.MoveImage=resourcePath.."bags_drag.jpg"
			iconWindow.FlashImage=resourcePath.."bags_border.tga"
		end
		iconWindow:SetBackground(iconWindow.UpImage)
		Settings.defaultToAllView=setupWindow.defaultToAll:IsChecked()
		if Settings.bagSeparator~=setupWindow.bagSeparator:IsChecked() then
			Settings.bagSeparator=setupWindow.bagSeparator:IsChecked()
			-- refresh view
		end
		local tmpNum=tonumber(setupWindow.cropDelay:GetText())
		if tmpNum~=nil then Settings.cropDelay=tmpNum end

		updateDisplayTabXref()
		getItemEntryLayout()
		applyItemEntryLayout()
		inventoryPanel:Layout()
		equipmentPanel:Refresh()
		allEIPanel:Refresh()
		setupWindow:SetVisible(false)
		inventoryPanel.cropDelay=Settings.cropDelay
		inventoryPanel:SetWantsUpdates(true)
	end
	setupWindow.Cancel=Turbine.UI.Lotro.Button()
	setupWindow.Cancel:SetParent(setupWindow)
	setupWindow.Cancel:SetFont(Turbine.UI.Lotro.Font.Verdana18)
	setupWindow.Cancel:SetSize(100,20)
	setupWindow.Cancel:SetText(Resource[language][37])
	setupWindow.Cancel.Click=function()
		setupWindow:Refresh()
		setupWindow:SetVisible(false)
	end

	setupWindow.Refresh=function()
		-- re-layout controls since font may have changed
		local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
		fontMetric:SetFont(Settings.fontFace)
		local tmpText={28,28,30,31,33,34,35,55} -- all captions
		local captionWidth=0
		local controlHeight=fontSize
		-- we have controls that can not go below 20 pixel height
		if controlHeight<20 then controlHeight=20 end
		for k,v in ipairs(tmpText) do
			local width=fontMetric:GetTextWidth("Resource[language][v]:")
			if width>captionWidth then captionWidth=width end
		end
		if captionWidth>displayWidth/4 then captionWidth=displayWidth/4 end -- gotta have some sort of reasonable limit
	
		tmpText={24,25,26}
		local tabWidth=0
		for k,v in ipairs(tmpText) do
			local width=fontMetric:GetTextWidth("Resource[language][v]:")+6
			if width>tabWidth then tabWidth=width end
		end

		local initHeight=150+fontSize+10*(controlHeight+5) -- 10 being the number of option rows per tab + 1 for tab controls
		local initWidth=captionWidth+450
		if initWidth<tabWidth*3+34 then initWidth=tabWidth*3+34 end
		if initHeight>displayHeight then initHeight=displayHeight end
		if initHeight+Settings.SWTop>displayHeight then Settings.SWTop=displayHeight-initHeight end
		if initWidth>displayWidth then initWidth=displayWidth end
		if initWidth+Settings.SWLeft>displayWidth then Settings.SWLeft=displayWidth-initWidth end

		setupWindow:SetPosition( Settings.SWLeft, Settings.SWTop )
		setupWindow:SetHeight(initHeight)
		setupWindow:SetWidth(initWidth)
		setupWindow.TabTrim:SetSize(setupWindow:GetWidth()-20,setupWindow:GetHeight()-fontSize-85)
		setupWindow.TabTrim:SetPosition(10,fontSize+46)
		setupWindow.TabBack:SetSize(setupWindow.TabTrim:GetWidth()-2,setupWindow.TabTrim:GetHeight()-2)
		setupWindow.Tab1Tab:SetSize(setupWindow.TabBack:GetWidth()-2,setupWindow.TabBack:GetHeight()-2)
		setupWindow.Tab2Tab:SetSize(setupWindow.TabBack:GetWidth()-2,setupWindow.TabBack:GetHeight()-2)
		setupWindow.Tab3Tab:SetSize(setupWindow.TabBack:GetWidth()-2,setupWindow.TabBack:GetHeight()-2)
		setupWindow.Tab1Button:SetSize(tabWidth,fontSize+6)
		setupWindow.Tab1Top:SetSize(setupWindow.Tab1Button:GetWidth()-10,4)
		setupWindow.Tab1RightCorner:SetPosition(setupWindow.Tab1Button:GetWidth()-5,0)
		setupWindow.Tab1LeftSide:SetSize(3,fontSize+6)
		setupWindow.Tab1Center:SetSize(setupWindow.Tab1Button:GetWidth()-6,fontSize+6)
		setupWindow.Tab1RightSide:SetSize(3,fontSize+6)
		setupWindow.Tab1RightSide:SetPosition(setupWindow.Tab1Button:GetWidth()-3,4)
		setupWindow.Tab1Text:SetSize(setupWindow.Tab1Button:GetWidth()-6,fontSize+1)
		setupWindow.Tab1Text:SetFont(Settings.fontFace)
		setupWindow.Tab1Text:SetText(Resource[language][24])
		setupWindow.Tab2Button:SetPosition(setupWindow.Tab1Button:GetLeft()+tabWidth+5,46)
		setupWindow.Tab2Button:SetSize(tabWidth,fontSize+6)
		setupWindow.Tab2Top:SetSize(setupWindow.Tab2Button:GetWidth()-10,4)
		setupWindow.Tab2RightCorner:SetPosition(setupWindow.Tab2Button:GetWidth()-5,0)
		setupWindow.Tab2LeftSide:SetSize(3,fontSize+6)
		setupWindow.Tab2Center:SetSize(setupWindow.Tab2Button:GetWidth()-6,fontSize+6)
		setupWindow.Tab2RightSide:SetSize(3,fontSize+6)
		setupWindow.Tab2RightSide:SetPosition(setupWindow.Tab2Button:GetWidth()-3,4)
		setupWindow.Tab2Text:SetSize(setupWindow.Tab2Button:GetWidth()-6,fontSize+1)
		setupWindow.Tab2Text:SetFont(Settings.fontFace)
		setupWindow.Tab2Text:SetText(Resource[language][25])
		setupWindow.Tab3Button:SetPosition(setupWindow.Tab2Button:GetLeft()+tabWidth+5,46)
		setupWindow.Tab3Button:SetSize(tabWidth,fontSize+6)
		setupWindow.Tab3Top:SetSize(setupWindow.Tab3Button:GetWidth()-10,4)
		setupWindow.Tab3RightCorner:SetPosition(setupWindow.Tab3Button:GetWidth()-5,0)
		setupWindow.Tab3LeftSide:SetSize(3,fontSize+6)
		setupWindow.Tab3Center:SetSize(setupWindow.Tab3Button:GetWidth()-6,fontSize+6)
		setupWindow.Tab3RightSide:SetSize(3,fontSize+6)
		setupWindow.Tab3RightSide:SetPosition(setupWindow.Tab3Button:GetWidth()-3,4)
		setupWindow.Tab3Text:SetSize(setupWindow.Tab3Button:GetWidth()-6,fontSize+1)
		setupWindow.Tab3Text:SetFont(Settings.fontFace)
		setupWindow.Tab3Text:SetText(Resource[language][26])
		setupWindow.Tab1Button.MouseClick()
		setupWindow.LanguageCaption:SetSize(captionWidth,controlHeight)
		setupWindow.LanguageCaption:SetFont(Settings.fontFace)
		setupWindow.LanguageCaption:SetText(Resource[language][28]..":")
		setupWindow.LanguageList:SetPosition(setupWindow.LanguageCaption:GetLeft()+setupWindow.LanguageCaption:GetWidth()+5,setupWindow.LanguageCaption:GetTop())
		setupWindow.LanguageList:SetSize(setupWindow.Tab1Tab:GetWidth()-5-setupWindow.LanguageList:GetLeft(),fontSize)
		setupWindow.LanguageList:SetFont(Settings.fontFace)
		setupWindow.TrimCaption:SetSize(setupWindow.LanguageCaption:GetSize())
		setupWindow.TrimCaption:SetPosition(setupWindow.LanguageCaption:GetLeft(),setupWindow.LanguageCaption:GetTop()+setupWindow.LanguageCaption:GetHeight()+10)
		setupWindow.TrimCaption:SetFont(Settings.fontFace)
		setupWindow.TrimCaption:SetText(Resource[language][29]..":")
		if controlHeight>20 then
			setupWindow.TrimColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.TrimCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.TrimColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.TrimCaption:GetTop())
		end
		setupWindow.BackCaption:SetSize(captionWidth,controlHeight)
		setupWindow.BackCaption:SetPosition(setupWindow.LanguageCaption:GetLeft(),setupWindow.TrimCaption:GetTop()+setupWindow.TrimCaption:GetHeight()+10)
		setupWindow.BackCaption:SetFont(Settings.fontFace)
		setupWindow.BackCaption:SetText(Resource[language][30]..":")
		if controlHeight>20 then
			setupWindow.BackColor:SetPosition(setupWindow.TrimColor:GetLeft(),setupWindow.BackCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.BackColor:SetPosition(setupWindow.TrimColor:GetLeft(),setupWindow.BackCaption:GetTop())
		end
		setupWindow.TextCaption:SetSize(captionWidth,controlHeight)
		setupWindow.TextCaption:SetPosition(setupWindow.LanguageCaption:GetLeft(),setupWindow.BackCaption:GetTop()+setupWindow.TextCaption:GetHeight()+10)
		setupWindow.TextCaption:SetFont(Settings.fontFace)
		setupWindow.TextCaption:SetText(Resource[language][31]..":")
		if controlHeight>20 then
			setupWindow.TextColor:SetPosition(setupWindow.TrimColor:GetLeft(),setupWindow.TextCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.TextColor:SetPosition(setupWindow.TrimColor:GetLeft(),setupWindow.TextCaption:GetTop())
		end
		setupWindow.HeadingsCaption:SetSize(captionWidth,controlHeight)
		setupWindow.HeadingsCaption:SetPosition(setupWindow.LanguageCaption:GetLeft(),setupWindow.TextCaption:GetTop()+setupWindow.LanguageCaption:GetHeight()+10)
		setupWindow.HeadingsCaption:SetFont(Settings.fontFace)
		setupWindow.HeadingsCaption:SetText(Resource[language][33]..":")
		if controlHeight>20 then
			setupWindow.HeadingsColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.HeadingsCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.HeadingsColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.HeadingsCaption:GetTop())
		end
		setupWindow.ListCaption:SetSize(captionWidth,controlHeight)
		setupWindow.ListCaption:SetPosition(setupWindow.LanguageCaption:GetLeft(),setupWindow.HeadingsCaption:GetTop()+setupWindow.LanguageCaption:GetHeight()+10)
		setupWindow.ListCaption:SetFont(Settings.fontFace)
		setupWindow.ListCaption:SetText(Resource[language][34]..":")
		if controlHeight>20 then
			setupWindow.ListColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.ListCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.ListColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.ListCaption:GetTop())
		end
		setupWindow.PanelCaption:SetSize(captionWidth,controlHeight)
		setupWindow.PanelCaption:SetPosition(setupWindow.LanguageCaption:GetLeft(),setupWindow.ListCaption:GetTop()+setupWindow.LanguageCaption:GetHeight()+10)
		setupWindow.PanelCaption:SetFont(Settings.fontFace)
		setupWindow.PanelCaption:SetText(Resource[language][35]..":")
		if controlHeight>20 then
			setupWindow.PanelColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.PanelCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.PanelColor:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.PanelCaption:GetTop())
		end
		setupWindow.DefaultColors:SetPosition((setupWindow.Tab1Tab:GetWidth()-setupWindow.DefaultColors:GetWidth())/2,setupWindow.PanelCaption:GetTop()+setupWindow.PanelCaption:GetHeight()+10)
		setupWindow.DefaultColors:SetText(Resource[language][32])
		setupWindow.ZoomCaption:SetSize(setupWindow.LanguageCaption:GetSize())
		setupWindow.ZoomCaption:SetPosition(setupWindow.BackCaption:GetLeft(),setupWindow.DefaultColors:GetTop()+setupWindow.DefaultColors:GetHeight()+10)
		setupWindow.ZoomCaption:SetFont(Settings.fontFace)
		setupWindow.ZoomCaption:SetText(Resource[language][55]..":")
		setupWindow.Zoom:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.ZoomCaption:GetTop())
		setupWindow.Zoom:SetSize(setupWindow.Tab1Tab:GetWidth()-5-setupWindow.Zoom:GetLeft(),controlHeight)
		setupWindow.Zoom:SetFont(Settings.fontFace)

		setupWindow.FontCaption:SetSize(setupWindow.LanguageCaption:GetSize())
		setupWindow.FontCaption:SetPosition(setupWindow.BackCaption:GetLeft(),setupWindow.ZoomCaption:GetTop()+setupWindow.ZoomCaption:GetHeight()+10)
		setupWindow.FontCaption:SetFont(Settings.fontFace)
		setupWindow.FontCaption:SetText(Resource[language][131]..":")
		if controlHeight>20 then
			setupWindow.FontSelect:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.FontCaption:GetTop()+(controlHeight-20)/2)
		else
			setupWindow.FontSelect:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.FontCaption:GetTop())
		end
		setupWindow.FontSelect:SetFont(Settings.fontFace)
		-- force font list to be hidden in case is was previously displayed
		FontSelectList:SetVisible(false)

		setupWindow.Save:SetPosition((setupWindow:GetWidth()-setupWindow.Save:GetWidth()*2)/3, setupWindow:GetHeight()-35)
		setupWindow.Save:SetText(Resource[language][36])
		setupWindow.Cancel:SetPosition(setupWindow.Save:GetWidth()+setupWindow.Save:GetLeft()*2, setupWindow.Save:GetTop())
		setupWindow.Cancel:SetText(Resource[language][37])
		setupWindow.replaceBags:SetSize(setupWindow.Tab2Tab:GetWidth()-20,controlHeight)
		setupWindow.replaceBags:SetFont(Settings.fontFace)
		setupWindow.replaceBags:SetText(Resource[language][38])
		setupWindow.totalsOnly:SetSize(setupWindow.replaceBags:GetWidth(),controlHeight)
		setupWindow.totalsOnly:SetPosition(setupWindow.replaceBags:GetLeft(),setupWindow.replaceBags:GetTop()+setupWindow.replaceBags:GetHeight()+10)
		setupWindow.totalsOnly:SetFont(Settings.fontFace)
		setupWindow.totalsOnly:SetText(Resource[language][53])
		setupWindow.loadMinimized:SetSize(setupWindow.replaceBags:GetWidth(),controlHeight)
		setupWindow.loadMinimized:SetPosition(setupWindow.replaceBags:GetLeft(),setupWindow.totalsOnly:GetTop()+setupWindow.totalsOnly:GetHeight()+10)
		setupWindow.loadMinimized:SetFont(Settings.fontFace)
		setupWindow.loadMinimized:SetText(Resource[language][39])

		setupWindow.showIconCaption:SetSize(setupWindow.LanguageCaption:GetSize())
		setupWindow.showIconCaption:SetPosition(setupWindow.BackCaption:GetLeft(),setupWindow.loadMinimized:GetTop()+setupWindow.loadMinimized:GetHeight()+10)
		setupWindow.showIconCaption:SetFont(Settings.fontFace)
		setupWindow.showIconCaption:SetText(Resource[language][40]..":")
		setupWindow.showIcon:SetPosition(setupWindow.BackCaption:GetLeft(),setupWindow.showIconCaption:GetTop()+setupWindow.showIconCaption:GetHeight()+5)
		setupWindow.showIcon:SetSize(setupWindow.Tab2Tab:GetWidth()-5-setupWindow.showIcon:GetLeft(),controlHeight)
		setupWindow.showIcon:SetFont(Settings.fontFace)

		setupWindow.iconLeftCaption:SetSize(setupWindow.Tab2Tab:GetWidth()/4-10,controlHeight)
		setupWindow.iconLeftCaption:SetPosition(10,setupWindow.showIcon:GetTop()+setupWindow.showIcon:GetHeight()+10)
		setupWindow.iconLeftCaption:SetFont(Settings.fontFace)
		setupWindow.iconLeftCaption:SetText(Resource[language][41]..":")
		setupWindow.iconLeftBack:SetSize(setupWindow.Tab2Tab:GetWidth()/4-10,fontSize+2)
		setupWindow.iconLeftBack:SetPosition(setupWindow.Tab2Tab:GetWidth()/4+5,setupWindow.iconLeftCaption:GetTop()-1)
		setupWindow.iconLeft:SetSize(setupWindow.Tab2Tab:GetWidth()/4-12,fontSize)
		setupWindow.iconLeft:SetFont(Settings.fontFace)
		setupWindow.iconLeft:SetText(setupWindow.iconLeft:GetText())
		setupWindow.iconTopCaption:SetSize(setupWindow.iconLeftCaption:GetWidth(),controlHeight)
		setupWindow.iconTopCaption:SetPosition(setupWindow.Tab2Tab:GetWidth()/2+5,setupWindow.iconLeftCaption:GetTop())
		setupWindow.iconTopCaption:SetFont(Settings.fontFace)
		setupWindow.iconTopCaption:SetText(Resource[language][42]..":")
		setupWindow.iconTopBack:SetSize(setupWindow.iconLeftBack:GetWidth(),fontSize+2)
		setupWindow.iconTopBack:SetPosition(setupWindow.Tab2Tab:GetWidth()*3/4,setupWindow.iconTopCaption:GetTop()-1)
		setupWindow.iconTop:SetSize(setupWindow.iconLeft:GetWidth(),fontSize)
		setupWindow.iconTop:SetFont(Settings.fontFace)
		setupWindow.iconTop:SetText(setupWindow.iconTop:GetText())
		setupWindow.miniIcon:SetSize(setupWindow.replaceBags:GetWidth(),controlHeight)
		setupWindow.miniIcon:SetPosition(setupWindow.loadMinimized:GetLeft(),setupWindow.iconTopCaption:GetTop()+setupWindow.iconTopCaption:GetHeight()+10)
		setupWindow.miniIcon:SetFont(Settings.fontFace)
		setupWindow.miniIcon:SetText(Resource[language][52])
		setupWindow.useMinimalHeader:SetSize(setupWindow.replaceBags:GetWidth(),controlHeight)
		setupWindow.useMinimalHeader:SetPosition(setupWindow.replaceBags:GetLeft(),setupWindow.miniIcon:GetTop()+setupWindow.miniIcon:GetHeight()+10)
		setupWindow.useMinimalHeader:SetFont(Settings.fontFace)
		setupWindow.useMinimalHeader:SetText(Resource[language][43])
		setupWindow.defaultToAll:SetSize(setupWindow.replaceBags:GetWidth(),controlHeight)
		setupWindow.defaultToAll:SetPosition(setupWindow.replaceBags:GetLeft(),setupWindow.useMinimalHeader:GetTop()+setupWindow.useMinimalHeader:GetHeight()+10)
		setupWindow.defaultToAll:SetFont(Settings.fontFace)
		setupWindow.defaultToAll:SetText(Resource[language][54])
	setupWindow.Tab3ViewPane:SetWidth(setupWindow.Tab3Tab:GetWidth()-10)
	setupWindow.Tab3VScroll:SetLeft(setupWindow.Tab3Tab:GetWidth()-10)
	setupWindow.Tab3VScroll:SetHeight(setupWindow.Tab3Tab:GetHeight())
	setupWindow.Tab3VScroll:SetValue(0) -- reset scroll to 0
	setupWindow.Tab3ViewPane:SetTop(0)

	local descWidth=setupWindow.Tab3ViewPane:GetWidth()-setupWindow.delayCaption:GetLeft()*2
	setupWindow.delayCaption:SetSize(descWidth,fontMetric:GetTextHeight(Resource[language][134],descWidth))
	setupWindow.delayCaption:SetFont(Settings.fontFace)
	setupWindow.delayCaption:SetText(Resource[language][134])
	setupWindow.sep1:SetTop(setupWindow.delayCaption:GetTop()+setupWindow.delayCaption:GetHeight()+4)
	setupWindow.sep1:SetWidth(descWidth)
	setupWindow.sep1:SetBackColor(Settings.trimColor)

	setupWindow.cropDelayCaption:SetPosition(setupWindow.BackCaption:GetLeft(),setupWindow.sep1:GetTop()+setupWindow.sep1:GetHeight()+5)
	setupWindow.cropDelayCaption:SetSize(setupWindow.BackCaption:GetSize())
	setupWindow.cropDelayCaption:SetFont(Settings.fontFace)
	setupWindow.cropDelayCaption:SetText(setupWindow.cropDelayCaption:GetText())
	setupWindow.cropDelayBack:SetPosition(setupWindow.LanguageList:GetLeft(),setupWindow.cropDelayCaption:GetTop()-1)
	setupWindow.cropDelayBack:SetSize(200,fontSize+2)
	setupWindow.cropDelay:SetSize(198,fontSize)
	setupWindow.cropDelay:SetFont(Settings.fontFace)
	setupWindow.cropDelay:SetText(Settings.cropDelay)
	setupWindow.cropDelayDesc:SetTop(setupWindow.cropDelayCaption:GetTop()+setupWindow.cropDelayCaption:GetHeight()+10)
	setupWindow.cropDelayDesc:SetSize(descWidth,fontMetric:GetTextHeight(Resource[language][133],descWidth))
	setupWindow.cropDelayDesc:SetFont(Settings.fontFace)
	setupWindow.cropDelayDesc:SetText(Resource[language][133])

	setupWindow.sep2:SetTop(setupWindow.cropDelayDesc:GetTop()+setupWindow.cropDelayDesc:GetHeight()+4)
	setupWindow.sep2:SetWidth(descWidth)
	setupWindow.sep2:SetBackColor(Settings.trimColor)

	setupWindow.Tab3ViewPane:SetHeight(setupWindow.sep2:GetTop()+setupWindow.sep2:GetHeight()+5)
	if setupWindow.Tab3ViewPane:GetHeight()>setupWindow.Tab3Tab:GetHeight() then
		setupWindow.Tab3VScroll:SetMaximum(setupWindow.Tab3ViewPane:GetHeight()-setupWindow.Tab3Tab:GetHeight())
		setupWindow.Tab3VScroll:SetVisible(true)
	else
		setupWindow.Tab3VScroll:SetVisible(false)
	end

		-- refresh current settings
		setupWindow.LanguageList:SetSelectedIndex(Settings.language+1)
		setupWindow.TrimColor:SetColor(Settings.trimColor)
		setupWindow.BackColor:SetColor(Settings.backColor)
		setupWindow.TextColor:SetColor(Settings.fontColor)
		setupWindow.HeadingsColor:SetColor(Settings.headingsColor)
		setupWindow.ListColor:SetColor(Settings.listTextColor)
		setupWindow.PanelColor:SetColor(Settings.panelBackColor)
		setupWindow.replaceBags:SetChecked(Settings.replaceBags)
		setupWindow.totalsOnly:SetChecked(Settings.totalsOnly)
		setupWindow.loadMinimized:SetChecked(Settings.loadMinimized)
		setupWindow.iconLeft:SetText(iconWindow:GetLeft())
		setupWindow.iconTop:SetText(iconWindow:GetTop())
		setupWindow.useMinimalHeader:SetChecked(Settings.useMinimalHeader)
		setupWindow.miniIcon:SetChecked(Settings.useMiniIcon)
		setupWindow.defaultToAll:SetChecked(Settings.defaultToAllView)
		setupWindow.bagSeparator:SetChecked(Settings.bagSeparator)
		if Settings.useMinimalHeader then
			setupWindow.PanelTop=minimalWindow:GetTop()+inventoryPanel:GetTop()
			setupWindow.PanelLeft=minimalWindow:GetLeft()+inventoryPanel:GetLeft()
		else
			setupWindow.PanelTop=inventoryWindow:GetTop()+inventoryPanel:GetTop()
			setupWindow.PanelLeft=inventoryWindow:GetLeft()+inventoryPanel:GetLeft()
		end
		if Settings.zoom==4 then
			setupWindow.Zoom:SetSelectedChoice(3)
		elseif Settings.zoom==2 then
			setupWindow.Zoom:SetSelectedChoice(2)
		else
			setupWindow.Zoom:SetSelectedChoice(1)
		end
		if Settings.showIcon==0 then
			setupWindow.showIcon:SetSelectedChoice(1)
		elseif Settings.showIcon==2 then
			setupWindow.showIcon:SetSelectedChoice(3)
		else
			setupWindow.showIcon:SetSelectedChoice(2)
		end
	end
end
setupWindow.Refresh() -- call refresh to finish the setup

setupWindow:SetText(Resource[language][27])

setupWindow.KeyUp=function(sender, args)
	if bShowKeyDown==true then
		Turbine.Shell.WriteLine("KeyUp Action="..args.Action..", .Alt="..tostring(args.Alt)..", .Control="..tostring(args.Control)..", .Shift="..tostring(args.Shift))
	end
end

setupWindow.KeyDown=function(sender, args)
	if bShowKeyDown==true then
		Turbine.Shell.WriteLine("KeyDown Action="..args.Action..", .Alt="..tostring(args.Alt)..", .Control="..tostring(args.Control)..", .Shift="..tostring(args.Shift))
	end
	if ( args.Action == Turbine.UI.Lotro.Action.Escape ) then
		setupWindow:SetVisible( false )
		inventoryWindow:SetVisible(false)
		minimalWindow:SetVisible(false)
		groupMaint:SetVisible(false)
		itemExplorer:SetVisible(false)
		itemInfoDetail:SetVisible(false)
		iconWindow:SetVisible(Settings.showIcon==1 or Settings.showIcon==2)
		hudVisible=true
	end
	if ( args.Action == 268435579 ) then -- reposition UI
		moveToggle=not moveToggle
	end
	if ( args.Action == 268435635 ) then -- toggle HUD
		hudVisible=not hudVisible
		if hudVisible then
			setupWindow:SetVisible(setupVisible)
			inventoryWindow:SetVisible(inventoryVisible)
			minimalWindow:SetVisible(minimalVisible)
			iconWindow:SetVisible((iconVisible and Settings.showIcon==1) or Settings.showIcon==2)
			groupMaint:SetVisible(groupMaintVisible)
			itemExplorer:SetVisible(itemExplorerVisible)
			itemInfoDetail:SetVisible(itemInfoDetailVisible)
			local container=getCurrentContainer()
			for k,v in ipairs(displayTabs[container]) do
				if v.window~=nil and not v.docked then
					v.window:SetVisible(true)
				end
			end
		else
			setupVisible=setupWindow:IsVisible()
			setupWindow:SetVisible(false)
			inventoryVisible=inventoryWindow:IsVisible()
			inventoryWindow:SetVisible(false)
			minimalVisible=minimalWindow:IsVisible()
			minimalWindow:SetVisible(false)
			iconVisible=iconWindow:IsVisible()
			iconWindow:SetVisible(false)
			groupMaintVisible=groupMaint:IsVisible()
			groupMaint:SetVisible(false)
			itemExplorerVisible=itemExplorer:IsVisible()
			itemExplorer:SetVisible(false)
			itemInfoDetailVisible=itemInfoDetail:IsVisible()
			itemInfoDetail:SetVisible(false)
			local container=getCurrentContainer()
			for k,v in ipairs(displayTabs[container]) do
				if v.window~=nil and not v.docked then
					v.window:SetVisible(false)
				end
			end
		end
	end
	if Settings.replaceBags and ( args.Action == Turbine.UI.Lotro.Action.ToggleBags or
		args.Action == Turbine.UI.Lotro.Action.ToggleBag1 or
		args.Action == Turbine.UI.Lotro.Action.ToggleBag2 or
		args.Action == Turbine.UI.Lotro.Action.ToggleBag3 or
		args.Action == Turbine.UI.Lotro.Action.ToggleBag4 or
		args.Action == Turbine.UI.Lotro.Action.ToggleBag5 or
		args.Action == Turbine.UI.Lotro.Action.ToggleBag6 )
	then
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack1, false )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack2, false )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack3, false )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack4, false )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack5, false )
		Turbine.UI.Lotro.LotroUI.SetEnabled( Turbine.UI.Lotro.LotroUIElement.Backpack6, false )
		if Settings.useMinimalHeader then
			minimalWindow:SetVisible(not minimalWindow:IsVisible())
			if (not minimalWindow:IsVisible() and Settings.showIcon==1) or Settings.showIcon==2 then
				iconWindow:SetVisible(true)
			end
		else
			inventoryWindow:SetVisible(not inventoryWindow:IsVisible())
			if (not inventoryWindow:IsVisible() and Settings.showIcon==1) or Settings.showIcon==2 then
				iconWindow:SetVisible(true)
			end
		end
	end
end

setupWindow:SetWantsKeyEvents(true)
setupWindow:SetWantsUpdates(true)

setupWindow.shellCmd=Turbine.ShellCommand()
local numberOfCommandsRegistered = Turbine.Shell.AddCommand("AltInventory",setupWindow.shellCmd)

setupWindow.shellCmd.Execute = function(sender, cmd, args)
	local lcmd=string.lower(cmd)
	local largs=string.lower(args)
	if (lcmd == "altinventory") then
		if (largs=="setup") then
			setupWindow:SetVisible(true)
			setupWindow:Refresh()
		elseif (largs=="show") then
			if Settings.showIcon==1 then
				iconWindow:SetVisible(false)
			end
			if Settings.useMinimalHeader then
				minimalWindow:SetVisible(true)
			else
				inventoryWindow:SetVisible(true)
			end
		elseif (largs=="hide") then
			setupWindow:SetVisible(false)
			inventoryWindow:SetVisible(false)
			minimalWindow:SetVisible(false)
--*** this isn't quite right
			if Settings.showIcon==1 then
				iconWindow:SetVisible(true)
			end
		elseif(largs=="explore") then
			itemExplorer:SetVisible(true)
		elseif (largs=="resetdata") then
			-- ONLY used for debugging
			-- will delete ALL inventories and then load only the current char's bags, vault(if accessible) and shared storage (if accessible)
			accountItemQty={}
			CharList={}
			CharList["Shared Storage"]={}
			CharList[charName]={}
			CharList[charVaultName]={}
			CharList[charEIName]={}
			setupWindow.Loaded=false
			-- clear minimalWindow.CharMenu
			minimalWindow.CharMenu:GetItems():Clear()
			-- clear inventoryWindow.CharList
			inventoryWindow.CharList:ClearList()
			-- repop minimalWindow.CharMenu and inventoryWindow.CharList
			refreshBackPackImage() -- will reset the used/capacity values
--			populateCharLists()
			setupWindow:SetWantsUpdates(true)
		elseif (largs=="toggle") then
			setupWindow:SetVisible(false)
			if inventoryWindow:IsVisible() or minimalWindow:IsVisible() then
				inventoryWindow:SetVisible(false)
				minimalWindow:SetVisible(false)
				if Settings.showIcon==1 or Settings.showIcon==2 then
					iconWindow:SetVisible(true)
				end
			else
				if Settings.showIcon==1 then
					iconWindow:SetVisible(false)
				end
				if Settings.useMinimalHeader then
					minimalWindow:SetVisible(true)
				else
					inventoryWindow:SetVisible(true)
				end
			end

		elseif (largs=="debug on") then
			Settings.debug=true
			debugWindow:SetVisible(true)
			Turbine.Shell.WriteLine("Debug is now: "..tostring(Settings.debug))
		elseif (largs=="debug off") then
			Settings.debug=false
			Turbine.Shell.WriteLine("Debug is now: "..tostring(Settings.debug))
		elseif (largs=="debug toggle") then
			Settings.debug=not Settings.debug
			if debugWindow~=nil then
				debugWindow:SetVisible(Settings.debug)
			end
			Turbine.Shell.WriteLine("Debug is now: "..tostring(Settings.debug))
		elseif (largs=="debug") then
			Turbine.Shell.WriteLine("Debug is: "..tostring(Settings.debug))
		else
			Turbine.Shell.WriteLine("usage:")
			Turbine.Shell.WriteLine("  /AltInventory setup")
			Turbine.Shell.WriteLine("       Displays the setup window.")
			Turbine.Shell.WriteLine("  /AltInventory show")
			Turbine.Shell.WriteLine("       Shows the main window.")
			Turbine.Shell.WriteLine("  /AltInventory hide")
			Turbine.Shell.WriteLine("       Hides the main window.")
			Turbine.Shell.WriteLine("  /AltInventory toggle")
			Turbine.Shell.WriteLine("       Toggles the main window.")
			Turbine.Shell.WriteLine("  /AltInventory resetdata")
			Turbine.Shell.WriteLine("       Clears ALL inventory data for ALL characters.")
			Turbine.Shell.WriteLine("  /AltInventory debug [on|off|toggle]")
			Turbine.Shell.WriteLine("       Turns debug on/off or toggles debug. Displays debug state.")
			Turbine.Shell.WriteLine(" /AltInventory explore")
			Turbine.Shell.WriteLine("       Displays the Item Explorer window")
		end
	end
end
setupWindow.shellCmd.GetHelp = function(sender, cmd)
	return("usage:\n  /AltInventory setup\n       Displays the setup window.\n  /AltInventory show\n       Shows the main window.\n  /AltInventory hide\n       Hides the main window.\n /AltInventory toggle\n       Toggles the main window\n /AltInventory explore\n       Displays the Item Explorer Window")
end

setupWindow.shellCmd.GetShortHelp = function(sender, cmd)
	return("AltInventory - usage: /AltInventory setup | show | hide | toggle | explore")
end

setupWindow.Update=function( sender, args )
	if (Plugins.AltInventory ~= nil) and (not setupWindow.Loaded) then
		setupWindow:SetWantsUpdates(false)
		LoadData()
		updateDisplayTabXref()
		if sharedStorage:IsAvailable() then
			refreshSharedStorage()
		end
		if vault:IsAvailable() then
			refreshVault()
		end
--		if Settings.enableEI then
--			refreshEI()
--		end
		setupWindow.Loaded=true

		Turbine.Shell.WriteLine(Resource[language][2].." "..Plugins.AltInventory:GetVersion().." "..Resource[language][3])
		Plugins.AltInventory.Unload = function(setupWindow,sender,args)
			UnloadPlugin()
		end
		-- attempt to unload the AIReloader plugin in case we were being reloaded
		pcall(Turbine.PluginManager.UnloadScriptState,"AIReloader")

		-- force display refresh to deal with stretchmode oddities during load
		getItemEntryLayout()
		applyItemEntryLayout()
		inventoryPanel:Layout()
		inventoryWindow:SetWantsUpdates(true)
		inventoryPanel.VScroll:SetWantsUpdates(true)
	end
end
