-- need third state, "pressed" for radio button like behavior

-- requires fontMetric to allow auto sizing (can still be manually sized)
-- has a built-in QuickSlot which is hidden between the button and the text but can be enabled and set to a shortcut. enabling the QS will disable the generic Click event

-- need to figure out how to size border. guess that depends on size of default resources? can skins change size of border resource?
-- I think the standard button background is just a left image, center tiled image and right image - if so, we need to somehow crop the upper left, middle left and lower left, same for top/bottom and right
-- only the middle left/right parts hae an issue tiling since they need to tile vertically (top/bottom tile horizontally which is ok the unused part crops nicely)

-- hmmm, button borders seem to available in a number of fixed sizes, 26,38,76
-- size WxH	left normal, mid normal, right normal, left hover, mid hover, right hover
-- 20	20x20	0x410001a9, 0x410001b0, 0x410001b7, 0x410001a5, 0x410001ac, 0x410001b3
-- 26	26x26	0x411699c4, 0x411699c6, 0x411699c8, 0x411699c5, 0x411699c7, 0x411699ca
-- 36	20x36	0x41135b96, 0x41135b99, 0x41135b9e, 0x41135b97, 0x41135b98, 0x41135b92
-- 64	45x64	0x410dcfed, 0x410dcfef, 0x410dcff1, 0x410de1e4, 0x410de1e6, 0x410de1e8
-- 78	39x78	0x4113477f, 0x41134784, 0x41134789, 0x4113477d, 0x41134782, 0x41134787

-- one usable scalable version has corners 45 wide, 14 high, sides 45 wide, 3 high, top/bottom 12 wide, 14 high
-- minimum dimension 45+12+45=102 wide, 14+3+14=31 high (or 45+0+45=90 wide by 14+0+14=28 high if we allow middle sections to go to 0)
-- border is 5 pixels, so, usable area becomes 92x21 (or 80x18) which is workable for all practical font sizes
-- topleft 0x4111dcda
-- top 0x4111dcd9
-- topright 0x4111dcdb
-- right 0x4111dcd8
-- bottomright 0x4111dcd5
-- bottom 0x4111dcd3
-- bottomleft 0x4111dcd4
-- left 0x4111dcd7
-- middlecenter 0x4111dcd6

-- note, there is no 'disabled state' perhaps this could be incorportated by blendmode to grey out the frame/back and tracking the state

-- creates a scalable button with built-in resources for border and area for image or text
ScalableButton = class(Turbine.UI.Control);

function ScalableButton:Constructor(width,height)
	if width==nil then width=90 end
	if height==nil then height=28 end
	if width<90 then width=90 end
	if height<28 then height=28 end

	Turbine.UI.Control.Constructor( self )
	-- min size 90x28
	self.TopLeft=Turbine.UI.Control()
	self.TopLeft:SetParent(self)
	self.TopLeft:SetBackground(0x4111dcda)
	self.TopLeft:SetSize(45,14)
	self.TopLeft:SetPosition(0,0)
	self.TopLeft:SetMouseVisible(false)
	self.TopLeft:SetBackColor(Turbine.UI.Color.DarkGray)
	self.TopLeft:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.TopCenter=Turbine.UI.Control()
	self.TopCenter:SetParent(self)
	self.TopCenter:SetBackground(0x4111dcd9)
	self.TopCenter:SetSize(width-90,14)
	self.TopCenter:SetPosition(45,0)
	self.TopCenter:SetMouseVisible(false)
	self.TopCenter:SetBackColor(Turbine.UI.Color.DarkGray)
	self.TopCenter:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.TopRight=Turbine.UI.Control()
	self.TopRight:SetParent(self)
	self.TopRight:SetBackground(0x4111dcdb)
	self.TopRight:SetSize(45,14)
	self.TopRight:SetPosition(width-45,0)
	self.TopRight:SetMouseVisible(false)
	self.TopRight:SetBackColor(Turbine.UI.Color.DarkGray)
	self.TopRight:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.Right=Turbine.UI.Control()
	self.Right:SetParent(self)
	self.Right:SetBackground(0x4111dcd8)
	self.Right:SetSize(45,height-28)
	self.Right:SetPosition(width-45,14)
	self.Right:SetMouseVisible(false)
	self.Right:SetBackColor(Turbine.UI.Color.DarkGray)
	self.Right:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.BottomRight=Turbine.UI.Control()
	self.BottomRight:SetParent(self)
	self.BottomRight:SetBackground(0x4111dcd5)
	self.BottomRight:SetSize(45,14)
	self.BottomRight:SetPosition(width-45,height-14)
	self.BottomRight:SetMouseVisible(false)
	self.BottomRight:SetBackColor(Turbine.UI.Color.DarkGray)
	self.BottomRight:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.BottomCenter=Turbine.UI.Control()
	self.BottomCenter:SetParent(self)
	self.BottomCenter:SetBackground(0x4111dcd3)
	self.BottomCenter:SetSize(width-90,14)
	self.BottomCenter:SetPosition(45,height-14)
	self.BottomCenter:SetMouseVisible(false)
	self.BottomCenter:SetBackColor(Turbine.UI.Color.DarkGray)
	self.BottomCenter:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.BottomLeft=Turbine.UI.Control()
	self.BottomLeft:SetParent(self)
	self.BottomLeft:SetBackground(0x4111dcd4)
	self.BottomLeft:SetSize(45,14)
	self.BottomLeft:SetPosition(0,height-14)
	self.BottomLeft:SetMouseVisible(false)
	self.BottomLeft:SetBackColor(Turbine.UI.Color.DarkGray)
	self.BottomLeft:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.Left=Turbine.UI.Control()
	self.Left:SetParent(self)
	self.Left:SetBackground(0x4111dcd7)
	self.Left:SetSize(45,height-28)
	self.Left:SetPosition(0,14)
	self.Left:SetMouseVisible(false)
	self.Left:SetBackColor(Turbine.UI.Color.DarkGray)
	self.Left:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)

	self.MiddleCenter=Turbine.UI.Control()
	self.MiddleCenter:SetParent(self)
	self.MiddleCenter:SetBackground(0x4111dcd6)
	self.MiddleCenter:SetSize(width-90,height-28)
	self.MiddleCenter:SetPosition(45,14)
	self.MiddleCenter:SetMouseVisible(false)
	self.MiddleCenter:SetBackColor(Turbine.UI.Color.DarkGray)
	self.MiddleCenter:SetBackColorBlendMode(Turbine.UI.BlendMode.Multiply)
	
	self.QS=Turbine.UI.Lotro.Quickslot()
	self.QS:SetParent(self)
	self.QS:SetPosition(5,5)
	self.QS:SetSize(width-10,height-10)
	self.QS:SetAllowDrop(false)
	self.QS:SetVisible(false) -- not enabled by default
	self.QSAllowDrop=true
	self.QS.Update=function()
		if not self.QSAllowDrop and self.QSData~=nil and self.QSType~=nil then
			-- restore the previous quickslot
			local sc=Turbine.UI.Lotro.Shortcut(self.QSType,self.QSData)
			if self.QSType==Turbine.UI.Lotro.ShortcutType.Alias then
				sc:SetData(self.QSData) -- fixes a bug with utf-8 special chars in aliases
			end
			self.QS:SetShortcut(sc)
		end
		self.QS:SetWantsUpdates(false)
	end

	self.Label=Turbine.UI.Label()
	self.Label:SetParent(self)
--	self.Label:SetPosition(5,5)
	self.Label:SetSize(width,height)
	self.Label:SetFont(Turbine.UI.Lotro.Font.Verdana20) -- default font for now
	self.Label:SetMultiline(false) -- can be overridden when we set text?
	self.Label:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
	self.Label:SetMouseVisible(false)
	self:SetSize(width,height)
end
ScalableButton.SizeChanged=function(self,args)
	local width,height=self:GetSize()
	if width<90 then
		-- enforce min width but don't call sizechanged again
		Turbine.UI.Control.SetWidth(self,90)
		width=90
	end
	if height<28 then
		-- enforce min height but don't call sizechanged again
		Turbine.UI.Control.SetHeight(self,28)
		height=28
	end
	self.TopCenter:SetWidth(width-90)
	self.TopRight:SetLeft(width-45)
	self.Right:SetHeight(height-28)
	self.Right:SetLeft(width-45)
	self.BottomRight:SetPosition(width-45,height-14)
	self.BottomCenter:SetWidth(width-90)
	self.BottomCenter:SetTop(height-14)
	self.BottomLeft:SetTop(height-14)
	self.Left:SetHeight(height-28)
	self.MiddleCenter:SetSize(width-90,height-28)
	self.QS:SetSize(width-10,height-10)
	self.Label:SetSize(width,height)
end
function ScalableButton:MouseEnter(args)
	self.TopLeft:SetBackColor(Turbine.UI.Color.White)
	self.TopCenter:SetBackColor(Turbine.UI.Color.White)
	self.TopRight:SetBackColor(Turbine.UI.Color.White)
	self.Right:SetBackColor(Turbine.UI.Color.White)
	self.BottomRight:SetBackColor(Turbine.UI.Color.White)
	self.BottomCenter:SetBackColor(Turbine.UI.Color.White)
	self.BottomLeft:SetBackColor(Turbine.UI.Color.White)
	self.Left:SetBackColor(Turbine.UI.Color.White)
	self.MiddleCenter:SetBackColor(Turbine.UI.Color.White)
end
function ScalableButton:MouseLeave(args)
	if self.Pressed then
		self.TopLeft:SetBackColor(Turbine.UI.Color.LightGray)
		self.TopCenter:SetBackColor(Turbine.UI.Color.LightGray)
		self.TopRight:SetBackColor(Turbine.UI.Color.LightGray)
		self.Right:SetBackColor(Turbine.UI.Color.LightGray)
		self.BottomRight:SetBackColor(Turbine.UI.Color.LightGray)
		self.BottomCenter:SetBackColor(Turbine.UI.Color.LightGray)
		self.BottomLeft:SetBackColor(Turbine.UI.Color.LightGray)
		self.Left:SetBackColor(Turbine.UI.Color.LightGray)
		self.MiddleCenter:SetBackColor(Turbine.UI.Color.LightGray)
	else
		self.TopLeft:SetBackColor(Turbine.UI.Color.DarkGray)
		self.TopCenter:SetBackColor(Turbine.UI.Color.DarkGray)
		self.TopRight:SetBackColor(Turbine.UI.Color.DarkGray)
		self.Right:SetBackColor(Turbine.UI.Color.DarkGray)
		self.BottomRight:SetBackColor(Turbine.UI.Color.DarkGray)
		self.BottomCenter:SetBackColor(Turbine.UI.Color.DarkGray)
		self.BottomLeft:SetBackColor(Turbine.UI.Color.DarkGray)
		self.Left:SetBackColor(Turbine.UI.Color.DarkGray)
		self.MiddleCenter:SetBackColor(Turbine.UI.Color.DarkGray)
	end
end
function ScalableButton:SetPressed(pressed)
	self.Pressed=pressed
	if self.Pressed then
		self.TopLeft:SetBackColor(Turbine.UI.Color.LightGray)
		self.TopCenter:SetBackColor(Turbine.UI.Color.LightGray)
		self.TopRight:SetBackColor(Turbine.UI.Color.LightGray)
		self.Right:SetBackColor(Turbine.UI.Color.LightGray)
		self.BottomRight:SetBackColor(Turbine.UI.Color.LightGray)
		self.BottomCenter:SetBackColor(Turbine.UI.Color.LightGray)
		self.BottomLeft:SetBackColor(Turbine.UI.Color.LightGray)
		self.Left:SetBackColor(Turbine.UI.Color.LightGray)
		self.MiddleCenter:SetBackColor(Turbine.UI.Color.LightGray)
	else
		self.TopLeft:SetBackColor(Turbine.UI.Color.DarkGray)
		self.TopCenter:SetBackColor(Turbine.UI.Color.DarkGray)
		self.TopRight:SetBackColor(Turbine.UI.Color.DarkGray)
		self.Right:SetBackColor(Turbine.UI.Color.DarkGray)
		self.BottomRight:SetBackColor(Turbine.UI.Color.DarkGray)
		self.BottomCenter:SetBackColor(Turbine.UI.Color.DarkGray)
		self.BottomLeft:SetBackColor(Turbine.UI.Color.DarkGray)
		self.Left:SetBackColor(Turbine.UI.Color.DarkGray)
		self.MiddleCenter:SetBackColor(Turbine.UI.Color.DarkGray)
	end
end
function ScalableButton:SetTextColor( color )
	-- sets the back color of the text/image area
	-- can't set back color of border area (this is controlled by border color)
	self.Label:SetForeColor(color)
end
function ScalableButton:GetTextColor()
	return self.Label:GetForeColor()
end
function ScalableButton:SetMarkupEnabled(enabled)
	self.Label:SetMarkupEnabled(enabled)
end
function ScalableButton:IsMarkupEnabled()
	return self.Label:IsMarkupEnabled()
end
function ScalableButton:GetBackColor()
	return self.BackColor
end
function ScalableButton:SetTextColor( color )
end
function ScalableButton:SetFont( font )
	local size
	if Turbine.UI.Lotro.FontInfo~=nil then
		if Turbine.UI.Lotro.FontInfo[font]~=nil then
			size=Turbine.UI.Lotro.FontInfo[font].size
		end
	end
	if size==nil then
		-- fontInfo data not found - try to get size directly from Turbine enumeration
		getFontSizeFromFont(font)
	end
	if size~=nil and self.AutoSize then
		size=size+10
		if size<28 then size=28 end
		local width=self:GetWidth()
		-- Note, if we don't have a fontMetricInstance, we can only scale height
		if self.FontMetricInstance~=nil then
			self.FontMetricInstance:SetFont(font)
			width=self.FontMetricInstance:GetTextWidth(self.Label:GetText())+10 -- add 10 for borders and margin
			if width<90 then width=90 end
		end
		self:SetSize(width,size)
	end
	self.Label:SetFont(font)
	self.Label:SetText(self.Label:GetText()) -- refresh the text so the font actually applies
end
function ScalableButton:GetFont()
	return self.Label:GetFont()
end
function ScalableButton:SetText( text, wrap )
	if self.AutoSize then
		-- recalc size
		local font=self.Label:GetFont()
		local size
		if Turbine.UI.Lotro.FontInfo~=nil then
			if Turbine.UI.Lotro.FontInfo[font]~=nil then
				size=Turbine.UI.Lotro.FontInfo[font].size
			end
		end
		if size==nil then
			-- fontInfo data not found - try to get size directly from Turbine enumeration
			getFontSizeFromFont(font)
		end
		if size~=nil then
			size=size+10
			if size<28 then size=28 end
			local width=self:GetWidth()
			-- Note, if we don't have a fontMetricInstance, we can only scale height
			if self.FontMetricInstance~=nil then
				self.FontMetricInstance:SetFont(font)
				width=self.FontMetricInstance:GetTextWidth(text)+20 -- add 10 for borders and margin
				if width<90 then width=90 end
			end
			self:SetSize(width,size)
		end
	end
	if wrap then
		self.Label:SetMultiline(true)
	else
		self.Label:SetMultiline(false)
	end
	self.Label:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
	self.Label:SetText(text)
end
function ScalableButton:GetText()
	return self.Label:GetText()
end
function ScalableButton:SetAutoSize( setAuto )
	if setAuto==nil then setAuto=false end
	self.AutoSize=setAuto
end
function getFontSizeFromFont(font)
	local size
	for k,v in pairs(Turbine.UI.Lotro.Font) do
		if v==font then
			local name=tostring(k)
			local i=string.find(name,"%d")
			if i~=nil then
				size=tonumber(string.sub(name,i))
			end
			break
		end
	end
	return size
end
function ScalableButton:SetAllowDrop(allowDrop)
	if allowDrop then
		self.QS:SetAllowDrop(true)
		self.QSAllowDrop=true
	else
		self.QS:SetAllowDrop(false)
		self.QSAllowDrop=false
		-- there is a glitch that allows a control to dragdrop onto itself and wipeout the contents of a quickslot. this attempts to fix that
		self.QS.DragDrop=function()
			self.QS:SetWantsUpdates(true)
		end
	end
end
function ScalableButton:SetQsEnabled(enable)
	if enable then
		self.QS:SetVisible(true)
		self.MouseClick=nil
	else
		self.QS:SetVisible(false) -- if not visible it also can't be dropped onto or clicked
	end
end
function ScalableButton:SetShortcut(sc)
	local data=sc:GetData()
	local typ=sc:GetType()
	self.QSData=data
	self.QSType=type
	self.QS:SetShortcut(sc)
end
function ScalableButton:SetFontMetricInstance(fontMetricInstance)
	self.FontMetricInstance=fontMetricInstance
end