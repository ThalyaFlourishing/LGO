fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
margin=10
criteriaMenu=Turbine.UI.ContextMenu()

-- this file is the Group Maintenance dialog
groupMaint=Turbine.UI.Lotro.Window()
groupMaint:SetZOrder(1)
groupMaint:SetOpacity(1)
if displayHeight<800 then
	groupMaint:SetSize(600,displayHeight)
else
	groupMaint:SetSize(600,800)
end
groupMaint:SetText(Resource[language][56])
groupMaint:SetPosition((displayWidth-groupMaint:GetWidth())/2,(displayHeight-groupMaint:GetHeight())/2)

groupMaint.loadDefaults=Turbine.UI.Lotro.Button()
groupMaint.loadDefaults:SetParent(groupMaint)
groupMaint.loadDefaults:SetPosition(margin,45)
groupMaint.loadDefaults:SetSize(200,20)
groupMaint.loadDefaults:SetText(Resource[language][129])
groupMaint.loadDefaults:SetEnabled(defaultTabs~=nil) -- only enabled if default tabs were previously saved
groupMaint.loadDefaults.Click=function(sender,args)
	-- this is a destructive copy, confirm that the user wants to overwrite all tab settings
	copyDefaultTabs()
end
getDTCopy=function(src)
	-- for some reason, the Table.Copy function is not working as anticipated, but the brute force manual copy works fine
	local ret={}
	if src~=nil and type(src)=="table" then
		-- seem to have a valid dt, copy the primary attributes
		ret.isMain=src.isMain
		ret.docked=src.docked
		ret.expanded=src.expanded
		ret.name=src.name
		ret.top=src.top
		ret.left=src.left
		ret.width=src.width
		ret.criteria={}
		if src.criteria~=nil and type(src.criteria)=="table" then
			ret.criteria.ItemNames={}
			ret.criteria.ItemInfoNames={}
			ret.criteria.CIDValues={}
			ret.criteria.UIDValues={} -- not currently used but defined for future use
			ret.criteria.GIDValues={} -- not currently used
			for k,v in pairs(src.criteria.ItemNames) do
				ret.criteria.ItemNames[k]=v
			end
			for k,v in pairs(src.criteria.ItemInfoNames) do
				ret.criteria.ItemInfoNames[k]=v
			end
			for k,v in pairs(src.criteria.CIDValues) do
				ret.criteria.CIDValues[k]=v
			end
		end
	end
	return ret
end
copyDefaultTabs=function()
	local container=getCurrentContainer()
	destroyOldItemEntries()

	-- copy defaultTabs to displayTabs
	displayTabs={}
	for k,v in pairs(defaultTabs) do
		if type(v)~="table" then
			displayTabs[k]=v
		else
			displayTabs[k]={}
			for index,dt in ipairs(v) do
				displayTabs[k][index]=getDTCopy(dt)
			end
		end
	end
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	groupMaint:Show(container)	
end

groupMaint.saveDefaults=Turbine.UI.Lotro.Button()
groupMaint.saveDefaults:SetParent(groupMaint)
groupMaint.saveDefaults:SetPosition(groupMaint:GetWidth()-200-margin,45)
groupMaint.saveDefaults:SetSize(200,20)
groupMaint.saveDefaults:SetText(Resource[language][130])
groupMaint.saveDefaults.Click=function(sender,args)
	-- copy the current displaytTabs setup to defaultTabs
	defaultTabs={}
	for k,v in pairs(displayTabs) do
		if type(v)~="table" then
			defaultTabs[k]=v
		else
			defaultTabs[k]={}
			for index, dt in ipairs(v) do
				defaultTabs[k][index]=getDTCopy(dt)
			end
		end
	end
	groupMaint.loadDefaults:SetEnabled(true)
	setGroupMaintMessage(Resource[language][86],Turbine.UI.Color(1,.4,1,.4))
end

groupMaint.sep1=Turbine.UI.Control()
groupMaint.sep1:SetParent(groupMaint)
groupMaint.sep1:SetSize(groupMaint:GetWidth()-margin*2,1)
groupMaint.sep1:SetPosition(margin,groupMaint.saveDefaults:GetTop()+groupMaint.saveDefaults:GetHeight()+7)
groupMaint.sep1:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.containerCaption=Turbine.UI.Label()
groupMaint.containerCaption:SetParent(groupMaint)
groupMaint.containerCaption:SetSize(200,fontSize)
groupMaint.containerCaption:SetPosition(margin,groupMaint.sep1:GetTop()+groupMaint.sep1:GetHeight()+7)
groupMaint.containerCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.containerCaption:SetFont(Settings.fontFace)
groupMaint.containerCaption:SetText(Resource[language][122]..":")

groupMaint.container=Turbine.UI.Label()
groupMaint.container:SetParent(groupMaint)
groupMaint.container:SetPosition(groupMaint.containerCaption:GetLeft()+groupMaint.containerCaption:GetWidth(),groupMaint.containerCaption:GetTop())
groupMaint.container:SetSize(200,fontSize)
groupMaint.container:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.container:SetFont(Settings.fontFace)

groupMaint.copyGroups=Turbine.UI.Lotro.Button()
groupMaint.copyGroups:SetParent(groupMaint)
groupMaint.copyGroups:SetPosition(groupMaint.container:GetLeft()+groupMaint.container:GetWidth(),groupMaint.container:GetTop())
groupMaint.copyGroups:SetSize(groupMaint:GetWidth()-groupMaint.copyGroups:GetLeft()-margin,20)
groupMaint.copyGroups:SetText(Resource[language][125])
groupMaint.copyGroups.Click=function(sender,args)
	-- create popup with callbacks to copy one of the other containers based on current container
	local container=getCurrentContainer(true)
	if container=="all" then --
		-- copy from inventory (7), vault (8), shared (9)
		PopUpDialog(Resource[language][125],string.gsub(Resource[language][126],"@container",Resource[language][12]),1,Resource[language][7],Resource[language][8],Resource[language][9],false,copyBagGroups,copyVaultGroups,copySharedGroups)
	elseif container=="vault" then --
		-- copy from inventory (7), shared (9), all (12)
		PopUpDialog(Resource[language][125],string.gsub(Resource[language][126],"@container",Resource[language][8]),1,Resource[language][7],Resource[language][9],Resource[language][12],false,copyBagGroups,copySharedGroups,copyAllGroups)
	elseif container=="shared" then --
		-- copy from inventory (7), vault (8), all (12)
		PopUpDialog(Resource[language][125],string.gsub(Resource[language][126],"@container",Resource[language][9]),1,Resource[language][7],Resource[language][8],Resource[language][12],false,copyBagGroups,copyVaultGroups,copyAllGroups)
	else --
		-- copy from vault (8), shared (9), all (12)
		PopUpDialog(Resource[language][125],string.gsub(Resource[language][126],"@container",Resource[language][12]),1,Resource[language][8],Resource[language][9],Resource[language][12],false,copyVaultGroups,copySharedGroups,copyAllGroups)
	end
end
getDTCopy=function(src)
	-- for some reason, the Table.Copy function is not working as anticipated, but the brute force manual copy works fine
	local ret={}
	if src~=nil and type(src)=="table" then
		-- seem to have a valid dt, copy the primary attributes
		ret.isMain=src.isMain
		ret.docked=src.docked
		ret.expanded=src.expanded
		ret.name=src.name
		ret.criteria={}
		if src.criteria~=nil and type(src.criteria)=="table" then
			ret.criteria.ItemNames={}
			ret.criteria.ItemInfoNames={}
			ret.criteria.CIDValues={}
			ret.criteria.UIDValues={} -- not currently used but defined for future use
			ret.criteria.GIDValues={} -- not currently used
			for k,v in pairs(src.criteria.ItemNames) do
				ret.criteria.ItemNames[k]=v
			end
			for k,v in pairs(src.criteria.ItemInfoNames) do
				ret.criteria.ItemInfoNames[k]=v
			end
			for k,v in pairs(src.criteria.CIDValues) do
				ret.criteria.CIDValues[k]=v
			end
		end
	end
	return ret
end

-- create four seperate functions because the callback feature of the popup doesn't handle passing parameters very well
copyBagGroups=function()
	local container=getCurrentContainer(true)
	-- clear displayTabs[container] and then copy all the groups everything from displayTabs.bags to displayTabs[container]
	destroyOldItemEntries()
	displayTabs[container]={} -- effectively clears prior table
	if displayTabs.bags~=nil then
		for k,v in ipairs(displayTabs.bags) do
			displayTabs[container][k]=getDTCopy(v)
		end
	end
	-- just monkeyed with the displayTabs, need to regenerate the Xref
	updateDisplayTabXref()
	--now need to reload the inventory panels to reflect the changed grouping
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
	-- need to re show groupMaint
	groupMaint:Show(container)
end
copyVaultGroups=function()
	local container=getCurrentContainer(true)
	-- clear displayTabs[container] and then copy all the groups everything from displayTabs.bags to displayTabs[container]
	destroyOldItemEntries()
	displayTabs[container]={} -- effectively clears prior table
	if displayTabs.vault~=nil then
		for k,v in ipairs(displayTabs.vault) do
			displayTabs[container][k]=getDTCopy(v)
		end
	end
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
	groupMaint:Show(container)
end
copySharedGroups=function()
	local container=getCurrentContainer(true)
	-- clear displayTabs[container] and then copy all the groups everything from displayTabs.bags to displayTabs[container]
	destroyOldItemEntries()
	displayTabs[container]={} -- effectively clears prior table
	if displayTabs.shared~=nil then
		for k,v in ipairs(displayTabs.shared) do
			displayTabs[container][k]=getDTCopy(v)
		end
	end
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
	groupMaint:Show(container)
end
copyAllGroups=function()
	local container=getCurrentContainer(true)
	-- clear displayTabs[container] and then copy all the groups everything from displayTabs.bags to displayTabs[container]
	destroyOldItemEntries()
	displayTabs[container]={} -- effectively clears prior table
	if displayTabs.all~=nil then
		for k,v in ipairs(displayTabs.all) do
			displayTabs[container][k]=getDTCopy(v)
		end
	end
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
	groupMaint:Show(container)
end
groupMaint.useBagsCB=Turbine.UI.Lotro.CheckBox()
groupMaint.useBagsCB:SetParent(groupMaint)
groupMaint.useBagsCB:SetSize(groupMaint:GetWidth()-groupMaint.container:GetLeft(),fontSize)
groupMaint.useBagsCB:SetPosition(groupMaint.container:GetLeft(),groupMaint.container:GetTop()+groupMaint.container:GetHeight()+5)
groupMaint.useBagsCB:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.useBagsCB:SetFont(Settings.fontFace)
groupMaint.useBagsCB:SetText(Resource[language][123])
groupMaint.useBagsCB.CheckedChanged=function()
	if not groupMaint.loading then
		--save the change and apply
		local container=getCurrentContainer(true)
		displayTabs[container].useBagTags=groupMaint.useBagsCB:IsChecked()
		updateDisplayTabXref()
		inventoryPanel:Refresh()
		getItemEntryLayout()
		applyItemEntryLayout()
		inventoryPanel:Layout()
		groupMaint.loading=false
		inventoryPanel.cropDelay=Settings.cropDelay
		inventoryPanel:SetWantsUpdates(true)
		groupMaint:Show()
	end
end

-- selection list? set default to Resource[language][70]?
groupMaint.groupCaption=Turbine.UI.Label()
groupMaint.groupCaption:SetParent(groupMaint)
groupMaint.groupCaption:SetSize(200,fontSize)
groupMaint.groupCaption:SetPosition(margin,groupMaint.useBagsCB:GetTop()+groupMaint.useBagsCB:GetHeight()+5)
groupMaint.groupCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.groupCaption:SetFont(Settings.fontFace)
groupMaint.groupCaption:SetText(Resource[language][71]..":")

groupMaint.groupSelect=DropDownList()
groupMaint.groupSelect:SetParent(groupMaint)
groupMaint.groupSelect:SetPosition(margin+groupMaint.groupCaption:GetWidth(),groupMaint.groupCaption:GetTop())
groupMaint.groupSelect:SetSize(groupMaint:GetWidth()-groupMaint.groupSelect:GetLeft()-10,fontSize+2)
groupMaint.groupSelect:SetDropRows(10)
groupMaint.groupSelect:SetBackColor(Turbine.UI.Color.Black)
groupMaint.groupSelect:SetSelectedIndex(1)
groupMaint.groupSelect:SetZOrder(1)
groupMaint.groupSelect:SetFont(Settings.fontFace)
updateGroupMaint=function(index)
	local selValue=groupMaint.groupSelect:GetValue()
	if index==selValue then
		local container=getCurrentContainer()
		local dt=displayTabs[container][index]
		-- the user did something to the actual window, reflect it here
		groupMaint.dockedCB:SetChecked(dt.docked)
		groupMaint.expandedCB:SetChecked(dt.expanded)
		groupMaint.left:SetText(dt.left)
		groupMaint.top:SetText(dt.top)
		groupMaint.width:SetText(dt.width)
		groupMaint.maxHeightCB:SetChecked(dt.lockHeight)
		groupMaint.maxHeight:SetText(dt.height)
	end
end
groupMaint.groupSelect.SelectedIndexChanged=function()
	local index=groupMaint.groupSelect:GetSelectedIndex()
	-- clear the criteria boxes
	groupMaint.crit1.listBox:ClearItems()
	groupMaint.crit2.listBox:ClearItems()
	for k=1,groupMaint.crit3.listBox:GetItemCount() do
		groupMaint.crit3.listBox:GetItem(k):SetChecked(false)
	end
	groupMaint.sortCrit3()

	if index==1 then
		groupMaint.SaveButton:SetText(Resource[language][66])
		groupMaint.name:SetText("")
		groupMaint.moveTop:SetEnabled(false)
		groupMaint.moveUp:SetEnabled(false)
		groupMaint.moveDown:SetEnabled(false)
		groupMaint.moveBottom:SetEnabled(false)
		groupMaint.mainLabel:SetVisible(false)
		groupMaint.dockedCB:SetChecked(true)
		groupMaint.undockedAttrib:SetVisible(false)
		groupMaint.left:SetText("")
		groupMaint.top:SetText("")
		groupMaint.width:SetText("")
		groupMaint.expandedCB:SetChecked(true)
		groupMaint.crit1:SetVisible(true)
		groupMaint.crit2:SetVisible(true)
		groupMaint.crit3:SetVisible(true)
	else
		local container=getCurrentContainer()
		local dt=displayTabs[container][index-1]
		if index==2 then
			groupMaint.moveTop:SetEnabled(false)
			groupMaint.moveUp:SetEnabled(false)
			groupMaint.moveDown:SetEnabled(true)
			groupMaint.moveBottom:SetEnabled(true)
		elseif index==groupMaint.groupSelect.ListData:GetItemCount() then
			groupMaint.moveTop:SetEnabled(true)
			groupMaint.moveUp:SetEnabled(true)
			groupMaint.moveDown:SetEnabled(false)
			groupMaint.moveBottom:SetEnabled(false)
		else
			groupMaint.moveTop:SetEnabled(true)
			groupMaint.moveUp:SetEnabled(true)
			groupMaint.moveDown:SetEnabled(true)
			groupMaint.moveBottom:SetEnabled(true)
		end
		groupMaint.SaveButton:SetText(Resource[language][68])
		groupMaint.name:SetText(dt.name)
		groupMaint.mainLabel:SetVisible(dt.isMain)
		groupMaint.dockedCB:SetChecked(dt.docked)
		groupMaint.undockedAttrib:SetVisible(not dt.docked)
		groupMaint.left:SetText(dt.left)
		groupMaint.top:SetText(dt.top)
		groupMaint.width:SetText(dt.width)
		groupMaint.maxHeightCB:SetChecked(dt.lockHeight)
		groupMaint.maxHeight:SetText(dt.height)
		groupMaint.expandedCB:SetChecked(dt.expanded)
		groupMaint.crit1:SetVisible(not dt.isMain)
		groupMaint.crit2:SetVisible(not dt.isMain)
		groupMaint.crit3:SetVisible(not dt.isMain)

		for k,v in ipairs(dt.criteria.ItemNames) do
			local tmpItem=Turbine.UI.Label()
			tmpItem:SetParent(groupMaint.crit1.listBox)
			tmpItem:SetSize(groupMaint.crit1.listBox:GetWidth(),fontSize)
			tmpItem:SetFont(Settings.fontFace)
			tmpItem:SetText(v)
			tmpItem.name=v
			tmpItem.MouseClick=function(sender,args)
				if args.Button==1 then
					groupMaint.crit1.newText:SetText(sender.name)
				elseif args.Button==2 then
					local menuItems=criteriaMenu:GetItems()
					menuItems:Clear()
					menuItems:Add(Turbine.UI.MenuItem(Resource[language][92]))
					criteriaMenu.item=sender
					menuItems:Get(1).Click=function(sender,args)
						-- remove the selected item
						groupMaint.crit1.listBox:RemoveItem(criteriaMenu.item)
					end
					criteriaMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
				end
			end
			groupMaint.crit1.listBox:AddItem(tmpItem)
		end
		for k,v in ipairs(dt.criteria.ItemInfoNames) do
			local tmpItem=Turbine.UI.Label()
			tmpItem:SetParent(groupMaint.crit2.listBox)
			tmpItem:SetSize(groupMaint.crit2.listBox:GetWidth(),fontSize)
			tmpItem:SetFont(Settings.fontFace)
			tmpItem:SetText(v)
			tmpItem.name=v
			tmpItem.MouseClick=function(sender,args)
				if args.Button==1 then
					groupMaint.crit2.newText:SetText(sender.name)
				elseif args.Button==2 then
					local menuItems=criteriaMenu:GetItems()
					menuItems:Clear()
					menuItems:Add(Turbine.UI.MenuItem(Resource[language][92]))
					criteriaMenu.item=sender
					menuItems:Get(1).Click=function(sender,args)
						-- remove the selected item
						groupMaint.crit2.listBox:RemoveItem(criteriaMenu.item)
					end
					criteriaMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
				end
			end
			groupMaint.crit2.listBox:AddItem(tmpItem)
		end
		for k=1,groupMaint.crit3.listBox:GetItemCount() do
			tmpItem=groupMaint.crit3.listBox:GetItem(k)
			if Table.Contains(dt.criteria.CIDValues,tmpItem.CategoryID,true) then
				tmpItem:SetChecked(true)
			end
		end
		groupMaint.sortCrit3()
	end
end

groupMaint.sep1=Turbine.UI.Control()
groupMaint.sep1:SetParent(groupMaint)
groupMaint.sep1:SetSize(groupMaint:GetWidth()-margin*2,1)
groupMaint.sep1:SetPosition(margin,groupMaint.groupCaption:GetTop()+groupMaint.groupCaption:GetHeight()+5)
groupMaint.sep1:SetBackColor(Turbine.UI.Color.Silver)

-- name
groupMaint.nameCaption=Turbine.UI.Label()
groupMaint.nameCaption:SetParent(groupMaint)
groupMaint.nameCaption:SetSize(groupMaint.groupCaption:GetWidth(),fontSize)
groupMaint.nameCaption:SetPosition(margin,groupMaint.sep1:GetTop()+groupMaint.sep1:GetHeight()+5)
groupMaint.nameCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.nameCaption:SetFont(Settings.fontFace)
groupMaint.nameCaption:SetText(Resource[language][72]..":")

groupMaint.nameBorder=Turbine.UI.Control()
groupMaint.nameBorder:SetParent(groupMaint)
groupMaint.nameBorder:SetSize(groupMaint.groupSelect:GetWidth(),fontSize+2)
groupMaint.nameBorder:SetPosition(groupMaint.groupSelect:GetLeft(),groupMaint.nameCaption:GetTop())
groupMaint.nameBorder:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.name=Turbine.UI.TextBox()
groupMaint.name:SetParent(groupMaint.nameBorder)
groupMaint.name:SetSize(groupMaint.nameBorder:GetWidth()-2,fontSize)
groupMaint.name:SetPosition(1,1)
groupMaint.name:SetFont(Settings.fontFace)
groupMaint.name:SetBackColor(Turbine.UI.Color.Black)

groupMaint.moveTop=Turbine.UI.Lotro.Button()
groupMaint.moveTop:SetParent(groupMaint)
groupMaint.moveTop:SetSize(110,20)
groupMaint.moveTop:SetPosition(margin,groupMaint.nameBorder:GetTop()+groupMaint.nameBorder:GetHeight()+5)
--groupMaint.moveTop:SetBackColor(Turbine.UI.Color.Black)
groupMaint.moveTop:SetText(Resource[language][93])
groupMaint.moveTop.Click=function(sender,args)
	local container=getCurrentContainer()
	local index=groupMaint.groupSelect:GetSelectedIndex()-1
	-- get a handle to the current dt
	local currentDisplayTab=displayTabs[container][index]
	for k=index-1, 1, -1 do
		-- move displayTabs[container][k] to displayTabs[container][k+1]
		displayTabs[container][k+1]=displayTabs[container][k]
	end
	displayTabs[container][1]=currentDisplayTab
	-- now repopulate the select list and reselect the 1st element
	groupMaint:Show(container, currentDisplayTab)
	setGroupMaintMessage(Resource[language][97],Turbine.UI.Color(1,.4,1,.4))
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
end

groupMaint.moveUp=Turbine.UI.Lotro.Button()
groupMaint.moveUp:SetParent(groupMaint)
groupMaint.moveUp:SetSize(groupMaint.moveTop:GetWidth(),20)
groupMaint.moveUp:SetPosition(groupMaint.moveTop:GetLeft()+groupMaint.moveTop:GetWidth()+5,groupMaint.moveTop:GetTop())
--groupMaint.moveUp:SetBlendMode(Turbine.UI.BlendMode.Overlay)
--groupMaint.moveUp:SetBackColor(Turbine.UI.Color.Black)
--0x411105b6,0x411105b7,0x411105b8 (highlight,normal,disabled) 27x28
--0x4112c11c,0x4112be9a,0x4112c234 (highlight,normal,disabled) 19x15
--groupMaint.moveUp:SetBackground(resourcePath.."TabBack_up.tga")
groupMaint.moveUp:SetText(Resource[language][94])
groupMaint.moveUp.Click=function(sender,args)
	local container=getCurrentContainer()
	local index=groupMaint.groupSelect:GetSelectedIndex()-1
	-- get a handle to the current dt
	local currentDisplayTab=displayTabs[container][index]
	displayTabs[container][index]=displayTabs[container][index-1]
	displayTabs[container][index-1]=currentDisplayTab
	-- now repopulate the select list and reselect the 1st element
	groupMaint:Show(container, currentDisplayTab)
	setGroupMaintMessage(Resource[language][97],Turbine.UI.Color(1,.4,1,.4))
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
end

--groupMaint.mainLabel=Turbine.UI.Lotro.CheckBox()
groupMaint.mainLabel=Turbine.UI.Label()
groupMaint.mainLabel:SetParent(groupMaint)
groupMaint.mainLabel:SetSize(groupMaint:GetWidth()-(groupMaint.moveUp:GetLeft()+groupMaint.moveUp:GetWidth()+5)*2,fontSize+2)
groupMaint.mainLabel:SetPosition(groupMaint.moveUp:GetLeft()+groupMaint.moveUp:GetWidth()+5,groupMaint.moveUp:GetTop())
groupMaint.mainLabel:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
groupMaint.mainLabel:SetForeColor(Turbine.UI.Color.Red) -- make "Main" stand out
groupMaint.mainLabel:SetFont(Settings.fontFace)
groupMaint.mainLabel:SetText(Resource[language][73])
groupMaint.mainLabel:SetVisible(false)

groupMaint.moveDown=Turbine.UI.Lotro.Button()
groupMaint.moveDown:SetParent(groupMaint)
groupMaint.moveDown:SetSize(groupMaint.moveTop:GetWidth(),20)
groupMaint.moveDown:SetPosition(groupMaint.mainLabel:GetLeft()+groupMaint.mainLabel:GetWidth()+5,groupMaint.moveTop:GetTop())
--groupMaint.moveDown:SetBackColor(Turbine.UI.Color.Black)
groupMaint.moveDown:SetText(Resource[language][95])
groupMaint.moveDown.Click=function(sender,args)
	local container=getCurrentContainer()
	local index=groupMaint.groupSelect:GetSelectedIndex()-1
	-- get a handle to the current dt
	local currentDisplayTab=displayTabs[container][index]
	displayTabs[container][index]=displayTabs[container][index+1]
	displayTabs[container][index+1]=currentDisplayTab
	-- now repopulate the select list and reselect the 1st element
	groupMaint:Show(container, currentDisplayTab)
	setGroupMaintMessage(Resource[language][97],Turbine.UI.Color(1,.4,1,.4))
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
end

groupMaint.moveBottom=Turbine.UI.Lotro.Button()
groupMaint.moveBottom:SetParent(groupMaint)
groupMaint.moveBottom:SetSize(groupMaint.moveTop:GetWidth(),20)
groupMaint.moveBottom:SetPosition(groupMaint.moveDown:GetLeft()+groupMaint.moveDown:GetWidth()+5,groupMaint.moveTop:GetTop())
--groupMaint.moveBottom:SetBackColor(Turbine.UI.Color.Black)
groupMaint.moveBottom:SetText(Resource[language][96])
groupMaint.moveBottom.Click=function(sender,args)
	local container=getCurrentContainer()
	local index=groupMaint.groupSelect:GetSelectedIndex()-1
	-- get a handle to the current dt
	local currentDisplayTab=displayTabs[container][index]
	local dtCount=#displayTabs[container]
	for k=index, dtCount-1 do
		-- move displayTabs[container][k+1] to displayTabs[container][k]
		displayTabs[container][k]=displayTabs[container][k+1]
	end
	displayTabs[container][dtCount]=currentDisplayTab
	-- now repopulate the select list and reselect the 1st element
	groupMaint:Show(container, currentDisplayTab)
	setGroupMaintMessage(Resource[language][97],Turbine.UI.Color(1,.4,1,.4))
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	inventoryPanel.cropDelay=Settings.cropDelay
	inventoryPanel:SetWantsUpdates(true)
end

--groupMaint
--0x41110a41,0x41110a42,0x41110a43 (disabled,highlight,normal) 27x28
--0x4112c233,0x4112c11e,0x4112be99 (disabled,highlight,normal) 19x15

groupMaint.dockedCB=Turbine.UI.Lotro.CheckBox()
groupMaint.dockedCB:SetParent(groupMaint)
groupMaint.dockedCB:SetSize(groupMaint.groupSelect:GetWidth(),fontSize+2)
groupMaint.dockedCB:SetPosition(groupMaint.groupSelect:GetLeft(),groupMaint.mainLabel:GetTop()+groupMaint.mainLabel:GetHeight()+5)
groupMaint.dockedCB:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.dockedCB:SetFont(Settings.fontFace)
groupMaint.dockedCB:SetText(Resource[language][74])
groupMaint.dockedCB.CheckedChanged=function()
	if groupMaint.dockedCB:IsChecked() then
		groupMaint.undockedAttrib:SetVisible(false)
	else
		groupMaint.undockedAttrib:SetVisible(true)
	end
end

groupMaint.undockedAttrib=Turbine.UI.Control()
groupMaint.undockedAttrib:SetParent(groupMaint)
groupMaint.undockedAttrib:SetPosition(margin,groupMaint.dockedCB:GetTop()+groupMaint.dockedCB:GetHeight()+5)
groupMaint.undockedAttrib:SetSize(groupMaint:GetWidth()-margin*2,fontSize*2+9)
--groupMaint.undockedAttrib:SetBackColor(Turbine.UI.Color.Green)

groupMaint.leftCaption=Turbine.UI.Label()
groupMaint.leftCaption:SetParent(groupMaint.undockedAttrib)
groupMaint.leftCaption:SetSize((groupMaint.undockedAttrib:GetWidth()-margin*2)/3-80,fontSize+2)
groupMaint.leftCaption:SetPosition(0,0)
groupMaint.leftCaption:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.leftCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight)
groupMaint.leftCaption:SetText(Resource[1][75]..": ")

groupMaint.leftBack=Turbine.UI.Control()
groupMaint.leftBack:SetParent(groupMaint.undockedAttrib)
groupMaint.leftBack:SetSize(80,fontSize+2)
groupMaint.leftBack:SetPosition(groupMaint.leftCaption:GetLeft()+groupMaint.leftCaption:GetWidth(),0)
groupMaint.leftBack:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.left=Turbine.UI.TextBox()
groupMaint.left:SetParent(groupMaint.leftBack)
groupMaint.left:SetSize(78,fontSize)
groupMaint.left:SetPosition(1,1)
groupMaint.left:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.left:SetBackColor(Turbine.UI.Color.Black)

groupMaint.topCaption=Turbine.UI.Label()
groupMaint.topCaption:SetParent(groupMaint.undockedAttrib)
groupMaint.topCaption:SetSize(groupMaint.leftCaption:GetSize())
groupMaint.topCaption:SetPosition(groupMaint.leftBack:GetLeft()+groupMaint.leftBack:GetWidth(),0)
groupMaint.topCaption:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.topCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight)
groupMaint.topCaption:SetText(Resource[1][76]..": ")

groupMaint.topBack=Turbine.UI.Control()
groupMaint.topBack:SetParent(groupMaint.undockedAttrib)
groupMaint.topBack:SetSize(80,fontSize+2)
groupMaint.topBack:SetPosition(groupMaint.topCaption:GetLeft()+groupMaint.topCaption:GetWidth(),0)
groupMaint.topBack:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.top=Turbine.UI.TextBox()
groupMaint.top:SetParent(groupMaint.topBack)
groupMaint.top:SetSize(78,fontSize)
groupMaint.top:SetPosition(1,1)
groupMaint.top:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.top:SetBackColor(Turbine.UI.Color.Black)


groupMaint.widthCaption=Turbine.UI.Label()
groupMaint.widthCaption:SetParent(groupMaint.undockedAttrib)
groupMaint.widthCaption:SetSize(groupMaint.leftCaption:GetSize())
groupMaint.widthCaption:SetPosition(groupMaint.topBack:GetLeft()+groupMaint.topBack:GetWidth(),0)
groupMaint.widthCaption:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.widthCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight)
groupMaint.widthCaption:SetText(Resource[1][77]..": ")

groupMaint.widthBack=Turbine.UI.Control()
groupMaint.widthBack:SetParent(groupMaint.undockedAttrib)
groupMaint.widthBack:SetSize(80,fontSize+2)
groupMaint.widthBack:SetPosition(groupMaint.widthCaption:GetLeft()+groupMaint.widthCaption:GetWidth(),0)
groupMaint.widthBack:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.width=Turbine.UI.TextBox()
groupMaint.width:SetParent(groupMaint.widthBack)
groupMaint.width:SetSize(78,fontSize)
groupMaint.width:SetPosition(1,1)
groupMaint.width:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.width:SetBackColor(Turbine.UI.Color.Black)

groupMaint.maxHeightCB=Turbine.UI.Lotro.CheckBox()
groupMaint.maxHeightCB:SetParent(groupMaint.undockedAttrib)
groupMaint.maxHeightCB:SetPosition(groupMaint.groupSelect:GetLeft()-groupMaint.undockedAttrib:GetLeft(),groupMaint.leftCaption:GetTop()+groupMaint.leftCaption:GetHeight()+5)
groupMaint.maxHeightCB:SetSize(groupMaint.undockedAttrib:GetWidth()-groupMaint.maxHeightCB:GetLeft()-80-margin,fontSize+2)
groupMaint.maxHeightCB:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.maxHeightCB:SetFont(Settings.fontFace)
groupMaint.maxHeightCB:SetText(Resource[language][135])

groupMaint.maxHeightBack=Turbine.UI.Control()
groupMaint.maxHeightBack:SetParent(groupMaint.undockedAttrib)
groupMaint.maxHeightBack:SetSize(80,fontSize+2)
groupMaint.maxHeightBack:SetPosition(groupMaint.widthBack:GetLeft(),groupMaint.maxHeightCB:GetTop())
groupMaint.maxHeightBack:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.maxHeight=Turbine.UI.TextBox()
groupMaint.maxHeight:SetParent(groupMaint.maxHeightBack)
groupMaint.maxHeight:SetSize(78,fontSize)
groupMaint.maxHeight:SetPosition(1,1)
groupMaint.maxHeight:SetFont(Turbine.UI.Lotro.Font.Verdana20)
groupMaint.maxHeight:SetBackColor(Turbine.UI.Color.Black)

groupMaint.expandedCB=Turbine.UI.Lotro.CheckBox()
groupMaint.expandedCB:SetParent(groupMaint)
groupMaint.expandedCB:SetSize(groupMaint.groupSelect:GetWidth(),fontSize+2)
groupMaint.expandedCB:SetPosition(groupMaint.groupSelect:GetLeft(),groupMaint.undockedAttrib:GetTop()+groupMaint.undockedAttrib:GetHeight()+5)
groupMaint.expandedCB:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft)
groupMaint.expandedCB:SetFont(Settings.fontFace)
groupMaint.expandedCB:SetText(Resource[language][91])

groupMaint.crit1=Turbine.UI.Control()
groupMaint.crit1:SetParent(groupMaint)
groupMaint.crit1:SetPosition(margin,groupMaint.expandedCB:GetTop()+groupMaint.expandedCB:GetHeight()+5)
groupMaint.crit1:SetSize(groupMaint:GetWidth()-margin*2,(groupMaint:GetHeight()-60-groupMaint.crit1:GetTop())/3)
groupMaint.crit1:SetBackColor(Turbine.UI.Color(1,.08,.1,.08))

groupMaint.crit1.sep=Turbine.UI.Control()
groupMaint.crit1.sep:SetParent(groupMaint.crit1)
groupMaint.crit1.sep:SetSize(groupMaint.crit1:GetWidth(),1)
groupMaint.crit1.sep:SetPosition(0,fontSize/2)
groupMaint.crit1.sep:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.crit1.header=Turbine.UI.Label()
groupMaint.crit1.header:SetParent(groupMaint.crit1)
groupMaint.crit1.header:SetSize(fontMetric:GetTextWidth(Resource[language][79]),fontSize)
groupMaint.crit1.header:SetPosition((groupMaint.crit1:GetWidth()-groupMaint.crit1.header:GetWidth())/2-60,0)
groupMaint.crit1.header:SetBackColor(Turbine.UI.Color.Black)
groupMaint.crit1.header:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
groupMaint.crit1.header:SetFont(Settings.fontFace)
groupMaint.crit1.header:SetText(Resource[language][79])

groupMaint.crit1.clearButton=Turbine.UI.Lotro.Button()
groupMaint.crit1.clearButton:SetParent(groupMaint.crit1)
groupMaint.crit1.clearButton:SetSize(120,fontSize)
groupMaint.crit1.clearButton:SetPosition(groupMaint.crit1:GetWidth()-groupMaint.crit1.clearButton:GetWidth()-margin/2,groupMaint.crit1.header:GetTop())
groupMaint.crit1.clearButton:SetText(Resource[language][87])
groupMaint.crit1.clearButton.Click=function()
	groupMaint.crit1.listBox:ClearItems()
end

groupMaint.crit1.listBox=Turbine.UI.ListBox()
groupMaint.crit1.listBox:SetParent(groupMaint.crit1)
groupMaint.crit1.listBox:SetPosition(margin/2,groupMaint.crit1.header:GetTop()+groupMaint.crit1.header:GetHeight()+5)
groupMaint.crit1.listBox:SetSize(groupMaint.crit1:GetWidth()-margin*2,groupMaint.crit1:GetHeight()-30-groupMaint.crit1.listBox:GetTop())
groupMaint.crit1.listBox:SetOrientation(Turbine.UI.Orientation.Vertical)
groupMaint.crit1.listBox:SetAllowDrop(true)
groupMaint.crit1.listBox.AddNewName=function(sender,name)
	--add item to listbox
	local tmpItem=Turbine.UI.Label()
	tmpItem:SetParent(groupMaint.crit1.listBox)
	tmpItem:SetSize(groupMaint.crit1.listBox:GetWidth(),fontSize)
	tmpItem:SetFont(Settings.fontFace)
	tmpItem:SetText(name)
	tmpItem.name=name

	tmpItem.MouseClick=function(sender,args)
		if args.Button==1 then
			groupMaint.crit1.newText:SetText(sender.name)
		elseif args.Button==2 then
			local menuItems=criteriaMenu:GetItems()
			menuItems:Clear()
			menuItems:Add(Turbine.UI.MenuItem(Resource[language][92]))
			criteriaMenu.item=sender
			menuItems:Get(1).Click=function(sender,args)
				-- remove the selected item
				groupMaint.crit1.listBox:RemoveItem(criteriaMenu.item)
			end
			criteriaMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
		end
	end
	local pos=1
	for k=1,groupMaint.crit1.listBox:GetItemCount() do
		if groupMaint.crit1.listBox:GetItem(k).name<name then
			pos=k+1
		else
			break
		end
	end
	groupMaint.crit1.listBox:InsertItem(pos,tmpItem)
end
groupMaint.crit1.listBox.DragDrop=function(sender,args)
	-- user dragged something onto the listbox
	local ddi=args.DragDropInfo
	if ddi~=nil then
		if ddi.GetShortcut~=nil then
			local sc=ddi:GetShortcut()
			local scType=sc:GetType()
			if scType==Turbine.UI.Lotro.ShortcutType.Item then
				local scItem=sc:GetItem()
				local scData=sc:GetData() -- will be string "0xnnnn,0xnn" where 0xnnnn is the ItemUID (or 0) and 0xnn is the ItemGID (or 0)
				if scItem ~=nil then
					local customName=scItem:GetName()
					local name=scItem:GetItemInfo():GetName()
					groupMaint.crit1.newText:SetText(customName)
					if customName==name then
						local popup=PopUpDialog(Resource[language][84],string.gsub(Resource[language][83],"@name",customName),1,Resource[language][79],Resource[language][80],Resource[language][63],false,groupMaint.crit1.addButton.Click,groupMaint.crit1.listBox.AddItemAsGeneric)
					else
						groupMaint.crit1.addButton:Click()
					end
				end
			end
		end
	end
end
groupMaint.crit1.listBox.AddItemAsGeneric=function()
	groupMaint.crit2.newText:SetText(groupMaint.crit1.newText:GetText())
	groupMaint.crit2.addButton:Click()
end
groupMaint.crit1.vScroll=Turbine.UI.Lotro.ScrollBar()
groupMaint.crit1.vScroll:SetParent(groupMaint.crit1)
groupMaint.crit1.vScroll:SetSize(10,groupMaint.crit1.listBox:GetHeight())
groupMaint.crit1.vScroll:SetPosition(groupMaint.crit1.listBox:GetLeft()+groupMaint.crit1.listBox:GetWidth(),groupMaint.crit1.listBox:GetTop())
groupMaint.crit1.vScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
groupMaint.crit1.listBox:SetVerticalScrollBar(groupMaint.crit1.vScroll)

groupMaint.crit1.newTextBorder=Turbine.UI.Control()
groupMaint.crit1.newTextBorder:SetParent(groupMaint.crit1)
groupMaint.crit1.newTextBorder:SetSize(groupMaint.crit1:GetWidth()-margin-120,fontSize+2)
groupMaint.crit1.newTextBorder:SetPosition(groupMaint.crit1.listBox:GetLeft(),groupMaint.crit1.listBox:GetTop()+groupMaint.crit1.listBox:GetHeight()+5)
groupMaint.crit1.newTextBorder:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.crit1.newText=Turbine.UI.TextBox()
groupMaint.crit1.newText:SetParent(groupMaint.crit1.newTextBorder)
groupMaint.crit1.newText:SetSize(groupMaint.crit1.newTextBorder:GetWidth()-2,fontSize)
groupMaint.crit1.newText:SetPosition(1,1)
groupMaint.crit1.newText:SetFont(Settings.fontFace)
groupMaint.crit1.newText:SetBackColor(Turbine.UI.Color.Black)

groupMaint.crit1.addButton=Turbine.UI.Lotro.Button()
groupMaint.crit1.addButton:SetParent(groupMaint.crit1)
groupMaint.crit1.addButton:SetSize(120,20)
groupMaint.crit1.addButton:SetPosition(groupMaint.crit1.newTextBorder:GetLeft()+groupMaint.crit1.newTextBorder:GetWidth(),groupMaint.crit1.newTextBorder:GetTop())
groupMaint.crit1.addButton:SetText(Resource[language][66])
groupMaint.crit1.addButton.Click=function(sender,args)
	-- verify that name is not already in a displayTab UniqueName criteria for this container
	local name=string.trim(groupMaint.crit1.newText:GetText())
	groupMaint.crit1.newText:SetText(name)
	local found=false
	local container=getCurrentContainer()
	for groupIndex,group in ipairs(displayTabs[container]) do
		if Table.Contains(group.criteria.ItemNames,name,true) then
			found=true
			local popup=PopUpDialog(Resource[language][57],string.gsub(string.gsub(Resource[language][82],"@group",tostring(group.name)),"@name",name),3,Resource[language][61])
			break
		end
	end
	if not found then
		-- make sure it wasn't already added to the list box
		for k=1,groupMaint.crit1.listBox:GetItemCount() do
			local tmpItem=groupMaint.crit1.listBox:GetItem(k)
			if tmpItem.name==name then
				found=true
				break
			end
		end
		if not found then
			groupMaint.crit1.listBox:AddNewName(name)
		end
	end
end

groupMaint.crit2=Turbine.UI.Control()
groupMaint.crit2:SetParent(groupMaint)
groupMaint.crit2:SetPosition(margin,groupMaint.crit1:GetTop()+groupMaint.crit1:GetHeight()+5)
groupMaint.crit2:SetSize(groupMaint.crit1:GetWidth(),groupMaint.crit1:GetHeight())
groupMaint.crit2:SetBackColor(Turbine.UI.Color(1,.08,.08,.1))

groupMaint.crit2.sep=Turbine.UI.Control()
groupMaint.crit2.sep:SetParent(groupMaint.crit2)
groupMaint.crit2.sep:SetSize(groupMaint.crit2:GetWidth(),1)
groupMaint.crit2.sep:SetPosition(0,fontSize/2)
groupMaint.crit2.sep:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.crit2.header=Turbine.UI.Label()
groupMaint.crit2.header:SetParent(groupMaint.crit2)
groupMaint.crit2.header:SetSize(fontMetric:GetTextWidth(Resource[language][80]),fontSize)
groupMaint.crit2.header:SetPosition((groupMaint.crit2:GetWidth()-groupMaint.crit2.header:GetWidth())/2-60,0)
groupMaint.crit2.header:SetBackColor(Turbine.UI.Color.Black)
groupMaint.crit2.header:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
groupMaint.crit2.header:SetFont(Settings.fontFace)
groupMaint.crit2.header:SetText(Resource[language][80])

groupMaint.crit2.clearButton=Turbine.UI.Lotro.Button()
groupMaint.crit2.clearButton:SetParent(groupMaint.crit2)
groupMaint.crit2.clearButton:SetSize(120,fontSize)
groupMaint.crit2.clearButton:SetPosition(groupMaint.crit2:GetWidth()-groupMaint.crit2.clearButton:GetWidth()-margin/2,groupMaint.crit2.header:GetTop())
groupMaint.crit2.clearButton:SetText(Resource[language][87])
groupMaint.crit2.clearButton.Click=function()
	groupMaint.crit2.listBox:ClearItems()
end

groupMaint.crit2.listBox=Turbine.UI.ListBox()
groupMaint.crit2.listBox:SetParent(groupMaint.crit2)
groupMaint.crit2.listBox:SetPosition(margin/2,groupMaint.crit2.header:GetTop()+groupMaint.crit2.header:GetHeight()+5)
groupMaint.crit2.listBox:SetSize(groupMaint.crit2:GetWidth()-margin*2,groupMaint.crit2:GetHeight()-30-groupMaint.crit2.listBox:GetTop())
groupMaint.crit2.listBox:SetOrientation(Turbine.UI.Orientation.Vertical)
groupMaint.crit2.listBox:SetAllowDrop(true)
groupMaint.crit2.listBox.AddNewName=function(sender,name)
	--add item to listbox
	local tmpItem=Turbine.UI.Label()
	tmpItem:SetParent(groupMaint.crit2.listBox)
	tmpItem:SetSize(groupMaint.crit2.listBox:GetWidth(),fontSize)
	tmpItem:SetFont(Settings.fontFace)
	tmpItem:SetText(name)
	tmpItem.name=name
	tmpItem.MouseClick=function(sender,args)
		if args.Button==1 then
			groupMaint.crit2.newText:SetText(sender.name)
		elseif args.Button==2 then
			local menuItems=criteriaMenu:GetItems()
			menuItems:Clear()
			menuItems:Add(Turbine.UI.MenuItem(Resource[language][92]))
			criteriaMenu.item=sender
			menuItems:Get(1).Click=function(sender,args)
				-- remove the selected item
				groupMaint.crit2.listBox:RemoveItem(criteriaMenu.item)
			end
			criteriaMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
		end
	end

	local pos=1
	for k=1,groupMaint.crit2.listBox:GetItemCount() do
		if groupMaint.crit2.listBox:GetItem(k).name<name then
			pos=k+1
		else
			break
		end
	end
	groupMaint.crit2.listBox:InsertItem(pos,tmpItem)
end
groupMaint.crit2.listBox.DragDrop=function(sender,args)
	-- user dragged something onto the listbox
	local ddi=args.DragDropInfo
	if ddi~=nil then
		if ddi.GetShortcut~=nil then
			local sc=ddi:GetShortcut()
			local scType=sc:GetType()
			if scType==Turbine.UI.Lotro.ShortcutType.Item then
				local scItem=sc:GetItem()
				local scData=sc:GetData() -- will be string "0xnnnn,0xnn" where 0xnnnn is the ItemUID (or 0) and 0xnn is the ItemGID (or 0)
				if scItem ~=nil then
					local name=scItem:GetItemInfo():GetName()
					groupMaint.crit2.newText:SetText(name)
					groupMaint.crit2.addButton:Click()
				end
			end
		end
	end
end

groupMaint.crit2.vScroll=Turbine.UI.Lotro.ScrollBar()
groupMaint.crit2.vScroll:SetParent(groupMaint.crit2)
groupMaint.crit2.vScroll:SetSize(10,groupMaint.crit2.listBox:GetHeight())
groupMaint.crit2.vScroll:SetPosition(groupMaint.crit2.listBox:GetLeft()+groupMaint.crit2.listBox:GetWidth(),groupMaint.crit2.listBox:GetTop())
groupMaint.crit2.vScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
groupMaint.crit2.listBox:SetVerticalScrollBar(groupMaint.crit2.vScroll)

groupMaint.crit2.newTextBorder=Turbine.UI.Control()
groupMaint.crit2.newTextBorder:SetParent(groupMaint.crit2)
groupMaint.crit2.newTextBorder:SetSize(groupMaint.crit2:GetWidth()-margin-120,fontSize+2)
groupMaint.crit2.newTextBorder:SetPosition(groupMaint.crit2.listBox:GetLeft(),groupMaint.crit2.listBox:GetTop()+groupMaint.crit2.listBox:GetHeight()+5)
groupMaint.crit2.newTextBorder:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.crit2.newText=Turbine.UI.TextBox()
groupMaint.crit2.newText:SetParent(groupMaint.crit2.newTextBorder)
groupMaint.crit2.newText:SetSize(groupMaint.crit2.newTextBorder:GetWidth()-2,fontSize)
groupMaint.crit2.newText:SetPosition(1,1)
groupMaint.crit2.newText:SetFont(Settings.fontFace)
groupMaint.crit2.newText:SetBackColor(Turbine.UI.Color.Black)

groupMaint.crit2.addButton=Turbine.UI.Lotro.Button()
groupMaint.crit2.addButton:SetParent(groupMaint.crit2)
groupMaint.crit2.addButton:SetSize(120,20)
groupMaint.crit2.addButton:SetPosition(groupMaint.crit2.newTextBorder:GetLeft()+groupMaint.crit2.newTextBorder:GetWidth(),groupMaint.crit2.newTextBorder:GetTop())
groupMaint.crit2.addButton:SetText(Resource[language][66])
groupMaint.crit2.addButton.Click=function(sender,args)
	-- verify that name is not already in a displayTab UniqueName criteria for this container
	local name=string.trim(groupMaint.crit2.newText:GetText())
	groupMaint.crit2.newText:SetText(name)
	local found=false
	local container=getCurrentContainer()
	for groupIndex,group in ipairs(displayTabs[container]) do
		if Table.Contains(group.criteria.ItemInfoNames,name,true) then
			found=true
			local popup=PopUpDialog(Resource[language][57],string.gsub(string.gsub(Resource[language][82],"@group",tostring(group.name)),"@name",name),3,Resource[language][61])
			break
		end
	end
	if not found then
		-- make sure it wasn't already added to the list box
		for k=1,groupMaint.crit2.listBox:GetItemCount() do
			local tmpItem=groupMaint.crit2.listBox:GetItem(k)
			if tmpItem.name==name then
				found=true
				break
			end
		end
		if not found then
			groupMaint.crit2.listBox:AddNewName(name)
		end
	end
end

groupMaint.crit3=Turbine.UI.Control()
groupMaint.crit3:SetParent(groupMaint)
groupMaint.crit3:SetPosition(margin,groupMaint.crit2:GetTop()+groupMaint.crit2:GetHeight()+5)
groupMaint.crit3:SetSize(groupMaint.crit2:GetWidth(),groupMaint.crit2:GetHeight())
groupMaint.crit3:SetBackColor(Turbine.UI.Color(1,.1,.08,.1))

groupMaint.crit3.sep=Turbine.UI.Control()
groupMaint.crit3.sep:SetParent(groupMaint.crit3)
groupMaint.crit3.sep:SetSize(groupMaint.crit3:GetWidth(),1)
groupMaint.crit3.sep:SetPosition(0,fontSize/2)
groupMaint.crit3.sep:SetBackColor(Turbine.UI.Color.Silver)

groupMaint.crit3.header=Turbine.UI.Label()
groupMaint.crit3.header:SetParent(groupMaint.crit3)
groupMaint.crit3.header:SetSize(fontMetric:GetTextWidth(Resource[language][81]),fontSize)
groupMaint.crit3.header:SetPosition((groupMaint.crit3:GetWidth()-groupMaint.crit3.header:GetWidth())/2-60,0)
groupMaint.crit3.header:SetBackColor(Turbine.UI.Color.Black)
groupMaint.crit3.header:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
groupMaint.crit3.header:SetFont(Settings.fontFace)
groupMaint.crit3.header:SetText(Resource[language][81])

groupMaint.crit3.clearButton=Turbine.UI.Lotro.Button()
groupMaint.crit3.clearButton:SetParent(groupMaint.crit3)
groupMaint.crit3.clearButton:SetSize(120,fontSize)
groupMaint.crit3.clearButton:SetPosition(groupMaint.crit3:GetWidth()-groupMaint.crit3.clearButton:GetWidth()-margin/2,groupMaint.crit3.header:GetTop())
groupMaint.crit3.clearButton:SetText(Resource[language][87])
groupMaint.crit3.clearButton.Click=function()
	for k=1,groupMaint.crit3.listBox:GetItemCount() do
		groupMaint.crit3.listBox:GetItem(k):SetChecked(false)
	end
	groupMaint.sortCrit3()
end

groupMaint.sortCrit3=function()
	-- need to resort the listbox with checked items at top of list
	local tmpTable={}
	for k=1,groupMaint.crit3.listBox:GetItemCount() do
		local tmpItem=groupMaint.crit3.listBox:GetItem(k)
		table.insert(tmpTable,tmpItem)
	end
	groupMaint.crit3.listBox:ClearItems()
	-- sort checked items to top and then sort by name
	table.sort(tmpTable,function(a,b) if a:IsChecked() and not b:IsChecked() then return true else if a:IsChecked()==b:IsChecked() and a:GetText()<b:GetText() then return true end end end)
	for k,v in ipairs(tmpTable) do
		groupMaint.crit3.listBox:AddItem(v)
	end
end

groupMaint.crit3.listBox=Turbine.UI.ListBox()
groupMaint.crit3.listBox:SetParent(groupMaint.crit3)
groupMaint.crit3.listBox:SetPosition(margin/2,groupMaint.crit3.header:GetTop()+groupMaint.crit3.header:GetHeight()+5)
groupMaint.crit3.listBox:SetSize(groupMaint.crit3:GetWidth()-margin*2,groupMaint.crit3:GetHeight()-30-groupMaint.crit3.listBox:GetTop())
groupMaint.crit3.listBox:SetOrientation(Turbine.UI.Orientation.Vertical)
groupMaint.crit3.listBox:SetAllowDrop(true)
groupMaint.crit3.listBox:SetMaxColumns(1)
-- populate lisbox with all itemcategory string entries
local tmpCat={}
-- ItemCategory is sorted by current language
for k,v in pairs(ItemCategory) do
	local tmpItem=Turbine.UI.Lotro.CheckBox()
	tmpItem:SetSize(groupMaint.crit3.listBox:GetWidth(),fontSize)
	tmpItem:SetText(v[2][language])
	tmpItem.CategoryID=v[1] -- v[1] is the actual turbine category code
	tmpItem:SetChecked(false)
	tmpItem.CheckedChanged=function()
		groupMaint.sortCrit3()
	end
	groupMaint.crit3.listBox:AddItem(tmpItem)
end

groupMaint.crit3.listBox.DragDrop=function(sender,args)
	-- user dragged something onto the listbox
	local ddi=args.DragDropInfo
	if ddi~=nil then
		if ddi.GetShortcut~=nil then
			local sc=ddi:GetShortcut()
			local scType=sc:GetType()
			if scType==Turbine.UI.Lotro.ShortcutType.Item then
				local scItem=sc:GetItem()
				if scItem ~=nil then
					local scItemInfo=scItem:GetItemInfo()
					if scItemInfo~=nil then
						local category=scItemInfo:GetCategory()
						for k=1,groupMaint.crit3.listBox:GetItemCount() do
							local tmpItem=groupMaint.crit3.listBox:GetItem(k)
							if tmpItem.CategoryID==category then
								tmpItem:SetChecked(true)
								setGroupMaintMessage(string.gsub(Resource[language][90],"@category",tmpItem:GetText()),Turbine.UI.Color(1,.4,1,.4))
								break
							end
						end
					end
				end
			end
		end
	end
	groupMaint.sortCrit3()
end

groupMaint.crit3.vScroll=Turbine.UI.Lotro.ScrollBar()
groupMaint.crit3.vScroll:SetParent(groupMaint.crit3)
groupMaint.crit3.vScroll:SetSize(10,groupMaint.crit3.listBox:GetHeight())
groupMaint.crit3.vScroll:SetPosition(groupMaint.crit3.listBox:GetLeft()+groupMaint.crit3.listBox:GetWidth(),groupMaint.crit3.listBox:GetTop())
groupMaint.crit3.vScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
groupMaint.crit3.listBox:SetVerticalScrollBar(groupMaint.crit3.vScroll)

groupMaint.SaveButton=Turbine.UI.Lotro.Button()
groupMaint.SaveButton:SetParent(groupMaint)
groupMaint.SaveButton:SetSize(120,20)
groupMaint.SaveButton:SetPosition(20,groupMaint:GetHeight()-45)
groupMaint.SaveButton.Click=function()
	local container=getCurrentContainer()
	local index=groupMaint.groupSelect:GetValue()
	local name=string.trim(groupMaint.name:GetText())
	groupMaint.name:SetText(name)

	-- verify that name is unique
	local error=false
	if name=="" then
		error=true
		local popup=PopUpDialog(Resource[language][57],Resource[language][88],3,Resource[language][61])
		setGroupMaintMessage(Resource[language][88],Turbine.UI.Color.Red)
	end
	if not error then
		for k,v in ipairs(displayTabs[container]) do
			if k~=index and v.name==name then
				error=true
				local popup=PopUpDialog(Resource[language][57],string.gsub(Resource[language][85],"@name",name),3,Resource[language][61])
				setGroupMaintMessage(Resource[language][89],Turbine.UI.Color.Red)
				break
			end
		end
	end
	if not error then
		local dt
		if index==0 then
			-- create generic blank display tab and append it to the end of the table
			index=#displayTabs[container]+1
			displayTabs[container][index]={}
			dt=displayTabs[container][index]
			dt.isMain=false
			dt.criteria={}
			dt.docked=true -- set it to true initially since that means the window is not expected to exist yet
			groupMaint.groupSelect:AddItem(name,index)
			local panel=createDisplayTabPanel(dt)
			dt.docked=groupMaint.dockedCB:IsChecked()
			if dt.docked then
				panel:SetParent(inventoryWindow.ItemList)
				inventoryPanel.ItemList:AddItem(panel)
			else
				dt.window=createDTWindow(dt)
				if dt.width==nil then dt.width=inventoryPanel.ItemList:GetWidth() end
				if dt.top==nil then dt.top=displayHeight/2 end
				if dt.top>displayHeight-20 then dt.top=displayHeight-20 end
				if dt.left==nil then dt.left=displayWidth/2 end
				if dt.left>displayWidth-40 then dt.left=displayWidth-40 end
				panel:SetParent(dt.window)
			end
			groupMaint.SaveButton:SetText(Resource[language][68])
			-- enable the move up and move top buttons since the new item is always added at the bottom
			groupMaint.moveTop:SetEnabled(true)
			groupMaint.moveUp:SetEnabled(true)
		else
			groupMaint.groupSelect.ListData:GetItem(index+1):SetText(name)
			groupMaint.groupSelect.CurrentValue:SetText(name)
		end
		dt=displayTabs[container][index]

		dt.name=name
		dt.panel.TitleBar:SetText(name)
		-- now update all the values
		dt.lockHeight=groupMaint.maxHeightCB:IsChecked()
		local tmpHeight=tonumber(groupMaint.maxHeight:GetText())
		if tmpHeight==nil then
			if dt.height==nil then
				tmpHeight=20+inventoryWindow.ItemEntryBackHeight+4
			else
				tmpHeight=dt.height
			end
		end
		if tmpHeight<20+inventoryWindow.ItemEntryBackHeight+4 then
			tmpHeight=20+inventoryWindow.ItemEntryBackHeight+4
		end
		if dt.lockHeight then
			groupMaint.maxHeight:SetText(tmpHeight)
		end
		dt.height=tmpHeight
		if not groupMaint.dockedCB:IsChecked() then
			local tmpLeft=tonumber(groupMaint.left:GetText())
			if tmpLeft==nil then
				if dt.left==nil then
					tmpLeft=displayWidth/2
				else
					tmpLeft=dt.left
				end
			end
			dt.left=tmpLeft
			if dt.left<0 then dt.left=0 end
			if dt.left>displayWidth-40 then dt.left=displayWidth-40 end
			groupMaint.left:SetText(tmpLeft)

			local tmpTop=tonumber(groupMaint.top:GetText())
			if tmpTop==nil then
				if dt.top==nil then
					tmpTop=displayWidth/2
				else
					tmpTop=dt.top
				end
			end
			dt.top=tmpTop
			if dt.top>displayHeight-20 then dt.top=displayHeight-20 end
			if dt.top<0 then dt.top=0 end
			groupMaint.top:SetText(tmpTop)

			local tmpWidth=tonumber(groupMaint.width:GetText())
			if tmpWidth==nil then
				if dt.width==nil then
					tmpWidth=inventoryPanel.ItemList:GetWidth()
				else
					tmpWidth=dt.width
				end
			end

			if tmpWidth<inventoryPanel.EntryNormalWidth+10 then tmpWidth=inventoryPanel.EntryNormalWidth+10 end
			if tmpWidth>displayWidth then tmpWidth=displayWidth end
			groupMaint.width:SetText(tmpWidth)
			dt.width=tmpWidth
			if dt.window~=nil then
				dt.window:SetPosition(dt.left,dt.top)
				dt.window:SetWidth(tmpWidth+4)
			end
		end
		if dt.docked~=groupMaint.dockedCB:IsChecked() then
			-- docked changed, apply it...
			dt.panel.DockButton:MouseClick()
		end
		dt.expanded=groupMaint.expandedCB:IsChecked()
		dt.criteria.ItemNames={}
		dt.criteria.ItemInfoNames={}
		dt.criteria.UIDValues={}
		dt.criteria.GIDValues={}
		dt.criteria.CIDValues={}
-- determine if criteria.ItemNames or ItemInfoNames contain a pattern
		dt.criteria.ItemNamesContainPattern=false
		dt.criteria.ItemInfoNamesContainPattern=false
		for k=1,groupMaint.crit1.listBox:GetItemCount() do
			local name=groupMaint.crit1.listBox:GetItem(k).name
			if not dt.criteria.ItemNamesContainPattern then
				-- test name to see if it is a pattern - for now, consider anything non-alpha as a pattern
				if string.isPattern(name) then dt.criteria.ItemNamesContainPattern=true end
			end
			table.insert(dt.criteria.ItemNames,name)
		end
		for k=1,groupMaint.crit2.listBox:GetItemCount() do
			local name=groupMaint.crit2.listBox:GetItem(k).name
			if not dt.criteria.ItemInfoNamesContainPattern then
				-- test name to see if it is a pattern - for now, consider anything non-alpha as a pattern
				if string.isPattern(name) then dt.criteria.ItemInfoNamesContainPattern=true end
			end
			table.insert(dt.criteria.ItemInfoNames,name)
		end
		for k=1,groupMaint.crit3.listBox:GetItemCount() do
			local tmpItem=groupMaint.crit3.listBox:GetItem(k)
			if tmpItem:IsChecked() then
				table.insert(dt.criteria.CIDValues,tmpItem.CategoryID)
			end
		end
		table.sort(dt.criteria.CIDValues) -- need to put them in order so that Table.Contains will work
		groupMaint.groupSelect:SetSelectedIndex(index+1)
		setGroupMaintMessage(Resource[language][86],Turbine.UI.Color(1,.4,1,.4))
		-- need to update the xref
		updateDisplayTabXref()
		inventoryPanel.Refresh()
		getItemEntryLayout()
		applyItemEntryLayout()
		inventoryPanel:Layout()
		inventoryPanel.cropDelay=Settings.cropDelay
		inventoryPanel:SetWantsUpdates(true)

		-- temporary bug workaround, reload the plugin to fix floating panel issues
		ReloadAltInventory()
	end
end

-- check name for uniqueness - can't allow multiple bags (in same container) with identical names as it is used to identify the index when not known
-- when saving, we need to be sure that criteria are not already defined elsewhere and if so, alert user that the other criteria will be removed...

groupMaint.message=Turbine.UI.Label()
groupMaint.message:SetParent(groupMaint)
groupMaint.message:SetSize(groupMaint:GetWidth()-(groupMaint.SaveButton:GetLeft()+groupMaint.SaveButton:GetWidth())*2,fontSize)
groupMaint.message:SetPosition(groupMaint.SaveButton:GetLeft()+groupMaint.SaveButton:GetWidth(),groupMaint.SaveButton:GetTop())
groupMaint.message:SetFont(Settings.fontFace)
groupMaint.message:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
groupMaint.message.Update=function()
	if groupMaint.message.endTime==nil then
		-- set up timer
		groupMaint.message.endTime=Turbine.Engine:GetLocalTime()+10
		groupMaint.message.startTime=Turbine.Engine:GetLocalTime()
		groupMaint.message:SetOpacity(1)
	else
		local timeNow=Turbine.Engine:GetLocalTime()
		if timeNow>=groupMaint.message.endTime then
			groupMaint.message:SetText("")
			groupMaint.message:SetWantsUpdates(false)
		else
			local a=1-((timeNow-groupMaint.message.startTime)/10)^3
			local color=groupMaint.message:GetForeColor()
			color["A"]=a
			groupMaint.message:SetForeColor(color)
		end
	end
end
setGroupMaintMessage=function(msg,color)
	if color==nil then
		groupMaint.message:SetForeColor(Turbine.UI.Color.White)
	else
		groupMaint.message:SetForeColor(color)
	end
	groupMaint.message:SetText(msg)
	groupMaint.message.endTime=nil
	groupMaint.message:SetWantsUpdates(true)
end

groupMaint.CloseButton=Turbine.UI.Lotro.Button()
groupMaint.CloseButton:SetParent(groupMaint)
groupMaint.CloseButton:SetSize(120,20)
groupMaint.CloseButton:SetText(Resource[language][63])
groupMaint.CloseButton:SetPosition(groupMaint:GetWidth()-20-groupMaint.CloseButton:GetWidth(),groupMaint:GetHeight()-45)
groupMaint.CloseButton.MouseClick=function()
	groupMaint:SetVisible(false)
end

groupMaint.Show=function(sender,container, dt)
	groupMaint.loading=true -- semaphore to prevent race conditions with the checkboxes
	local index=0 -- 0 equals new
	if container==nil then
		-- get current category
		container=getCurrentContainer()
	end
	local realContainer=getCurrentContainer(true)
	if realContainer=="all" then
		groupMaint.container:SetText(Resource[1][12])
		groupMaint.useBagsCB:SetChecked(displayTabs.all.useBagTags)
		groupMaint.useBagsCB:SetVisible(true)
	elseif realContainer=="vault" then
		groupMaint.container:SetText(Resource[1][8])
		groupMaint.useBagsCB:SetChecked(displayTabs.vault.useBagTags)
		groupMaint.useBagsCB:SetVisible(true)
	elseif realContainer=="shared" then
		groupMaint.container:SetText(Resource[1][9])
		groupMaint.useBagsCB:SetChecked(displayTabs.shared.useBagTags)
		groupMaint.useBagsCB:SetVisible(true)
	else
		groupMaint.useBagsCB:SetChecked(false)
		groupMaint.useBagsCB:SetVisible(false)
		groupMaint.container:SetText(Resource[1][7])
	end

	if dt==nil or dt.name==nil then
	else
		index=getDisplayTabIndexFromName(container,dt.name)
	end
	-- now clear the selection list and repopulate it
	groupMaint.groupSelect:ClearList()
	groupMaint.groupSelect:AddItem(Resource[language][70],0)

	for k,v in ipairs(displayTabs[container]) do
		groupMaint.groupSelect:AddItem(v.name,k)
	end
	groupMaint.groupSelect:SetSelectedIndex(index+1) -- add 1 to account for "new" entry
	groupMaint.groupSelect:SelectedIndexChanged()
	groupMaint:SetVisible(true)
	groupMaint.loading=false
end