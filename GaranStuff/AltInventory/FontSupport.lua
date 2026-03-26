-- this file includes several font enhancements:
-- the undocumented fonts are added to the Turbine.UI.Lotro.Fonts enumeration
-- a new Turbine.UI.Lotro.FontInfo extended font information table
-- a FontMetric class is added to allow determining required text display widths and heights
-- a FontSelect class is added with a font selection list control
--	the FontSelectList object is strictly for internal use and should not be accessed directly)
--	just create an instanceof the FontSelect object and remember to include the FontSelect.tga file in your Resources folder
--	set a font using FontSelect.SetFont() or accept the default Verdana20
--	then assign an event handler to the FontSelect.FontChanged event
--	or use FontSelect.GetFont() to retrieve the currently selected font

if importPath==nil then importPath=string.gsub(getfenv(1)._.Name,"%.FontSupport","").."." end
if resourcePath==nil then resourcePath=string.gsub(importPath,"%.","/").."Resources/" end

if Turbine==nil then
	import "Turbine"
end
if Turbine.UI==nil then
	import "Turbine.UI"
end
if Turbine.UI.Lotro==nil then
	import "Turbine.UI.Lotro"
end

-- as of update U24 there are no longer any identified but undocumented fonts. :)

Turbine.UI.Lotro.FontInfo={} -- table for extended font information
for k,v in pairs(Turbine.UI.Lotro.Font) do
	if k~="TrajanPro25" then
		local name=tostring(k)
		local i=string.find(name,"%d")
		if i~=nil then
			local size=tonumber(string.sub(name,i))
			name=string.sub(name,0,i-1)
			i=string.find(name,"%u",2)
			while i~=nil do
				name=string.sub(name,0,i-1).." "..string.sub(name,i)
				i=string.find(name,"%u",i+2) -- increment by 2 since we just added a padding space
			end
			local bold=not(string.find(name," Bold",1,true)==nil)
			Turbine.UI.Lotro.FontInfo[v]={["name"]=name,["size"]=size,["bold"]=bold}
		end
	end
end
function GetLargerFont(font)
	local retVal
	if font~=nil then
		local tmpFont={}
		local fontName,size
		local tmpInfo=Turbine.UI.Lotro.FontInfo[font]
		fontName=tmpInfo.name
		size=tmpInfo.size
		for k,v in pairs(Turbine.UI.Lotro.FontInfo) do
			if v.name==fontName and v.size>size then
				table.insert(tmpFont,{k,v.size})
			end
		end
		if #tmpFont==0 then
			-- return nil
		else
			if #tmpFont>1 then
				table.sort(tmpFont,function(arg1,arg2) if arg1[2]<arg2[2] then return true end end)
			end
			-- first font that is larger than font is now in element 1
			retVal=tmpFont[1][1]
		end
	end	
	return retVal
end
function GetSmallerFont(font)
	local retVal	
	if font~=nil then
		local tmpFont={}
		local fontName,size
		local tmpInfo=Turbine.UI.Lotro.FontInfo[font]
		fontName=tmpInfo.name
		size=tmpInfo.size
		for k,v in pairs(Turbine.UI.Lotro.FontInfo) do
			if v.name==fontName and v.size<size then
				table.insert(tmpFont,{k,v.size})
			end
		end
		if #tmpFont==0 then
			-- return nil
		else
			if #tmpFont>1 then
				table.sort(tmpFont,function(arg1,arg2) if arg1[2]>arg2[2] then return true end end)
			end
			-- first font that is smaller than font is now in element 1
			retVal=tmpFont[1][1]
		end
	end
	return retVal
end

FontMetric = class( Turbine.UI.Label );
function FontMetric:Constructor()
	Turbine.UI.Control.Constructor( self );
	self.Text=Turbine.UI.Label();
	self.Text:SetParent(self);
	self.VScroll=Turbine.UI.Lotro.ScrollBar();
	self.VScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
	self.VScroll:SetParent(self);
	self.Text:SetVerticalScrollBar(self.VScroll);
	self.HScroll=Turbine.UI.Lotro.ScrollBar();
	self.HScroll:SetOrientation(Turbine.UI.Orientation.Horizontal);
	self.HScroll:SetParent(self);
	self.Text:SetHorizontalScrollBar(self.HScroll);
	self.fontHeight=12;
	self.SetFont=function(sender,font)
		self.Text:SetFont(font);
		if Turbine.UI.Lotro.FontInfo[font]~=nil then
			self.fontHeight=Turbine.UI.Lotro.FontInfo[font].size
		else
			self.fontHeight=self:GetHeight()
		end
	end
	self.GetTextWidth=function(sender,text,height)
		if height==nil or height<self.fontHeight then height=self.fontHeight end
		local width=0
		if text~=nil and text~="" then
			self.Text:SetHeight(height);
			self.Text:SetMultiline(false);
			width=string.len(text);
			self.Text:SetText(text);
			self.Text:SetWidth(width);
			while self.HScroll:IsVisible() do
				width=width+1;
				self.Text:SetWidth(width);
			end
		end
		return width;
	end
	self.GetTextHeight=function(sender,text,width)
		if width~=nil then
			-- since we don't actually have a minimum width for a single character, use the point size as a good estimate - this is needed to prevent possible infinite loop issues
			if width<self.fontHeight then width=self.fontHeight end
			self.Text:SetWidth(width)
		end
		local height=self.fontHeight
		self.Text:SetMultiline(true);
		if text~=nil and text~="" then
			self.Text:SetText(text);
			self.Text:SetHeight(height);
			while self.HScroll:IsVisible() or self.VScroll:IsVisible() do
				height=height+1;
				self.Text:SetHeight(height);
			end
		end
		return height;
	end
end

FontSelectList=Turbine.UI.Control()
FontSelectList:SetSize(342,202)
FontSelectList:SetBackColor(Turbine.UI.Color(0,0,.4))
FontSelectList.ListData=Turbine.UI.ListBox()
FontSelectList.ListData:SetParent(FontSelectList)
FontSelectList.ListData:SetSize(FontSelectList:GetWidth()-12,FontSelectList:GetHeight()-2)
FontSelectList.ListData:SetPosition(1,1)
FontSelectList.ListData:SetBackColor(Turbine.UI.Color.Black)
FontSelectList.ListVScroll=Turbine.UI.Lotro.ScrollBar();
FontSelectList.ListVScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
FontSelectList.ListVScroll:SetParent(FontSelectList);
FontSelectList.ListVScroll:SetBackColor(Turbine.UI.Color.Black);
FontSelectList.ListVScroll:SetPosition(FontSelectList:GetWidth()-11,0);
FontSelectList.ListVScroll:SetWidth(10);
FontSelectList.ListVScroll:SetHeight(FontSelectList:GetHeight());
FontSelectList.ListData:SetVerticalScrollBar(FontSelectList.ListVScroll);
FontSelectList.ListData.SelectedIndexChanged=function(sender,args)
	if FontSelectList.SelectedItem~=nil then
		FontSelectList.SelectedItem:SetBackColor(FontSelectList.ListData:GetBackColor())
		FontSelectList.SelectedItem.MouseClick=nil

	end
	FontSelectList.SelectedItem=FontSelectList.ListData:GetSelectedItem()
	FontSelectList.SelectedItem:SetBackColor(FontSelectList:GetBackColor())
	FontSelectList.SelectedItem.MouseClick=function(sender,args)

		-- needed to allow closing the window without a change
		if FontSelectList.SelectionChanged~=nil then
			FontSelectList.SelectionChanged()
		end
		FontSelectList:SetVisible(false)
	end

	if FontSelectList.SelectionChanged~=nil then
		FontSelectList.SelectionChanged(sender,args)
	end
end
FontSelectList.ListData:SetSelectedIndex(24) -- verdana20
local defaultIndex=1
local fontList={}

for k,v in pairs(Turbine.UI.Lotro.Font) do
	if Turbine.UI.Lotro.FontInfo[v]~=nil then
		table.insert(fontList,{Turbine.UI.Lotro.FontInfo[v].name,Turbine.UI.Lotro.FontInfo[v].size,v})
	end
end
table.sort(fontList,function(arg1,arg2) if arg1[1]<arg2[1] then return true else if arg1[1]==arg2[1] and arg1[2]<arg2[2] then return true end end end)
for k,v in ipairs(fontList) do
	local tmpItem=Turbine.UI.Label()
	tmpItem:SetParent(FontSelectList.ListData)
	tmpItem:SetSize(FontSelectList.ListData:GetWidth(),v[2])
	tmpItem:SetFont(v[3])
	tmpItem:SetText("("..tostring(v[2])..") "..v[1]);
	if v[3]==Turbine.UI.Lotro.Font.Verdana20 then defaultIndex=k end
	FontSelectList.ListData:AddItem(tmpItem);
end
FontSelectList.SetFont=function(sender,newFont)
	-- set the currently selected item to the one matching the font
	local index=1
	for k=1,FontSelectList.ListData:GetItemCount() do
		if FontSelectList.ListData:GetItem(k):GetFont()==newFont then
			index=k
			break
		end
	end
	FontSelectList.ListData:SetSelectedIndex(index)
end
FontSelectList.ListData:SetSelectedIndex(defaultIndex)
FontSelectList.SizeChanged=function(sender,args)
	local width,height=FontSelectList:GetSize()
	FontSelectList.ListData:SetSize(width-12,height-2)
	FontSelectList.ListVScroll:SetLeft(width-12)
	FontSelectList.ListVScroll:SetHeight(height-2)

	local tmpItem
	for k=1,FontSelectList.ListData:GetItemCount() do
		tmpItem=FontSelectList.ListData:GetItem(k)
		tmpItem:SetWidth(width-2)
	end
end
FontSelectList.Click=function(sender,args)
	FontSelectList:SetVisible(false)
end
FontSelectList:SetVisible(false)
FontSelect=class(Turbine.UI.Button)
-- an icon with a pop-up font selection list
function FontSelect:Constructor()
	Turbine.UI.Button.Constructor( self );
	self:SetSize(20,20)
	self:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	self:SetBackground(resourcePath.."FontSelect.tga")
	self.MouseClick=function(sender,args)
		--display FontSelectList
		local zOrder=sender:GetZOrder()+1
		local parent=sender:GetParent()
		local controlList=parent:GetControls()
		for k=1,controlList:GetCount() do
			local child=controlList:Get(k)
			if child~=FontSelectList and type(child)=="table" and child.GetZOrder then
				local tmpOrder=child:GetZOrder()
				if tmpOrder>=zOrder then zOrder=tmpOrder+1 end
			end
		end
		FontSelectList:SetParent(parent)
		FontSelectList:SetVisible(true)
		FontSelectList:SetZOrder(zOrder) -- force it topmost within it's parent
		FontSelectList.SelectionChanged=nil

		local top=self:GetTop()
		local left=self:GetLeft()
		if parent:GetHeight()-FontSelectList:GetHeight()<top then top=parent:GetHeight()-FontSelectList:GetHeight() end
		if top<0 then top=0 end
		if parent:GetWidth()-FontSelectList:GetWidth()<left then left=parent:GetWidth()-FontSelectList:GetWidth() end
		if left<0 then left=0 end
		FontSelectList:SetPosition(left,top)
		FontSelectList:SetFont(self.Font)
		FontSelectList.SelectionChanged=function(sender,args)
			self.Font=FontSelectList.SelectedItem:GetFont()
			if self.FontChanged~=nil then
				self.FontChanged(self,args)
			end
		end
	end
	self.Font=Turbine.UI.Lotro.Font.Verdana20
	self.SetFont=function(sender,newFont)
		if newFont~=nil then
			-- set the selected index to the new font
			self.Font=newFont
		end
	end
	self.GetFont=function()
		return self.Font
	end
end

