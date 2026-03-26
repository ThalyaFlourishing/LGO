-- this window allows searching the AIItemData table by name, category, durability, quality...
-- set controls using Settings.fontFace for text size, etc.

itemExplorer=Turbine.UI.Lotro.Window()
itemExplorer:SetText(Resource[language][141])
itemExplorer:SetSize(800,800)
itemExplorer:SetResizable(true)
itemExplorer:SetZOrder(1)
itemExplorer.borderWidth=10
itemExplorer.captionPad=5
itemExplorer.controlSpace=10
itemExplorer.controlVerticalSep=15
itemExplorer.msgBox=nil

itemExplorer.MagnifierBack=Turbine.UI.Control()
itemExplorer.MagnifierBack:SetBackColor(Turbine.UI.Color.Black)
itemExplorer.MagnifierBack:SetSize(32,32)
itemExplorer.MagnifierBack:SetPosition(0,0)
itemExplorer.MagnifierBack:SetParent(itemExplorer)
itemExplorer.MagnifierBack:SetVisible(false)
itemExplorer.Magnifier=Turbine.UI.Control()
itemExplorer.Magnifier:SetParent(itemExplorer.MagnifierBack)
itemExplorer.Magnifier:SetSize(32,32)
itemExplorer.Magnifier:SetPosition(0,0)
itemExplorer.Magnifier:SetBlendMode(Turbine.UI.BlendMode.Overlay)

itemExplorer.minIDCapt=Turbine.UI.Label()
itemExplorer.minIDCapt:SetMultiline(false)
itemExplorer.minIDCapt:SetParent(itemExplorer)
itemExplorer.minIDCapt:SetPosition(10,45) -- anchor position for all labels and captions - they layout from here
itemExplorer.minIDBack=Turbine.UI.Control()
itemExplorer.minIDBack:SetParent(itemExplorer)
itemExplorer.minID=Turbine.UI.TextBox()
itemExplorer.minID:SetMultiline(false)
itemExplorer.minID:SetParent(itemExplorer.minIDBack)
itemExplorer.minID:SetPosition(1,1)
itemExplorer.minID:SetBackColor(getContrastedBackgroundColor())

itemExplorer.maxIDCapt=Turbine.UI.Label()
itemExplorer.maxIDCapt:SetMultiline(false)
itemExplorer.maxIDCapt:SetParent(itemExplorer)
itemExplorer.maxIDBack=Turbine.UI.Control()
itemExplorer.maxIDBack:SetParent(itemExplorer)
itemExplorer.maxID=Turbine.UI.TextBox()
itemExplorer.maxID:SetMultiline(false)
itemExplorer.maxID:SetParent(itemExplorer.maxIDBack)
itemExplorer.maxID:SetPosition(1,1)

itemExplorer.nameCapt=Turbine.UI.Label()
itemExplorer.nameCapt:SetMultiline(false)
itemExplorer.nameCapt:SetParent(itemExplorer)
itemExplorer.nameBack=Turbine.UI.Control()
itemExplorer.nameBack:SetParent(itemExplorer)
itemExplorer.name=Turbine.UI.TextBox()
itemExplorer.name:SetMultiline(false)
itemExplorer.name:SetParent(itemExplorer.nameBack)
itemExplorer.name:SetPosition(1,1)

itemExplorer.name.FocusGained=function() itemExplorer.name:SetWantsKeyEvents(true) end
itemExplorer.name.FocusLost=function() itemExplorer.name:SetWantsKeyEvents(false) end
itemExplorer.name.KeyDown=function(sender,args)
	if args.Action==162 then
		itemExplorer.name:SetWantsKeyEvents(false)
		itemExplorer.searchButton:Focus()
		itemExplorer.searchButton.MouseClick()
	end
end

itemExplorer.categoryCapt=Turbine.UI.Label()
itemExplorer.categoryCapt:SetMultiline(false)
itemExplorer.categoryCapt:SetParent(itemExplorer)
itemExplorer.category=DropDownList()
itemExplorer.category:SetParent(itemExplorer)
itemExplorer.category:SetDropRows(5);
itemExplorer.category:SetZOrder(2);
itemExplorer.category.ValueMask:SetVisible(false)
itemExplorer.category:SetMatchType(1)
itemExplorer.loadCategoryList=function(value)
	local index=1 -- we pass value instead of index since the list is sorted alphabetically and if we changed languages, the selected category will have a different name

	-- load from ItemCategory table (the Turbine.Gameplay.ItemCategory enumeration is horribly out of date and doesn't easily support translation)
	local tmpList={}
	for k,v in pairs(ItemCategory) do
		table.insert(tmpList,{v[2][language],v[1]})
	end
	-- now add any categories from AIItemData that are not already in the list
	for cat, _ in pairs(AIItemData) do
		local found=false
		for k,v in pairs(tmpList) do
			if v[2]==cat then
				found=true
				break
			end
		end
		if not found then
			-- should be rare, like after an update when a new category is added
			table.insert(tmpList,{Resource[language][158].." ("..tostring(cat)..")",cat})
		end
	end

	table.sort(tmpList,function(a,b) if a[1]<b[1] then return true else return false end end)
	itemExplorer.category:ClearList()
	itemExplorer.category:AddItem(Resource[language][12],-1)
	local count=2
	for k, v in ipairs(tmpList) do
		if v[2]==value then index=count end
		count=count+1
		itemExplorer.category:AddItem(v[1],v[2])
	end
	itemExplorer.category:SetSelectedIndex(index)
end
itemExplorer.loadCategoryList(-1)

itemExplorer.qualityCapt=Turbine.UI.Label()
itemExplorer.qualityCapt:SetMultiline(false)
itemExplorer.qualityCapt:SetParent(itemExplorer)
itemExplorer.quality=DropDownList()
itemExplorer.quality:SetParent(itemExplorer)
itemExplorer.quality:SetDropRows(5);
itemExplorer.quality:SetZOrder(1);
-- fill from Turbine.Gameplay.ItemQuality
itemExplorer.loadQualityList=function(index)
	local tmpList={}
	for k,v in pairs(Turbine.Gameplay.ItemQuality) do
		if type(v)=="number" and v>=0 then
			table.insert(tmpList,{k,v})
		end
	end
	table.sort(tmpList,function(a,b) if a[2]>b[2] then return true else return false end end)
	itemExplorer.quality:ClearList()
	itemExplorer.quality:AddItem(Resource[language][12],-1)
	for k, v in ipairs(tmpList) do
		itemExplorer.quality:AddItem(v[1],v[2])
	end
	itemExplorer.quality:SetSelectedIndex(index)
end
itemExplorer.loadQualityList(1)

itemExplorer.durabilityCapt=Turbine.UI.Label()
itemExplorer.durabilityCapt:SetMultiline(false)
itemExplorer.durabilityCapt:SetParent(itemExplorer)
itemExplorer.durability=DropDownList()
itemExplorer.durability:SetParent(itemExplorer)
itemExplorer.durability:SetDropRows(5);
itemExplorer.durability:SetZOrder(1);
-- fill from Turbine.Gameplay.ItemDurability
itemExplorer.loadDurabilityList=function(index)
	local tmpList={}
	for k,v in pairs(Turbine.Gameplay.ItemDurability) do
		if type(v)=="number" and v>=0 then
			table.insert(tmpList,{k,v})
		end
	end
	table.sort(tmpList,function(a,b) if a[2]>b[2] then return true else return false end end)
	itemExplorer.durability:ClearList()
	itemExplorer.durability:AddItem(Resource[language][12],-1)
	for k, v in ipairs(tmpList) do
		itemExplorer.durability:AddItem(v[1],v[2])
	end
	itemExplorer.durability:SetSelectedIndex(index)
end
itemExplorer.loadDurabilityList(1)

itemExplorer.sortCapt=Turbine.UI.Label()
itemExplorer.sortCapt:SetMultiline(false)
itemExplorer.sortCapt:SetParent(itemExplorer)
itemExplorer.sort=DropDownList()
itemExplorer.sort:SetParent(itemExplorer)
itemExplorer.sort:SetDropRows(5);
itemExplorer.sort:SetZOrder(1);
itemExplorer.loadSortList=function(index)
	itemExplorer.sort:ClearList()
	itemExplorer.sort:AddItem("Name, ID",1)
	itemExplorer.sort:AddItem("ID",2)
	itemExplorer.sort:AddItem("Category, Name, ID",3)
	itemExplorer.sort:SetSelectedIndex(index)
end
itemExplorer.loadSortList(1)
itemExplorer.sort.SelectedIndexChanged=function()
	itemExplorer.searchButton.startTime=Turbine.Engine:GetGameTime()
	itemExplorer.showSearchMessage=false
	itemExplorer.searchButton.state=2
	itemExplorer.searchButton:SetWantsUpdates(true) -- do not allow 'Cancel' since this is just a sort of the existing items
end

itemExplorer.resultPanel=Turbine.UI.Control()
itemExplorer.resultPanel:SetParent(itemExplorer)

itemExplorer.resultList=Turbine.UI.ListBox()
itemExplorer.resultList:SetParent(itemExplorer.resultPanel)
itemExplorer.resultList:SetPosition(1,1)
itemExplorer.resultList:SetOrientation(Turbine.UI.Orientation.Horizontal)
itemExplorer.resultVScroll=Turbine.UI.Lotro.ScrollBar()
itemExplorer.resultVScroll:SetParent(itemExplorer.resultPanel)
itemExplorer.resultVScroll:SetWidth(10)
itemExplorer.resultVScroll:SetTop(1)
itemExplorer.resultVScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
itemExplorer.resultList:SetVerticalScrollBar(itemExplorer.resultVScroll)

itemExplorer.hourglass=Turbine.UI.Control()
itemExplorer.hourglass:SetParent(itemExplorer)
itemExplorer.hourglass:SetVisible(false)
itemExplorer.hourglass:SetSize(32,32)
itemExplorer.hourglass:SetZOrder(1)
itemExplorer.hourglass.state=1
itemExplorer.hourglass:SetBlendMode(Turbine.UI.BlendMode.Overlay)

itemExplorer.searchButton=ScalableButton()
itemExplorer.searchButton:SetParent(itemExplorer)
itemExplorer.searchButton:SetAutoSize(true)
itemExplorer.searchButton.Update=function()
	if itemExplorer.searchButton.categoryIndex==nil then itemExplorer.searchButton.categoryIndex=1 end
	if itemExplorer.searchButton.maxCategoryIndex==nil then itemExplorer.searchButton.maxCategoryIndex=#ItemCategoryIndex end
	if itemExplorer.searchButton.category==nil then
		itemExplorer.searchButton.category=ItemCategoryIndex[itemExplorer.searchButton.categoryIndex][1]
		itemExplorer.searchButton.startIndex=1
	end
	if itemExplorer.searchButton.startIndex==nil then itemExplorer.searchButton.startIndex=1 end
	if itemExplorer.hourglass.timeout==nil then itemExplorer.hourglass.timeout=Turbine.Engine:GetGameTime()+.125 end
	if Turbine.Engine:GetGameTime()>itemExplorer.hourglass.timeout then
		itemExplorer.hourglass.timeout=Turbine.Engine:GetGameTime()+.125
		itemExplorer.hourglass.state=itemExplorer.hourglass.state+1
		if itemExplorer.hourglass.state>8 then itemExplorer.hourglass.state=1 end
	end
	itemExplorer.hourglass:SetBackground(resourcePath.."itemExplorerMagnifier"..tostring(itemExplorer.hourglass.state)..".tga")
	if  itemExplorer.searchButton.state==1 then
		-- get next Settings.itemExplorerSearchThrottle items
		local maxIDReturned=0
		local tmpItems,nextIndex=findItemIDs(itemExplorer.searchButton.name,itemExplorer.searchButton.category,itemExplorer.searchButton.durability,itemExplorer.searchButton.quality,itemExplorer.searchButton.minID,itemExplorer.searchButton.maxID,itemExplorer.searchButton.startIndex,Settings.itemExplorerSearchThrottle) -- gets next 100 matches (if less than 100 returned then done)
		if tmpItems==nil then
			-- check if that was the last category
			itemExplorer.searchButton.categoryIndex=itemExplorer.searchButton.categoryIndex+1
			if itemExplorer.searchButton.categoryIndex>itemExplorer.searchButton.maxCategoryIndex then
				-- completed last category
				itemExplorer.searchButton.state=2
			else
				-- set up next category
				itemExplorer.searchButton.category=ItemCategoryIndex[itemExplorer.searchButton.categoryIndex][1]
				itemExplorer.searchButton.startIndex=1
			end
		else
			for k,v in ipairs(tmpItems) do
				table.insert(itemExplorer.searchButton.items,v)
			end
			if #tmpItems<Settings.itemExplorerSearchThrottle then
				itemExplorer.searchButton.categoryIndex=itemExplorer.searchButton.categoryIndex+1
				if itemExplorer.searchButton.categoryIndex>itemExplorer.searchButton.maxCategoryIndex then
					-- completed last category
					itemExplorer.searchButton.state=2
				else
					-- set up next category
					itemExplorer.searchButton.category=ItemCategoryIndex[itemExplorer.searchButton.categoryIndex][1]
					itemExplorer.searchButton.startIndex=1
				end
			else
-- need to know the index of the last item returned... nextIndex is returned by findItemIDs
				if nextIndex~=nil then
					itemExplorer.searchButton.startIndex=nextIndex
				else
					itemExplorer.searchButton.startIndex=itemExplorer.searchButton.startIndex+100
				end
			end
		end
	elseif itemExplorer.searchButton.state==2 then
		if itemExplorer.searchButton.items==nil then
			itemExplorer.searchButton.items={}
			itemExplorer.searchButton.state=5
			itemExplorer.hourglass:SetVisible(false)
			itemExplorer.searchButton:SetWantsUpdates(false)
			itemExplorer.searchButton.mode=1
			itemExplorer.searchButton:SetText(Resource[language][14])
		else
			-- sort result table
			sort=itemExplorer.sort:GetValue()
			-- entries are {id, name, category}
			if sort==2 then
				-- id
				table.sort(itemExplorer.searchButton.items, function(a,b) if a[1]<b[1] then return true end end)
			elseif sort==3 then
				-- category, name, id
				table.sort(itemExplorer.searchButton.items, function(a,b) if a[3]<b[3] then return true else if a[3]==b[3] and a[2]<b[2] then return true else if a[3]==b[3] and a[2]==b[2] and a[1]<b[1] then return true end end end end)
			else
				-- name, id
				table.sort(itemExplorer.searchButton.items, function(a,b) if a[2]<b[2] then return true else if a[2]==b[2] and a[1]<b[1] then return true end end end)
			end
		end
		itemExplorer.resultList:ClearItems()
		itemExplorer.searchButton.addIndex=1
		itemExplorer.searchButton.addMax=#itemExplorer.searchButton.items
		itemExplorer.searchButton.state=3
	elseif itemExplorer.searchButton.state==3 then
		-- add the next 100 result table items to list
		local count=0
		while itemExplorer.searchButton.addIndex<=itemExplorer.searchButton.addMax and count<100 do
			local newControl=itemDetails(itemExplorer.searchButton.items[itemExplorer.searchButton.addIndex][1])
			itemExplorer.resultList:AddItem(newControl)
			itemExplorer.searchButton.addIndex=itemExplorer.searchButton.addIndex+1
			count=count+1
		end
		if itemExplorer.searchButton.addIndex>itemExplorer.searchButton.addMax then
			itemExplorer.searchButton.state=4
		end
	elseif itemExplorer.searchButton.state==4 then
		itemExplorer.hourglass:SetVisible(false)
		itemExplorer.searchButton:SetWantsUpdates(false)
		itemExplorer.searchButton.mode=1
		itemExplorer.searchButton:SetText(Resource[language][14])
		local count=itemExplorer.resultList:GetItemCount()
		local elapsedSeconds=math.floor((Turbine.Engine:GetGameTime()-itemExplorer.searchButton.startTime)*100+.5)/100
		if itemExplorer.showSearchMessage then
			if count==0 then
				itemExplorer.msgBox=PopUpDialog("No items found","No items found. Try changing your criteria and search again.",1,"OK")
			elseif count==1 then
				itemExplorer.msgBox=PopUpDialog("Item found","1 item found.\nin "..tostring(elapsedSeconds).." seconds",1,"OK")
			else
				itemExplorer.msgBox=PopUpDialog("Items found",tostring(count).." items found.\nin "..tostring(elapsedSeconds).." seconds",1,"OK")
			end
			itemExplorer.showSearchMessage=false
		end
	else
		-- not sure how we got here, just kill it
		itemExplorer.hourglass:SetVisible(false)
		itemExplorer.searchButton:SetWantsUpdates(false)
		itemExplorer.searchButton.mode=1
		itemExplorer.searchButton:SetText(Resource[language][14])
	end
end
itemExplorer.searchButton:SetWantsUpdates(false) -- initially this should be turned off
itemExplorer.searchButton.mode=1
itemExplorer.searchButton.MouseClick=function(sender,args)
	if itemExplorer.msgBox~=nil then
		itemExplorer.msgBox:SetVisible(false)
		itemExplorer.msgBox=nil
	end
	-- toggle between "Search" and "Cancel"
	if itemExplorer.searchButton.mode==1 then
		-- uses an update handler because some criteria may yield hundreds of thousands of results - might want a 'Cancel' button
		local minID,maxID,name,category,quality,durability
		minID=tonumber(string.trim(itemExplorer.minID:GetText()))
		maxID=tonumber(string.trim(itemExplorer.maxID:GetText()))
		name=string.trim(itemExplorer.name:GetText())
		if name=="" then name=nil end
		category=itemExplorer.category:GetValue()
		if category==-1 then
			itemExplorer.searchButton.categoryIndex=1
			itemExplorer.searchButton.maxCategoryIndex=#ItemCategoryIndex
			itemExplorer.searchButton.category=ItemCategoryIndex[itemExplorer.searchButton.categoryIndex][1]
			itemExplorer.searchButton.startIndex=1
		else
			for k,v in ipairs(ItemCategoryIndex) do
				if v[1]==category then
					itemExplorer.searchButton.categoryIndex=k
					itemExplorer.searchButton.maxCategoryIndex=k
					itemExplorer.searchButton.category=category
					itemExplorer.searchButton.startIndex=1
					break
				end
			end
		end
		quality=itemExplorer.quality:GetValue()
		if quality==-1 then quality=nil end
		durability=itemExplorer.durability:GetValue()
		if durability==-1 then durability=nil end
		if minID==nil and maxID==nil and (name==nil or name=="") and (category==nil or category==-1) and quality==nil and durability==nil then
			itemExplorer.msgBox=PopUpDialog("Invalid Search Parameters","Please enter at least one search parameter and try again.",1,"OK")
		else
			fontMetric:SetFont(Settings.fontFace) -- do this here so we don't have to set it for every itemDetails control
			itemExplorer.resultList:ClearItems()
			itemExplorer.searchButton.minID=minID
			itemExplorer.searchButton.maxID=maxID
			itemExplorer.searchButton.name=name
			itemExplorer.searchButton.durability=durability
			itemExplorer.searchButton.quality=quality
			itemExplorer.searchButton.items={}
			itemExplorer.searchButton.state=1
			itemExplorer.searchButton.startTime=Turbine.Engine:GetGameTime()
			itemExplorer.hourglass.state=1
			itemExplorer.hourglass:SetBackground(resourcePath.."itemExplorerMagnifier"..tostring(itemExplorer.hourglass.state)..".tga")
			itemExplorer.hourglass:SetVisible(true)
			itemExplorer.showSearchMessage=true
			itemExplorer.searchButton:SetWantsUpdates(true)
			itemExplorer.searchButton.mode=0
			itemExplorer.searchButton:SetText(Resource[language][37])
		end
	else
		-- we are in 'Cancel' mode
		itemExplorer.searchButton.mode=1
		itemExplorer.hourglass:SetVisible(false)
		itemExplorer.searchButton:SetWantsUpdates(false)
		itemExplorer.searchButton.mode=1
		itemExplorer.searchButton:SetText(Resource[language][14])
	end
end
itemExplorer.rescanButton=ScalableButton()
--*** set this visible to force rescan of item database
itemExplorer.rescanButton:SetVisible(false)
itemExplorer.rescanButton:SetParent(itemExplorer)
itemExplorer.rescanButton:SetAutoSize(true)
itemExplorer.rescanButton.MouseClick=function()
	Settings.itemDataCurrentScanStart=Settings.itemDataCurrentScanItem -- just in case we were already in a scan
	Settings.itemDataScanLooped=false -- also in case we were already in a scan, force it to check again
	Settings.itemDataNeedsScan=true
	itemDataScan:SetWantsUpdates(true)
end

itemDetails=class(Turbine.UI.Control)
function itemDetails:Constructor(itemID)
	Turbine.UI.Control.Constructor( self )
	local width=300
	local height=300
	local size=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size+2
	-- itemInfoControl 35x35, id
	-- name, description, quality, durability
	self.itemID=itemID
	self:SetBackColor(Settings.trimColor)
	self.background=Turbine.UI.Control()
	self.background:SetParent(self)
	self.background:SetPosition(1,1)
	self.background:SetBackColor(Settings.backColor)
	self.background.MouseDoubleClick=function()
		itemInfoDetail:ShowItem(self.itemID)
	end
	local tmpItemInfo=getItemInfo(itemID)
	local tmpQuality=tmpItemInfo:GetQuality()
	qualityColor=Turbine.UI.Color.LightGray
	if tmpQuality==1 then
		-- legendary - gold
		qualityColor=Turbine.UI.Color.Gold
	elseif tmpQuality==2 then
		-- rare
		qualityColor=Turbine.UI.Color.Purple
	elseif tmpQuality==3 then
		-- incomprable
		qualityColor=Turbine.UI.Color.Teal
	elseif tmpQuality==4 then
		-- uncommon
		qualityColor=Turbine.UI.Color.Yellow
	elseif tmpQuality==5 then
		-- common
		qualityColor=Turbine.UI.Color.White
	end
	self.itemInfo=Turbine.UI.Lotro.ItemInfoControl(tmpItemInfo)	
	self.itemInfo:SetParent(self.background)
	self.itemInfo:SetPosition(1,1)
	self.id=Turbine.UI.Label()
	self.id:SetFont(Settings.fontFace)
	self.id:SetSelectable(true)
	self.id:SetForeColor(qualityColor)
	self.id:SetParent(self.background)
	self.id:SetSize(width-40,size+2)
	self.id:SetText(string.format("0x%x",itemID))
	local tmpTop=(35-size)/2 --what if font size is ever >35?
	self.id:SetPosition(36,tmpTop)
	self.id.MouseDoubleClick=function()
		itemInfoDetail:ShowItem(self.itemID)
	end
	self.name=Turbine.UI.Label()
	self.name:SetFont(Settings.fontFace)
	self.name:SetSelectable(true)
	self.name:SetForeColor(qualityColor)
	self.name:SetParent(self.background)
	self.name:SetPosition(1,41)
	self.name:SetMultiline(true)
	local tmpName=tmpItemInfo:GetName()
	tmpWidth=fontMetric:GetTextWidth(tmpName,size)
	if tmpWidth+4>width then
		tmpWidth=width-4
		-- determine if we have to crop text
		local tmpHeight=fontMetric:GetTextHeight(tmpName,tmpWidth)
		local tmpLen=string.len(tmpName)-2
		local tmpCrop=tmpName
		while tmpHeight>size*2+4 and tmpLen>0 do
			tmpLen=tmpLen-1
			tmpCrop=string.sub(tmpName,1,tmpLen).."..."
			tmpHeight=fontMetric:GetTextHeight(tmpCrop,tmpWidth)
		end
		tmpName=tmpCrop
	end
	self.name:SetSize(tmpWidth,size*2+2)
	self.name:SetText(tmpName)
	self.name.MouseDoubleClick=function()
		itemInfoDetail:ShowItem(self.itemID)
	end

	self.quality=Turbine.UI.Label()
	self.quality:SetParent(self.background)
	self.quality:SetPosition(1,self.name:GetTop()+self.name:GetHeight()+5)
	self.quality:SetMouseVisible(false)
	self.quality:SetFont(Settings.fontFace)
	self.quality:SetForeColor(qualityColor)
	self.quality:SetSize(296,size)
	local tmpText="Undefined"
	for k,v in pairs(Turbine.Gameplay.ItemQuality) do
		if v==tmpQuality then
			tmpText=k
			break
		end
	end

	self.quality:SetText(tmpText)

	self.durability=Turbine.UI.Label()
	self.durability:SetParent(self.background)
	self.durability:SetPosition(1,self.quality:GetTop()+self.quality:GetHeight()+5)
	self.durability:SetMouseVisible(false)
	self.durability:SetFont(Settings.fontFace)
	self.durability:SetSize(296,size)
	local tmpText="Undefined"
	local tmpDurability=tmpItemInfo:GetDurability()
	for k,v in pairs(Turbine.Gameplay.ItemDurability) do
		if v==tmpDurability then
			tmpText=k
			break
		end
	end
	self.durability:SetText(tmpText)

	self.category=Turbine.UI.Label()
	self.category:SetParent(self.background)
	self.category:SetPosition(1,self.durability:GetTop()+self.durability:GetHeight()+5)
	self.category:SetMouseVisible(false)
	self.category:SetFont(Settings.fontFace)
	self.category:SetSize(296,size)
	local tmpCategory=tmpItemInfo:GetCategory()
	local tmpText="Unknown ("..tostring(tmpCategory)..")"
	for k,v in pairs(Turbine.Gameplay.ItemCategory) do
		if v==tmpCategory then
			tmpText=k
			break
		end
	end
	self.category:SetText(tmpText)

	self.description=Turbine.UI.Label()
	self.description:SetParent(self.background)
	self.description:SetSelectable(true)
	self.description:SetPosition(1,self.category:GetTop()+self.category:GetHeight()+5)
	self.description:SetSize(296,296-self:GetTop())
	self.description:SetFont(Settings.fontFace)
	self.description:SetMarkupEnabled(true)
	self.description:SetText(tmpItemInfo:GetDescription())
	self.description.MouseDoubleClick=function()
		itemInfoDetail:ShowItem(self.itemID)
	end

	self:SetSize(width,height)
	self.background:SetSize(width-2,height-2)
end

itemExplorer.SetFont=function(sender,font)
	if font==nil then font=Settings.fontFace end
	itemExplorer.minIDCapt:SetFont(font)
	itemExplorer.minIDCapt:SetText(Resource[language][142])
	itemExplorer.minID:SetFont(font)
	itemExplorer.minID:SetText(itemExplorer.minID:GetText())

	itemExplorer.maxIDCapt:SetFont(font)
	itemExplorer.maxIDCapt:SetText(Resource[language][143])
	itemExplorer.maxID:SetFont(font)
	itemExplorer.maxID:SetText(itemExplorer.maxID:GetText())

	itemExplorer.nameCapt:SetFont(font)
	itemExplorer.nameCapt:SetText(Resource[language][72])
	itemExplorer.name:SetFont(font)
	itemExplorer.name:SetText(itemExplorer.name:GetText())
	itemExplorer.categoryCapt:SetFont(font)
	itemExplorer.categoryCapt:SetText(Resource[language][19])

	itemExplorer.qualityCapt:SetFont(font)
	itemExplorer.qualityCapt:SetText(Resource[language][20])
	itemExplorer.durabilityCapt:SetFont(font)
	itemExplorer.durabilityCapt:SetText(Resource[language][144])
	itemExplorer.sortCapt:SetFont(font)
	itemExplorer.sortCapt:SetText(Resource[language][15])

	itemExplorer.searchButton:SetFont(font)
	itemExplorer.searchButton:SetText(Resource[language][14])
	-- need to clear and reload the dropdownlistboxes
	itemExplorer.loadCategoryList(itemExplorer.category:GetValue()) -- set selection based on value since categories are sorted alphabetically
	itemExplorer.loadQualityList(itemExplorer.quality:GetSelectedIndex())
	itemExplorer.loadDurabilityList(itemExplorer.durability:GetSelectedIndex())
	itemExplorer.loadSortList(itemExplorer.sort:GetSelectedIndex())

	itemExplorer.rescanButton:SetFont(font)
	itemExplorer.rescanButton:SetText(Resource[language][171])

	itemExplorer:Layout()
end
itemExplorer.SetTrimColor=function(sender,trimColor)
	if trimColor==nil then trimColor=Settings.trimColor end
	itemExplorer.category:SetBorderColor(trimColor)
	itemExplorer.quality:SetBorderColor(trimColor)
	itemExplorer.durability:SetBorderColor(trimColor)
	itemExplorer.sort:SetBorderColor(trimColor)
	itemExplorer.resultPanel:SetBackColor(trimColor)
end
itemExplorer.SetBackColor=function(sender,backColor)
	if backColor==nil then backColor=Settings.backColor end
	local tmpContrastBack=getContrastedBackgroundColor()
	itemExplorer.minID:SetBackColor(tmpContrastBack)
	itemExplorer.maxID:SetBackColor(tmpContrastBack)
	itemExplorer.name:SetBackColor(tmpContrastBack)
	itemExplorer.category:SetBackColor(backColor);
	itemExplorer.quality:SetBackColor(backColor);
	itemExplorer.durability:SetBackColor(backColor);
	itemExplorer.sort:SetBackColor(backColor);	
	itemExplorer.category:SetCurrentBackColor(backColor);
	itemExplorer.quality:SetCurrentBackColor(backColor);
	itemExplorer.durability:SetCurrentBackColor(backColor);
	itemExplorer.sort:SetCurrentBackColor(backColor);	
	itemExplorer.resultList:SetBackColor(backColor)
end
itemExplorer.SetFontColor=function(sender,fontColor)
	if fontColor==nil then fontColor=Settings.fontColor end
	itemExplorer.minIDCapt:SetForeColor(fontColor)
	itemExplorer.minID:SetForeColor(fontColor)
	itemExplorer.maxIDCapt:SetForeColor(fontColor)
	itemExplorer.maxID:SetForeColor(fontColor)
	itemExplorer.nameCapt:SetForeColor(fontColor)
	itemExplorer.name:SetForeColor(fontColor)
	itemExplorer.categoryCapt:SetForeColor(fontColor);
	itemExplorer.qualityCapt:SetForeColor(fontColor);
	itemExplorer.durabilityCapt:SetForeColor(fontColor);
	itemExplorer.sortCapt:SetForeColor(fontColor)
end
itemExplorer.SetHeadingsColor=function(sender,headingsColor)
	if headingsColor==nil then headingsColor=Settings.headingsColor end
end
itemExplorer.SetListTextColor=function(sender,listTextColor)
	if listTextColor==nil then listTextColor=Settings.listTextColor end
	itemExplorer.category:SetTextColor(Settings.listTextColor)
	itemExplorer.quality:SetTextColor(Settings.listTextColor)
	itemExplorer.durability:SetTextColor(Settings.listTextColor)
	itemExplorer.sort:SetTextColor(Settings.listTextColor)
end
itemExplorer.SetPanelBackColor=function(sender,panelBackColor)
	if panelBackColor==nil then panelBackColor=Settings.panelBackColor end
end
itemExplorer.Layout=function()
	-- size and position controls
	fontMetric:SetFont(itemExplorer.minID:GetFont()) -- read from control just incase we decide to allow this window to have a seperate font assignment from the default Settings.fontFace
	local controlHeight=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size+2
	itemExplorer.captionWidth=fontMetric:GetTextWidth(Resource[language][142])
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][72])
	if tmpWidth>itemExplorer.captionWidth then itemExplorer.captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][19])
	if tmpWidth>itemExplorer.captionWidth then itemExplorer.captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][20])
	if tmpWidth>itemExplorer.captionWidth then itemExplorer.captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][143])
	if tmpWidth>itemExplorer.captionWidth then itemExplorer.captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][144])
	if tmpWidth>itemExplorer.captionWidth then itemExplorer.captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][15])
	if tmpWidth>itemExplorer.captionWidth then itemExplorer.captionWidth=tmpWidth end

	-- we now have the max width for captions
	itemExplorer.minIDCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	itemExplorer.maxIDCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	itemExplorer.nameCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	itemExplorer.categoryCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	itemExplorer.qualityCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	itemExplorer.durabilityCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	itemExplorer.sortCapt:SetSize(itemExplorer.captionWidth,controlHeight)
	-- controls are: row1: minid, maxid, name row2: category, quality, durability row3: sort
	-- row1
	local rowTop=itemExplorer.minIDCapt:GetTop()
	local controlWidth=math.floor((itemExplorer:GetWidth()-itemExplorer.borderWidth*2-itemExplorer.captionPad*3-itemExplorer.controlSpace*2-itemExplorer.captionWidth*3)/3+.5)
	itemExplorer.minIDBack:SetPosition(itemExplorer.minIDCapt:GetLeft()+itemExplorer.captionWidth+itemExplorer.captionPad,rowTop)
	itemExplorer.minIDBack:SetSize(controlWidth,controlHeight)
	itemExplorer.minID:SetSize(controlWidth-2,controlHeight-2)

	itemExplorer.maxIDCapt:SetPosition(itemExplorer.minIDBack:GetLeft()+controlWidth+itemExplorer.controlSpace,rowTop)
	itemExplorer.maxIDBack:SetPosition(itemExplorer.maxIDCapt:GetLeft()+itemExplorer.captionWidth+itemExplorer.captionPad,rowTop)
	itemExplorer.maxIDBack:SetSize(controlWidth,controlHeight)
	itemExplorer.maxID:SetSize(controlWidth-2,controlHeight-2)

	itemExplorer.nameCapt:SetPosition(itemExplorer.maxIDBack:GetLeft()+controlWidth+itemExplorer.controlSpace,rowTop)
	itemExplorer.nameBack:SetPosition(itemExplorer.nameCapt:GetLeft()+itemExplorer.captionWidth+itemExplorer.captionPad,rowTop)
	itemExplorer.nameBack:SetSize(controlWidth,controlHeight)
	itemExplorer.name:SetSize(controlWidth-2,controlHeight-2)

	--row2
	rowTop=itemExplorer.minIDCapt:GetTop()+controlHeight+itemExplorer.controlVerticalSep
	itemExplorer.categoryCapt:SetPosition(itemExplorer.minIDCapt:GetLeft(),rowTop)
	itemExplorer.category:SetPosition(itemExplorer.categoryCapt:GetLeft()+itemExplorer.captionWidth+itemExplorer.captionPad,rowTop)
	itemExplorer.category:SetSize(controlWidth,controlHeight)
	itemExplorer.qualityCapt:SetPosition(itemExplorer.maxIDCapt:GetLeft(),rowTop)
	itemExplorer.quality:SetPosition(itemExplorer.maxIDBack:GetLeft(),rowTop)
	itemExplorer.quality:SetSize(controlWidth,controlHeight)
	itemExplorer.durabilityCapt:SetPosition(itemExplorer.nameCapt:GetLeft(),rowTop)
	itemExplorer.durability:SetPosition(itemExplorer.nameBack:GetLeft(),rowTop)
	itemExplorer.durability:SetSize(controlWidth,controlHeight)

	--row3
	rowTop=itemExplorer.categoryCapt:GetTop()+controlHeight+itemExplorer.controlVerticalSep	
	itemExplorer.sortCapt:SetPosition(itemExplorer.categoryCapt:GetLeft(),rowTop)
	itemExplorer.sort:SetPosition(itemExplorer.category:GetLeft(),rowTop)
	itemExplorer.sort:SetSize(controlWidth,controlHeight)

	itemExplorer.searchButton:SetPosition((itemExplorer:GetWidth()-itemExplorer.searchButton:GetWidth())/2,itemExplorer:GetHeight()-itemExplorer.borderWidth-itemExplorer.searchButton:GetHeight())
	itemExplorer.rescanButton:SetPosition(itemExplorer:GetWidth()-45-itemExplorer.rescanButton:GetWidth(),itemExplorer.searchButton:GetTop())

	itemExplorer.resultPanel:SetPosition(itemExplorer.borderWidth,rowTop+controlHeight+itemExplorer.controlVerticalSep)
	itemExplorer.resultPanel:SetSize(itemExplorer:GetWidth()-itemExplorer.borderWidth*2,itemExplorer.searchButton:GetTop()-itemExplorer.controlVerticalSep-itemExplorer.resultPanel:GetTop())
	itemExplorer.resultList:SetSize(itemExplorer.resultPanel:GetWidth()-12,itemExplorer.resultPanel:GetHeight()-2)
	itemExplorer.resultList:SetMaxColumns(math.floor(itemExplorer.resultList:GetWidth()/300))

	itemExplorer.resultVScroll:SetLeft(itemExplorer.resultList:GetWidth()+1)
	itemExplorer.resultVScroll:SetHeight(itemExplorer.resultList:GetHeight())
	itemExplorer.hourglass:SetPosition(itemExplorer.resultPanel:GetLeft()+itemExplorer.resultPanel:GetWidth()/2,itemExplorer.resultPanel:GetTop()+itemExplorer.resultPanel:GetHeight()/2)
end

-- finally, set initial size and position
do
	local width=800
	local height=800
	if Settings.itemExplorerWidth~=nil then
		width=math.floor(Settings.itemExplorerWidth*displayWidth+.5)
	end
	if Settings.itemExplorerHeight~=nil then
		height=math.floor(Settings.itemExplorerHeight*displayHeight+.5)
	end
	local top=math.floor((displayHeight-height)/2+.5)
	local left=math.floor((displayWidth-width)/2+.5)
	if Settings.itemExplorerTop~=nil then
		top=math.floor(Settings.itemExplorerTop*displayHeight+.5)
	end
	if Settings.itemExplorerLeft~=nil then
		left=math.floor(Settings.itemExplorerLeft*displayWidth+.5)
	end
	itemExplorer:SetSize(width,height)
	itemExplorer:SetPosition(left,top)

	itemExplorer:SetTrimColor()
	itemExplorer:SetBackColor()
	itemExplorer:SetFontColor()
	itemExplorer:SetHeadingsColor()
	itemExplorer:SetListTextColor()
	itemExplorer:SetPanelBackColor()
	itemExplorer:SetFont(Settings.fontFace) -- also calls layout
end
itemExplorer.SizeChanged=function()
	itemExplorer:Layout()
end

itemInfoDetail=Turbine.UI.Lotro.Window()
itemInfoDetail:SetZOrder(2)
itemInfoDetail:SetText(Resource[language][145])
-- itemID
itemInfoDetail.idCapt=Turbine.UI.Label()
itemInfoDetail.idCapt:SetParent(itemInfoDetail)
itemInfoDetail.id=Turbine.UI.Label()
itemInfoDetail.id:SetParent(itemInfoDetail)
itemInfoDetail.id:SetSelectable(true)
-- category
itemInfoDetail.categoryCapt=Turbine.UI.Label()
itemInfoDetail.categoryCapt:SetParent(itemInfoDetail)
itemInfoDetail.category=Turbine.UI.Label()
itemInfoDetail.category:SetParent(itemInfoDetail)
itemInfoDetail.category:SetSelectable(true)
-- name
itemInfoDetail.nameCapt=Turbine.UI.Label()
itemInfoDetail.nameCapt:SetParent(itemInfoDetail)
itemInfoDetail.name=Turbine.UI.Label()
itemInfoDetail.name:SetParent(itemInfoDetail)
itemInfoDetail.name:SetSelectable(true)
-- quality
itemInfoDetail.qualityCapt=Turbine.UI.Label()
itemInfoDetail.qualityCapt:SetParent(itemInfoDetail)
itemInfoDetail.quality=Turbine.UI.Label()
itemInfoDetail.quality:SetParent(itemInfoDetail)
itemInfoDetail.quality:SetSelectable(true)
-- durability
itemInfoDetail.durabilityCapt=Turbine.UI.Label()
itemInfoDetail.durabilityCapt:SetParent(itemInfoDetail)
itemInfoDetail.durability=Turbine.UI.Label()
itemInfoDetail.durability:SetParent(itemInfoDetail)
itemInfoDetail.durability:SetSelectable(true)
-- description - largest field, with autosizing it would be a LOT simpler without this...
itemInfoDetail.descriptionCapt=Turbine.UI.Label()
itemInfoDetail.descriptionCapt:SetParent(itemInfoDetail)
itemInfoDetail.description=Turbine.UI.Label()
itemInfoDetail.description:SetParent(itemInfoDetail)
itemInfoDetail.description:SetSelectable(true)
itemInfoDetail.description:SetMarkupEnabled(true)
-- maxQuantity
itemInfoDetail.maxQuantityCapt=Turbine.UI.Label()
itemInfoDetail.maxQuantityCapt:SetParent(itemInfoDetail)
itemInfoDetail.maxQuantity=Turbine.UI.Label()
itemInfoDetail.maxQuantity:SetParent(itemInfoDetail)
itemInfoDetail.maxQuantity:SetSelectable(true)
-- maxStackSize
itemInfoDetail.maxStackSizeCapt=Turbine.UI.Label()
itemInfoDetail.maxStackSizeCapt:SetParent(itemInfoDetail)
itemInfoDetail.maxStackSize=Turbine.UI.Label()
itemInfoDetail.maxStackSize:SetParent(itemInfoDetail)
itemInfoDetail.maxStackSize:SetSelectable(true)
-- iconImageID
itemInfoDetail.iconImageIDCapt=Turbine.UI.Label()
itemInfoDetail.iconImageIDCapt:SetParent(itemInfoDetail)
itemInfoDetail.iconImageIDCapt:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.iconImageID=Turbine.UI.Label()
itemInfoDetail.iconImageID:SetParent(itemInfoDetail)
itemInfoDetail.iconImageID:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.iconImageID:SetSelectable(true)
itemInfoDetail.iconImageBack=Turbine.UI.Control()
itemInfoDetail.iconImageBack:SetParent(itemInfoDetail)
itemInfoDetail.iconImageBack:SetBackColor(Turbine.UI.Color.Black)
itemInfoDetail.iconImageBack:SetSize(32,32)
itemInfoDetail.iconImage=Turbine.UI.Control()
itemInfoDetail.iconImage:SetParent(itemInfoDetail.iconImageBack)
itemInfoDetail.iconImage:SetSize(32,32)
itemInfoDetail.iconImage:SetBlendMode(Turbine.UI.BlendMode.Overlay)
-- backgroundImageID
itemInfoDetail.backgroundImageIDCapt=Turbine.UI.Label()
itemInfoDetail.backgroundImageIDCapt:SetParent(itemInfoDetail)
itemInfoDetail.backgroundImageIDCapt:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.backgroundImageID=Turbine.UI.Label()
itemInfoDetail.backgroundImageID:SetParent(itemInfoDetail)
itemInfoDetail.backgroundImageID:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.backgroundImageID:SetSelectable(true)
itemInfoDetail.backgroundImageBack=Turbine.UI.Control()
itemInfoDetail.backgroundImageBack:SetParent(itemInfoDetail)
itemInfoDetail.backgroundImageBack:SetBackColor(Turbine.UI.Color.Black)
itemInfoDetail.backgroundImageBack:SetSize(32,32)
itemInfoDetail.backgroundImage=Turbine.UI.Control()
itemInfoDetail.backgroundImage:SetParent(itemInfoDetail.backgroundImageBack)
itemInfoDetail.backgroundImage:SetSize(32,32)
itemInfoDetail.backgroundImage:SetBlendMode(Turbine.UI.BlendMode.Overlay)
-- qualityImageID
itemInfoDetail.qualityImageIDCapt=Turbine.UI.Label()
itemInfoDetail.qualityImageIDCapt:SetParent(itemInfoDetail)
itemInfoDetail.qualityImageIDCapt:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.qualityImageID=Turbine.UI.Label()
itemInfoDetail.qualityImageID:SetParent(itemInfoDetail)
itemInfoDetail.qualityImageID:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.qualityImageID:SetSelectable(true)
itemInfoDetail.qualityImageBack=Turbine.UI.Control()
itemInfoDetail.qualityImageBack:SetParent(itemInfoDetail)
itemInfoDetail.qualityImageBack:SetBackColor(Turbine.UI.Color.Black)
itemInfoDetail.qualityImage=Turbine.UI.Control()
itemInfoDetail.qualityImage:SetParent(itemInfoDetail.qualityImageBack)
itemInfoDetail.qualityImage:SetSize(32,32)
itemInfoDetail.qualityImage:SetBlendMode(Turbine.UI.BlendMode.Overlay)
-- shadowImageID
itemInfoDetail.shadowImageIDCapt=Turbine.UI.Label()
itemInfoDetail.shadowImageIDCapt:SetParent(itemInfoDetail)
itemInfoDetail.shadowImageIDCapt:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.shadowImageID=Turbine.UI.Label()
itemInfoDetail.shadowImageID:SetParent(itemInfoDetail)
itemInfoDetail.shadowImageID:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.shadowImageID:SetSelectable(true)
itemInfoDetail.shadowImageBack=Turbine.UI.Control()
itemInfoDetail.shadowImageBack:SetParent(itemInfoDetail)
itemInfoDetail.shadowImageBack:SetBackColor(Turbine.UI.Color.Black)
itemInfoDetail.shadowImage=Turbine.UI.Control()
itemInfoDetail.shadowImage:SetParent(itemInfoDetail.shadowImageBack)
itemInfoDetail.shadowImage:SetSize(32,32)
itemInfoDetail.shadowImage:SetBlendMode(Turbine.UI.BlendMode.Overlay)
-- underlayImageID
itemInfoDetail.underlayImageIDCapt=Turbine.UI.Label()
itemInfoDetail.underlayImageIDCapt:SetParent(itemInfoDetail)
itemInfoDetail.underlayImageIDCapt:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.underlayImageID=Turbine.UI.Label()
itemInfoDetail.underlayImageID:SetParent(itemInfoDetail)
itemInfoDetail.underlayImageID:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
itemInfoDetail.underlayImageID:SetSelectable(true)
itemInfoDetail.underlayImageBack=Turbine.UI.Control()
itemInfoDetail.underlayImageBack:SetParent(itemInfoDetail)
itemInfoDetail.underlayImageBack:SetBackColor(Turbine.UI.Color.Black)
itemInfoDetail.underlayImage=Turbine.UI.Control()
itemInfoDetail.underlayImage:SetParent(itemInfoDetail.underlayImageBack)
itemInfoDetail.underlayImage:SetSize(32,32)
itemInfoDetail.underlayImage:SetBlendMode(Turbine.UI.BlendMode.Overlay)
-- isMagic - note, NONE of the items currently return true for isMagic. not sure why
itemInfoDetail.magicCapt=Turbine.UI.Label()
itemInfoDetail.magicCapt:SetParent(itemInfoDetail)
itemInfoDetail.magic=Turbine.UI.Label()
itemInfoDetail.magic:SetParent(itemInfoDetail)
itemInfoDetail.magic:SetSelectable(true)
-- isUnique
itemInfoDetail.uniqueCapt=Turbine.UI.Label()
itemInfoDetail.uniqueCapt:SetParent(itemInfoDetail)
itemInfoDetail.unique=Turbine.UI.Label()
itemInfoDetail.unique:SetParent(itemInfoDetail)
itemInfoDetail.unique:SetSelectable(true)
itemInfoDetail:SetWidth(500)

itemInfoDetail.ShowItem=function(sender,itemID)
	local font=Settings.fontFace
	local fontSize=Turbine.UI.Lotro.FontInfo[font].size+2
	local tmpItemInfo=getItemInfo(itemID)
	fontMetric:SetFont(font)
	local imageTextWidth=fontMetric:GetTextWidth("0x88888888 ")
	local captionWidth=fontMetric:GetTextWidth(Resource[language][146])
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][19])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][72])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][20])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][144])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][147])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][148])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][149])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][150])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][151])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][152])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][153])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][154])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][155])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local tmpWidth=fontMetric:GetTextWidth(Resource[language][156])
	if tmpWidth>captionWidth then captionWidth=tmpWidth end
	local valueWidth=itemInfoDetail:GetWidth()-25-captionWidth
-- resize and relayout all controls and populate text, basically, rebuild window
-- itemID
	itemInfoDetail.idCapt:SetFont(Settings.fontFace)
	itemInfoDetail.idCapt:SetPosition(10,45)
	itemInfoDetail.idCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.idCapt:SetText(Resource[language][146])
	itemInfoDetail.id:SetFont(Settings.fontFace)
	itemInfoDetail.id:SetPosition(captionWidth+15,45)
	itemInfoDetail.id:SetSize(valueWidth,fontSize)
	itemInfoDetail.id:SetText(string.format("0x%x",itemID))
-- category
	itemInfoDetail.categoryCapt:SetFont(Settings.fontFace)
	itemInfoDetail.categoryCapt:SetPosition(10,itemInfoDetail.idCapt:GetTop()+itemInfoDetail.idCapt:GetHeight()+5)
	itemInfoDetail.categoryCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.categoryCapt:SetText(Resource[language][19])
	itemInfoDetail.category:SetFont(Settings.fontFace)
	itemInfoDetail.category:SetPosition(captionWidth+15,itemInfoDetail.categoryCapt:GetTop())
	itemInfoDetail.category:SetSize(valueWidth,fontSize)
	local tmpVal=tmpItemInfo:GetCategory()
	local tmpCatDesc=""
	for k,v in pairs(ItemCategory) do
		if v[1]==tmpVal then
			tmpCatDesc=v[2][language]
			break
		end
	end
	itemInfoDetail.category:SetText(tmpVal.." "..tmpCatDesc)
-- name
	itemInfoDetail.nameCapt:SetFont(Settings.fontFace)
	itemInfoDetail.nameCapt:SetPosition(10,itemInfoDetail.categoryCapt:GetTop()+itemInfoDetail.categoryCapt:GetHeight()+5)
	itemInfoDetail.nameCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.nameCapt:SetText(Resource[language][72])
	itemInfoDetail.name:SetFont(Settings.fontFace)
	itemInfoDetail.name:SetPosition(captionWidth+15,itemInfoDetail.nameCapt:GetTop())
	itemInfoDetail.name:SetSize(valueWidth,fontSize)
	itemInfoDetail.name:SetText(tmpItemInfo:GetName())
-- quality
	itemInfoDetail.qualityCapt:SetFont(Settings.fontFace)
	itemInfoDetail.qualityCapt:SetPosition(10,itemInfoDetail.nameCapt:GetTop()+itemInfoDetail.nameCapt:GetHeight()+5)
	itemInfoDetail.qualityCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.qualityCapt:SetText(Resource[language][20])
	itemInfoDetail.quality:SetFont(Settings.fontFace)
	itemInfoDetail.quality:SetPosition(captionWidth+15,itemInfoDetail.qualityCapt:GetTop())
	itemInfoDetail.quality:SetSize(valueWidth,fontSize)
	tmpVal=tmpItemInfo:GetQuality()
	itemInfoDetail.quality:SetText(tmpVal)
-- durability
	itemInfoDetail.durabilityCapt:SetFont(Settings.fontFace)
	itemInfoDetail.durabilityCapt:SetPosition(10,itemInfoDetail.qualityCapt:GetTop()+itemInfoDetail.qualityCapt:GetHeight()+5)
	itemInfoDetail.durabilityCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.durabilityCapt:SetText(Resource[language][144])
	itemInfoDetail.durability:SetFont(Settings.fontFace)
	itemInfoDetail.durability:SetPosition(captionWidth+15,itemInfoDetail.durabilityCapt:GetTop())
	itemInfoDetail.durability:SetSize(valueWidth,fontSize)
	tmpVal=tmpItemInfo:GetDurability()
	itemInfoDetail.durability:SetText(tmpVal)
-- description - largest field, with autosizing it would be a LOT simpler without this...
	itemInfoDetail.descriptionCapt:SetFont(Settings.fontFace)
	itemInfoDetail.descriptionCapt:SetPosition(10,itemInfoDetail.durabilityCapt:GetTop()+itemInfoDetail.durabilityCapt:GetHeight()+5)
	itemInfoDetail.descriptionCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.descriptionCapt:SetText(Resource[language][147])
	itemInfoDetail.description:SetFont(Settings.fontFace)
	itemInfoDetail.description:SetPosition(captionWidth+15,itemInfoDetail.descriptionCapt:GetTop())
	tmpVal=tmpItemInfo:GetDescription()
	tmpHeight=fontMetric:GetTextHeight(tmpVal,valueWidth)
	local rowHeight=32
	if fontSize>rowHeight then rowHeight=fontSize end
	local maxHeight=displayHeight-itemInfoDetail.description:GetTop()-90-rowHeight*5-fontSize*4
	if tmpHeight>maxHeight then tmpHeight=maxHeight end
	itemInfoDetail.description:SetSize(valueWidth,tmpHeight)
	itemInfoDetail.description:SetText(tmpVal)
-- maxQuantity
	itemInfoDetail.maxQuantityCapt:SetFont(Settings.fontFace)
	itemInfoDetail.maxQuantityCapt:SetPosition(10,itemInfoDetail.description:GetTop()+itemInfoDetail.description:GetHeight()+5)
	itemInfoDetail.maxQuantityCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.maxQuantityCapt:SetText(Resource[language][148])
	itemInfoDetail.maxQuantity:SetFont(Settings.fontFace)
	itemInfoDetail.maxQuantity:SetPosition(captionWidth+15,itemInfoDetail.maxQuantityCapt:GetTop())
	itemInfoDetail.maxQuantity:SetSize(valueWidth,fontSize)
	itemInfoDetail.maxQuantity:SetText(tmpItemInfo:GetMaxQuantity())
-- maxStackSize
	itemInfoDetail.maxStackSizeCapt:SetFont(Settings.fontFace)
	itemInfoDetail.maxStackSizeCapt:SetPosition(10,itemInfoDetail.maxQuantityCapt:GetTop()+itemInfoDetail.maxQuantityCapt:GetHeight()+5)
	itemInfoDetail.maxStackSizeCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.maxStackSizeCapt:SetText(Resource[language][149])
	itemInfoDetail.maxStackSize:SetFont(Settings.fontFace)
	itemInfoDetail.maxStackSize:SetPosition(captionWidth+15,itemInfoDetail.maxStackSizeCapt:GetTop())
	itemInfoDetail.maxStackSize:SetSize(valueWidth,fontSize)
	itemInfoDetail.maxStackSize:SetText(tmpItemInfo:GetMaxStackSize())
-- iconImageID
	itemInfoDetail.iconImageIDCapt:SetFont(Settings.fontFace)
	itemInfoDetail.iconImageIDCapt:SetPosition(10,itemInfoDetail.maxStackSizeCapt:GetTop()+itemInfoDetail.maxStackSizeCapt:GetHeight()+5)
	itemInfoDetail.iconImageIDCapt:SetSize(captionWidth,rowHeight)
	itemInfoDetail.iconImageIDCapt:SetText(Resource[language][150])
	itemInfoDetail.iconImageID:SetFont(Settings.fontFace)
	itemInfoDetail.iconImageID:SetPosition(captionWidth+15,itemInfoDetail.iconImageIDCapt:GetTop())
	itemInfoDetail.iconImageID:SetSize(imageTextWidth,rowHeight)
	tmpVal=tmpItemInfo:GetIconImageID()
	if tmpVal~=nil then
		itemInfoDetail.iconImageID:SetText(string.format("0x%x",tmpVal))
	else
		itemInfoDetail.iconImageID:SetText("")
	end
	if rowHeight>32 then
		itemInfoDetail.iconImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.iconImageIDCapt:GetTop()+(rowHeight-32)/2)
	else
		itemInfoDetail.iconImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.iconImageIDCapt:GetTop())
	end
	if tmpVal~=nil and tmpVal>0 then
		itemInfoDetail.iconImage:SetBackground(tmpVal)
	else
		itemInfoDetail.iconImage:SetBackground(nil)
	end
-- backgroundImageID
	if fontSize>rowHeight then rowHeight=fontSize end
	itemInfoDetail.backgroundImageIDCapt:SetFont(Settings.fontFace)
	itemInfoDetail.backgroundImageIDCapt:SetPosition(10,itemInfoDetail.iconImageIDCapt:GetTop()+itemInfoDetail.iconImageIDCapt:GetHeight()+5)
	itemInfoDetail.backgroundImageIDCapt:SetSize(captionWidth,rowHeight)
	itemInfoDetail.backgroundImageIDCapt:SetText(Resource[language][151])
	itemInfoDetail.backgroundImageID:SetFont(Settings.fontFace)
	itemInfoDetail.backgroundImageID:SetPosition(captionWidth+15,itemInfoDetail.backgroundImageIDCapt:GetTop())
	itemInfoDetail.backgroundImageID:SetSize(imageTextWidth,rowHeight)
	tmpVal=tmpItemInfo:GetBackgroundImageID()
	if tmpVal~=nil then
		itemInfoDetail.backgroundImageID:SetText(string.format("0x%x",tmpVal))
	else
		itemInfoDetail.backgroundImageID:SetText("")
	end
	if rowHeight>32 then
		itemInfoDetail.backgroundImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.backgroundImageIDCapt:GetTop()+(rowHeight-32)/2)
	else
		itemInfoDetail.backgroundImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.backgroundImageIDCapt:GetTop())
	end
	if tmpVal~=nil and tmpVal>0 then
		itemInfoDetail.backgroundImage:SetBackground(tmpVal)
	else
		itemInfoDetail.backgroundImage:SetBackground(nil)
	end
-- qualityImageID
	if fontSize>rowHeight then rowHeight=fontSize end
	itemInfoDetail.qualityImageIDCapt:SetFont(Settings.fontFace)
	itemInfoDetail.qualityImageIDCapt:SetPosition(10,itemInfoDetail.backgroundImageIDCapt:GetTop()+itemInfoDetail.backgroundImageIDCapt:GetHeight()+5)
	itemInfoDetail.qualityImageIDCapt:SetSize(captionWidth,rowHeight)
	itemInfoDetail.qualityImageIDCapt:SetText(Resource[language][152])
	itemInfoDetail.qualityImageID:SetFont(Settings.fontFace)
	itemInfoDetail.qualityImageID:SetPosition(captionWidth+15,itemInfoDetail.qualityImageIDCapt:GetTop())
	itemInfoDetail.qualityImageID:SetSize(imageTextWidth,rowHeight)
	tmpVal=tmpItemInfo:GetQualityImageID()
	if tmpVal~=nil then
		itemInfoDetail.qualityImageID:SetText(string.format("0x%x",tmpVal))
	else
		itemInfoDetail.qualityImageID:SetText("")
	end
	if rowHeight>32 then
		itemInfoDetail.qualityImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.qualityImageIDCapt:GetTop()+(rowHeight-32)/2)
	else
		itemInfoDetail.qualityImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.qualityImageIDCapt:GetTop())
	end
	if tmpVal~=nil and tmpVal>0 then
		itemInfoDetail.qualityImage:SetBackground(tmpVal)
	else
		itemInfoDetail.qualityImage:SetBackground(nil)
	end
-- shadowImageID
	if fontSize>rowHeight then rowHeight=fontSize end
	itemInfoDetail.shadowImageIDCapt:SetFont(Settings.fontFace)
	itemInfoDetail.shadowImageIDCapt:SetPosition(10,itemInfoDetail.qualityImageIDCapt:GetTop()+itemInfoDetail.qualityImageIDCapt:GetHeight()+5)
	itemInfoDetail.shadowImageIDCapt:SetSize(captionWidth,rowHeight)
	itemInfoDetail.shadowImageIDCapt:SetText(Resource[language][153])
	itemInfoDetail.shadowImageID:SetFont(Settings.fontFace)
	itemInfoDetail.shadowImageID:SetPosition(captionWidth+15,itemInfoDetail.shadowImageIDCapt:GetTop())
	itemInfoDetail.shadowImageID:SetSize(imageTextWidth,rowHeight)
	tmpVal=tmpItemInfo:GetShadowImageID()
	if tmpVal~=nil then
		itemInfoDetail.shadowImageID:SetText(string.format("0x%x",tmpVal))
	else
		itemInfoDetail.shadowImageID:SetText("")
	end
	if rowHeight>32 then
		itemInfoDetail.shadowImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.shadowImageIDCapt:GetTop()+(rowHeight-32)/2)
	else
		itemInfoDetail.shadowImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.shadowImageIDCapt:GetTop())
	end
	if tmpVal~=nil and tmpVal>0 then
		itemInfoDetail.shadowImage:SetBackground(tmpVal)
	else
		itemInfoDetail.shadowImage:SetBackground(nil)
	end
-- underlayImageID
	if fontSize>rowHeight then rowHeight=fontSize end
	itemInfoDetail.underlayImageIDCapt:SetFont(Settings.fontFace)
	itemInfoDetail.underlayImageIDCapt:SetPosition(10,itemInfoDetail.shadowImageIDCapt:GetTop()+itemInfoDetail.shadowImageIDCapt:GetHeight()+5)
	itemInfoDetail.underlayImageIDCapt:SetSize(captionWidth,rowHeight)
	itemInfoDetail.underlayImageIDCapt:SetText(Resource[language][154])
	itemInfoDetail.underlayImageID:SetFont(Settings.fontFace)
	itemInfoDetail.underlayImageID:SetPosition(captionWidth+15,itemInfoDetail.underlayImageIDCapt:GetTop())
	itemInfoDetail.underlayImageID:SetSize(imageTextWidth,rowHeight)
	tmpVal=tmpItemInfo:GetUnderlayImageID()
	if tmpVal~=nil then
		itemInfoDetail.underlayImageID:SetText(string.format("0x%x",tmpVal))
	else
		itemInfoDetail.underlayImageID:SetText("")
	end
	if rowHeight>32 then
		itemInfoDetail.underlayImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.underlayImageIDCapt:GetTop()+(rowHeight-32)/2)
	else
		itemInfoDetail.underlayImageBack:SetPosition(captionWidth+15+imageTextWidth,itemInfoDetail.underlayImageIDCapt:GetTop())
	end
	if tmpVal~=nil and tmpVal>0 then
		itemInfoDetail.underlayImage:SetBackground(tmpVal)
	else
		itemInfoDetail.underlayImage:SetBackground(nil)
	end
-- isMagic - note, NONE of the items currently return true for isMagic. not sure why
	itemInfoDetail.magicCapt:SetFont(Settings.fontFace)
	itemInfoDetail.magicCapt:SetPosition(10,itemInfoDetail.underlayImageIDCapt:GetTop()+itemInfoDetail.underlayImageIDCapt:GetHeight()+5)
	itemInfoDetail.magicCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.magicCapt:SetText(Resource[language][155])
	itemInfoDetail.magic:SetFont(Settings.fontFace)
	itemInfoDetail.magic:SetPosition(captionWidth+15,itemInfoDetail.magicCapt:GetTop())
	itemInfoDetail.magic:SetSize(valueWidth,fontSize)
	local tmpVal=tmpItemInfo:GetMaxStackSize()
	itemInfoDetail.magic:SetText(tmpVal)
-- isUnique
	itemInfoDetail.uniqueCapt:SetFont(Settings.fontFace)
	itemInfoDetail.uniqueCapt:SetPosition(10,itemInfoDetail.magicCapt:GetTop()+itemInfoDetail.magicCapt:GetHeight()+5)
	itemInfoDetail.uniqueCapt:SetSize(captionWidth,fontSize)
	itemInfoDetail.uniqueCapt:SetText(Resource[language][156])
	itemInfoDetail.unique:SetFont(Settings.fontFace)
	itemInfoDetail.unique:SetPosition(captionWidth+15,itemInfoDetail.uniqueCapt:GetTop())
	itemInfoDetail.unique:SetSize(valueWidth,fontSize)
	local tmpVal=tmpItemInfo:GetMaxStackSize()
	itemInfoDetail.unique:SetText(tmpVal)

	itemInfoDetail:SetHeight(itemInfoDetail.uniqueCapt:GetTop()+fontSize+45)
	itemInfoDetail:SetVisible(true)
end
