-- the entire display mechanism needs to be overhauled. adding the zoom factor caused some issues that were never fully cleaned up and were only jury-rigged to work using update handlers and delays that should NOT be necessary


-- check floating panel and window size calculation - somewhere the panel and window are not getting resized when an item is added or removed - this causes the panel to continue to appear blank when it was previously blank and a matching item is added or to remain displaying full size when the last item is removed and it should shrink
-- this may be due to an error in the category matching (a new type of category was added by SSG, might have to do with prep for carry-all naming)

--***** Notes on groups (previously referred to as tabs) ****************************
-- now supports groups - each group container has a table of criteria specifying which items are displayed in that group. the criteria are ItemName, ItemInfoName and CIDValues (Category ID)
-- -- UIDValues (Unique ItemID), GUIDValues (Generic ItemID) are also defined for future use (assuming we ever get a way to retrieve item IDs)
-- -- should we allow tracking a preferred position? that would raise havoc with the sort feature

-- groups are defined for each storage type, "bags", "vault", "shared" and "all"
-- -- displayTabs.bags[1].Name=""
-- -- displayTabs[1].isMain=true
-- -- displayTabs[1].UIDValues={} -- ItemUID - not yet available
-- -- displayTabs[1].GIDValues={} -- ItemGID - not yet available
-- -- displayTabs[1].ItemNames={} -- names
-- -- displayTabs[1].ItemInfoNames={} -- names
-- -- displayTabs[1].CIDValues={} -- categoryID (from ItemCategory)
-- -- displayTabs[2].Name="Oils"
-- -- displayTabs[2].isMain=false
-- -- displayTabs[2].UIDValues={}
-- -- displayTabs[2].UIDValues[1]=0x1234567812345678 -- ItemUID
-- -- displayTabs[2].GIDValues={} -- ItemGID
-- -- displayTabs[2].GIDValues[1]=0x12345678 -- ItemGID
-- -- dispalyTabs[2].ItemNames={} -- used to track items with unique names like crafted items or LIs - can use ItemInfoName if unique is not desired
-- -- displayTabs[2].ItemInfoNames={} -- info names
-- -- dispalyTabs[2].ItemInfoNames[1]="Fire Oil"
-- -- dispalyTabs[2].ItemInfoNames[2]="Light Oil"
-- -- displayTabs[2].CIDValues={} -- categoryID (from ItemCategory)
-- -- displayTabs[2].CIDValues[1]=15 -- Turbine.Gameplay.ItemCategory.Oil
-- -- etc

-- each tab has a reference to two UI elements
-- -- .panel is the control that houses the title bar (with the name, dock button and expand/contract button) and the listbox that actually displays the items
-- -- .window is a container for holding the tab when it is not docked and has the footer with the height lock
 
-- when bags are refreshed, each item is assigned a tab (group) based on which "Tab" it matches BEST - matching based on ItemName first then ItemInfoName and categoryID last. if not found, then it goes into the "main" tab which is everything else (undefined)

-- assign displayTabXref to each inventory item when plugin loads
-- whenever inventory changes, assign displayTabXref to any item that doesn't have an entry
-- whenever tab definitions change, reassign ALL displayTabXref entries
-- that means that whenever it is time to refresh the inventory display, we know exactly which tab an item belongs in. we can simply clear all tabs and assign items to tabs
-- based on displayTabXref and apply the sort to each tab that has items.

-- each tab will have it's own Listbox control with the first entry being the tab header
-- -- tab header has the title, docked button, expand/contract button - when contracted, simply set the panel height to 20, effectively hiding the list
-- right edge of the window has to allow resizing width to a minimum width of inventoryPanel.EntryWideWidth, max of displayWidth-displayTab.left
-- after applying the filter to each listbox, reassign the listbox height to the top of the last row+the height of the last row - override if the panel is detached and the height is greater than the displayHeight-panel.top
-- the overall display has a row for each tab listbox
-- the "main" tab listbox must always exist, has NO criteria and thus contains all items that are not assigned to another tab

-- allow displayTabs to be undocked - 
-- -- when undocked, they need to create their own window,
-- -- set it as their parent and display at their last saved left/top position (or center of screen if no prior position)

--*********************************

qtyDetail=Turbine.UI.ContextMenu()
groupMenu=Turbine.UI.ContextMenu()
toolTip=Turbine.UI.ContextMenu()

-- this is the main display
function createInventoryWindow()
	inventoryWindow=nil -- incase this is a re-creation due to the SetSTretchMode(0) bug
	inventoryWindow=Turbine.UI.Lotro.Window();
	inventoryWindow.LeftMargin=5;
	inventoryWindow.TopMargin=140;
	inventoryWindow.BottomMargin=47;
	inventoryWindow:SetText("      "..Resource[language][2].."      ");
	inventoryWindow:SetVisible(not Settings.useMinimalHeader and not Settings.loadMinimized)
	inventoryWindow.MinimumWidth=Settings.panelMinWidth*displayWidth+inventoryWindow.LeftMargin*2
	inventoryWindow.MinimumHeight=Settings.panelMinHeight*displayHeight+inventoryWindow.TopMargin+inventoryWindow.BottomMargin
	local left=Settings.panelLeft*displayWidth-inventoryWindow.LeftMargin*Settings.zoom
	local top=Settings.panelTop*displayHeight-inventoryWindow.TopMargin*Settings.zoom
	if left<0 then left=0 end
	if top<0 then top=0 end
	inventoryWindow:SetPosition(left,top)
	inventoryWindow:SetResizable(true);
	inventoryWindow:SetSize(Settings.panelWidth*displayWidth+inventoryWindow.LeftMargin*2*Settings.zoom,Settings.panelHeight*displayHeight+(inventoryWindow.TopMargin+inventoryWindow.BottomMargin)*Settings.zoom);
	inventoryWindow.TitleIcon=Turbine.UI.Control();
	inventoryWindow.TitleIcon:SetParent(inventoryWindow);
	inventoryWindow.TitleIcon:SetSize(30,25);
	if language==1 then
		inventoryWindow.TitleIcon:SetPosition(inventoryWindow:GetWidth()/2-104,5);
	elseif language==2 then
		inventoryWindow.TitleIcon:SetPosition(inventoryWindow:GetWidth()/2-119,5);
	else
		inventoryWindow.TitleIcon:SetPosition(inventoryWindow:GetWidth()/2-100,5);
	end
	inventoryWindow.TitleIcon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TitleIcon:SetBackground("GaranStuff/AltInventory/Resources/title_icon.tga");
	inventoryWindow.TitleIcon:SetMouseVisible(false);

	-- panel background
	inventoryWindow.TopPanelTiled=Turbine.UI.Control();
	inventoryWindow.TopPanelTiled:SetParent(inventoryWindow);
	inventoryWindow.TopPanelTiled:SetZOrder(-1);
	inventoryWindow.TopPanelTiled:SetSize(inventoryWindow:GetWidth()-30,111);
	inventoryWindow.TopPanelTiled:SetPosition(6,29);
	inventoryWindow.TopPanelTiled:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelTiled:SetBackground("GaranStuff/AltInventory/Resources/background_toptiled_bw.jpg");
	inventoryWindow.TopPanelTiled:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
	inventoryWindow.TopPanelTiled:SetBackColor(Settings.panelBackColor);
	inventoryWindow.TopPanelTiled:SetMouseVisible(false);

	inventoryWindow.TopPanelTiledTrim=Turbine.UI.Control();
	inventoryWindow.TopPanelTiledTrim:SetParent(inventoryWindow);
	inventoryWindow.TopPanelTiledTrim:SetZOrder(-1);
	inventoryWindow.TopPanelTiledTrim:SetSize(inventoryWindow:GetWidth()-30,4);
	inventoryWindow.TopPanelTiledTrim:SetPosition(6,136);
	inventoryWindow.TopPanelTiledTrim:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelTiledTrim:SetBackground("GaranStuff/AltInventory/Resources/background_toptiled_trim.jpg");
	inventoryWindow.TopPanelTiledTrim:SetMouseVisible(false);

	inventoryWindow.TopPanelLeft=Turbine.UI.Control();
	inventoryWindow.TopPanelLeft:SetParent(inventoryWindow);
	inventoryWindow.TopPanelLeft:SetZOrder(-1);
	inventoryWindow.TopPanelLeft:SetSize(86,118);
	inventoryWindow.TopPanelLeft:SetPosition(5,22);
	inventoryWindow.TopPanelLeft:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelLeft:SetBackground("GaranStuff/AltInventory/Resources/background_top_left.tga");
	inventoryWindow.TopPanelLeft:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
	inventoryWindow.TopPanelLeft:SetBackColor(Settings.panelBackColor);
	inventoryWindow.TopPanelLeft:SetMouseVisible(false);

	inventoryWindow.TopPanelLeftTrim=Turbine.UI.Control();
	inventoryWindow.TopPanelLeftTrim:SetParent(inventoryWindow);
	inventoryWindow.TopPanelLeftTrim:SetZOrder(-1);
	inventoryWindow.TopPanelLeftTrim:SetSize(86,118);
	inventoryWindow.TopPanelLeftTrim:SetPosition(5,22);
	inventoryWindow.TopPanelLeftTrim:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelLeftTrim:SetBackground("GaranStuff/AltInventory/Resources/background_top_left_trim.tga");
	inventoryWindow.TopPanelLeftTrim:SetMouseVisible(false);

	inventoryWindow.TopPanelCenter=Turbine.UI.Control();
	inventoryWindow.TopPanelCenter:SetParent(inventoryWindow);
	inventoryWindow.TopPanelCenter:SetSize(382,106);
	inventoryWindow.TopPanelCenter:SetPosition((inventoryWindow:GetWidth()-inventoryWindow.TopPanelCenter:GetWidth())/2-3,30);
	inventoryWindow.TopPanelCenter:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelCenter:SetBackground("GaranStuff/AltInventory/Resources/background_top_center.tga");
	inventoryWindow.TopPanelCenter:SetMouseVisible(false);

	inventoryWindow.TopPanelRight=Turbine.UI.Control();
	inventoryWindow.TopPanelRight:SetParent(inventoryWindow);
	inventoryWindow.TopPanelRight:SetZOrder(-1);
	inventoryWindow.TopPanelRight:SetSize(86,118);
	inventoryWindow.TopPanelRight:SetPosition(inventoryWindow:GetWidth()-inventoryWindow.TopPanelRight:GetWidth()-5,22);
	inventoryWindow.TopPanelRight:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelRight:SetBackground("GaranStuff/AltInventory/Resources/background_top_right.tga");
	inventoryWindow.TopPanelRight:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
	inventoryWindow.TopPanelRight:SetBackColor(Settings.panelBackColor);
	inventoryWindow.TopPanelRight:SetMouseVisible(false);

	inventoryWindow.TopPanelRightTrim=Turbine.UI.Control();
	inventoryWindow.TopPanelRightTrim:SetParent(inventoryWindow);
	inventoryWindow.TopPanelRightTrim:SetZOrder(-1);
	inventoryWindow.TopPanelRightTrim:SetSize(86,118);
	inventoryWindow.TopPanelRightTrim:SetPosition(inventoryWindow:GetWidth()-inventoryWindow.TopPanelRightTrim:GetWidth()-5,22);
	inventoryWindow.TopPanelRightTrim:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.TopPanelRightTrim:SetBackground("GaranStuff/AltInventory/Resources/background_top_right_trim.tga");
	inventoryWindow.TopPanelRightTrim:SetMouseVisible(false);

	inventoryWindow.CloseButton=Turbine.UI.Control();
	inventoryWindow.CloseButton:SetParent(inventoryWindow);
	inventoryWindow.CloseButton:SetSize(16,16);
	inventoryWindow.CloseButton:SetPosition(inventoryWindow:GetWidth()-inventoryWindow.CloseButton:GetWidth()-6,21);
	inventoryWindow.CloseButton:SetBackground(0x41000196);
	inventoryWindow.CloseButton:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.CloseButton.MouseEnter=function()
		inventoryWindow.CloseButton:SetBackground(0x41000198);
	end
	inventoryWindow.CloseButton.MouseLeave=function()
		inventoryWindow.CloseButton:SetBackground(0x41000196);
	end
	inventoryWindow.CloseButton.MouseDown=function()
		inventoryWindow.CloseButton:SetBackground(0x41000197);
	end
	inventoryWindow.CloseButton.MouseClick=function()
		inventoryWindow:Close();
		if Settings.showIcon==1 or Settings.showIcon==2 then
			iconWindow:SetVisible(true);
		end
	end

	inventoryWindow.CapacityPanel={};
	inventoryWindow.CapacityPanel=Turbine.UI.Control();
	inventoryWindow.CapacityPanel:SetParent(inventoryWindow);
	inventoryWindow.CapacityPanel.MaxWidth=376;
	inventoryWindow.CapacityPanel:SetPosition((inventoryWindow:GetWidth()-inventoryWindow.CapacityPanel.MaxWidth)/2,65);
	inventoryWindow.CapacityPanel:SetSize(inventoryWindow.CapacityPanel.MaxWidth,17);
	inventoryWindow.CapacityEmpty=Turbine.UI.Label();
	inventoryWindow.CapacityEmpty:SetParent(inventoryWindow);
	inventoryWindow.CapacityEmpty:SetPosition(inventoryWindow.CapacityPanel:GetLeft()+3,64);
	inventoryWindow.CapacityEmpty:SetSize(60,20);
	inventoryWindow.CapacityEmpty:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft);
	inventoryWindow.CapacityEmpty:SetFont(Turbine.UI.Lotro.Font.TrajanPro16);
	inventoryWindow.CapacityEmpty:SetForeColor(Turbine.UI.Color(.2,1,.2));
	inventoryWindow.CapacityEmpty:SetOutlineColor(Turbine.UI.Color(0,0,0));
	inventoryWindow.CapacityEmpty:SetFontStyle(Turbine.UI.FontStyle.Outline);
	inventoryWindow.CapacityEmpty:SetText(Resource[language][4]);
	inventoryWindow.CapacityDisplay=Turbine.UI.Label();
	inventoryWindow.CapacityDisplay:SetParent(inventoryWindow);
	inventoryWindow.CapacityDisplay:SetSize(368,20);
	inventoryWindow.CapacityDisplay:SetPosition((inventoryWindow:GetWidth()-inventoryWindow.CapacityDisplay:GetWidth())/2,64);
	inventoryWindow.CapacityDisplay:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleCenter);
	inventoryWindow.CapacityDisplay:SetFont(Turbine.UI.Lotro.Font.TrajanPro16);
	inventoryWindow.CapacityDisplay:SetForeColor(fontColor);
	inventoryWindow.CapacityDisplay:SetOutlineColor(Turbine.UI.Color(0,0,0));
	inventoryWindow.CapacityDisplay:SetFontStyle(Turbine.UI.FontStyle.Outline);
	inventoryWindow.CapacityDisplay:SetText(Resource[language][5]);
	inventoryWindow.CapacityFull=Turbine.UI.Label();
	inventoryWindow.CapacityFull:SetParent(inventoryWindow);
	inventoryWindow.CapacityFull:SetSize(60,20);
	inventoryWindow.CapacityFull:SetPosition(inventoryWindow:GetWidth()-inventoryWindow.CapacityEmpty:GetLeft()-66,64);
	inventoryWindow.CapacityFull:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleRight);
	inventoryWindow.CapacityFull:SetFont(Turbine.UI.Lotro.Font.TrajanPro16);
	inventoryWindow.CapacityFull:SetForeColor(Turbine.UI.Color(1,0,0));
	inventoryWindow.CapacityFull:SetOutlineColor(Turbine.UI.Color(0,0,0));
	inventoryWindow.CapacityFull:SetFontStyle(Turbine.UI.FontStyle.Outline);
	inventoryWindow.CapacityFull:SetText(Resource[language][6]);

	inventoryWindow.CharCaption=Turbine.UI.Label();
	inventoryWindow.CharCaption:SetParent(inventoryWindow);
	inventoryWindow.CharCaption:SetSize(145,20);
	inventoryWindow.CharCaption:SetPosition(0,105);
	inventoryWindow.CharCaption:SetFont(Turbine.UI.Lotro.Font.Verdana18);
	inventoryWindow.CharCaption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight);
	inventoryWindow.CharCaption:SetOutlineColor(Turbine.UI.Color(0,0,0));
	inventoryWindow.CharCaption:SetFontStyle(Turbine.UI.FontStyle.Outline);
	inventoryWindow.CharCaption:SetForeColor(Settings.fontColor);
	inventoryWindow.CharCaption:SetText(Resource[language][13]..":");
	inventoryWindow.CharList=DropDownList();
	inventoryWindow.CharList:SetParent(inventoryWindow);
	inventoryWindow.CharList:SetZOrder(1);
	inventoryWindow.CharList:SetSize(230,20);
	inventoryWindow.CharList:SetFont(Turbine.UI.Lotro.Font.Verdana18);
	inventoryWindow.CharList:SetPosition(150,105);
	inventoryWindow.CharList:SetBackColor(backColor);
	inventoryWindow.CharList:SetCurrentBackColor(backColor);
	inventoryWindow.CharList:SetBorderColor(backColor);
	inventoryWindow.CharList.Update=function()
		-- gettings a collision on update handlers. this delays the final hander until the prior handler(s) are complete
		-- *** this should be able to be cleaned up if the collision is straightened out - seems some initialization is getting overwritten
		if updatecount==nil then updatecount=0 end
		updatecount=updatecount+1
		if updatecount>2 then
			updatecount=nil
			getItemEntryLayout()
			applyItemEntryLayout()
			inventoryPanel:Layout()
			inventoryWindow.CharList:SetWantsUpdates(false)
		end
	end
	inventoryWindow.CharList.SelectedIndexChanged=function()
		if inventoryWindow.CurrentContainer~=nil then
			destroyOldItemEntries(inventoryWindow.CurrentContainer) -- need to clear the display due to stretchmode artifacts being left on screen
		end
		local currentSelectedIndex=inventoryWindow.CharList:GetSelectedIndex()
		inventoryWindow.CurrentContainer=getCurrentContainer()
		updateDisplayTabXref() -- force update of xref in case the display type changed
		inventoryWindow.QuickSelectBags:SetVisible(inventoryWindow.currentCharBagsIndex~=currentSelectedIndex)
		if currentSelectedIndex==1 then
			-- all display
			inventoryWindow.CapacityDisplay:SetText(Resource[language][10])
			inventoryWindow.CapacityPanel:SetVisible(false);
			equipmentPanel:SetVisible(false)
			allEIPanel:SetVisible(false)
			inventoryPanel:SetVisible(true)
			inventoryPanel:Refresh();
			inventoryWindow.RemoveCharButton:SetVisible(false);
			minimalWindow.DelButton:SetVisible(false);
		elseif currentSelectedIndex==3 then
			-- allEI panel
			inventoryWindow.CapacityDisplay:SetText(Resource[language][10])
			inventoryWindow.CapacityPanel:SetVisible(false);
			inventoryPanel:SetVisible(false)
			equipmentPanel:SetVisible(false)
			allEIPanel:SetVisible(true)
			allEIPanel:Refresh()
			inventoryWindow.RemoveCharButton:SetVisible(false)
			minimalWindow.DelButton:SetVisible(false)
		elseif string.find(inventoryWindow.CharList:GetText(),"(EI)") ~=nil then
			-- equipped items display
			inventoryWindow.CapacityDisplay:SetText(Resource[language][10])
			inventoryWindow.CapacityPanel:SetVisible(false);
			inventoryPanel:SetVisible(false)
			allEIPanel:SetVisible(false)
			equipmentPanel:SetVisible(true)
			equipmentPanel:Refresh()
			inventoryWindow.RemoveCharButton:SetVisible(false);
			minimalWindow.DelButton:SetVisible(false);
		else
			-- bags, vault, or shared storage display
			local filled,capacity
			if currentSelectedIndex==2 then
				filled=tonumber(CharList["Shared Storage"].used);
				capacity=tonumber(CharList["Shared Storage"].capacity);
				inventoryWindow.RemoveCharButton:SetVisible(false);
				minimalWindow.DelButton:SetVisible(false);
			else
				filled=nil
				if CharList[inventoryWindow.CharList:GetText()].used~=nil then
					filled=tonumber(CharList[inventoryWindow.CharList:GetText()].used)
				end
				capacity=nil
				if CharList[inventoryWindow.CharList:GetText()].capacity~=nil then
					capacity=tonumber(CharList[inventoryWindow.CharList:GetText()].capacity)
				end
				local char=inventoryWindow.CharList:GetText();
				if char~=charName and string.find(char,"(Vault)")==nil then
					inventoryWindow.RemoveCharButton:SetVisible(true);
					minimalWindow.DelButton:SetVisible(true);
				else
					inventoryWindow.RemoveCharButton:SetVisible(false);
					minimalWindow.DelButton:SetVisible(false);
				end
			end
			inventoryWindow.CapacityDisplay:SetText(Resource[language][5].." ("..tostring(filled).."/"..tostring(capacity)..")")
			if filled==nil then filled=0 end
			if capacity==nil then capacity=1 end
			inventoryWindow.CapacityPanel:SetSize(inventoryWindow.CapacityPanel.MaxWidth*(filled/capacity),17);
			inventoryWindow.CapacityPanel:SetBackColor(Turbine.UI.Color(0,.3,0))
			if (filled/capacity)>=.75 then
				inventoryWindow.CapacityPanel:SetBackColor(Turbine.UI.Color(.3,0,0))
			elseif (filled/capacity)>=.5 then
				inventoryWindow.CapacityPanel:SetBackColor(Turbine.UI.Color(.75,.65,0))
			end
			inventoryWindow.CapacityPanel:SetVisible(true);
			equipmentPanel:SetVisible(false)
			allEIPanel:SetVisible(false)
			inventoryPanel:SetVisible(true)
			inventoryPanel:Refresh();
		end
		minimalWindow.CharText:SetText(Resource[language][13]..": "..inventoryWindow.CharList:GetText());
		ApplySearch()
		inventoryWindow.CharList:SetWantsUpdates(true)
--*** 11/05/2025
--		getItemEntryLayout()
--		applyItemEntryLayout()
--		inventoryPanel:Layout()
	end
	inventoryWindow.RemoveCharButton=Turbine.UI.Control();
	inventoryWindow.RemoveCharButton:SetParent(inventoryWindow);
	inventoryWindow.RemoveCharButton:SetPosition(inventoryWindow.RemoveCharButton:GetLeft()+inventoryWindow.RemoveCharButton:GetWidth()+12,inventoryWindow.CharList:GetTop());
	inventoryWindow.RemoveCharButton:SetSize(20,20);
	inventoryWindow.RemoveCharButton:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.RemoveCharButton:SetBackground("GaranStuff/AltInventory/Resources/skull.tga");
	inventoryWindow.RemoveCharButton:SetVisible(true);
	inventoryWindow.RemoveCharButton.MouseEnter = function()
		inventoryWindow.RemoveCharButton:SetBackground("GaranStuff/AltInventory/Resources/skull_hover.tga");
	end
	inventoryWindow.RemoveCharButton.MouseLeave = function()
		inventoryWindow.RemoveCharButton:SetBackground("GaranStuff/AltInventory/Resources/skull.tga");
	end
	inventoryWindow.RemoveCharButton.MouseHover = function()
		local menuItems=toolTip:GetItems()
		menuItems:Clear()
		menuItems:Add(Turbine.UI.MenuItem(Resource[language][136]))
		toolTip:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
	end
	inventoryWindow.RemoveCharButton.MouseClick = function()
			-- display confirmation
		inventoryWindow:SetEnabled(false);
		inventoryPanel.RemoveConfirmation:SetPosition((inventoryPanel.RemoveConfirmation:GetParent():GetWidth()-inventoryPanel.RemoveConfirmation:GetWidth())/2,(inventoryPanel.RemoveConfirmation:GetParent():GetHeight()-inventoryPanel.RemoveConfirmation:GetHeight())/2);
		inventoryPanel.RemoveConfirmation:SetVisible(true);
	end

	inventoryWindow.QuickSelectBags=Turbine.UI.Control();
	inventoryWindow.QuickSelectBags:SetParent(inventoryWindow);
	inventoryWindow.QuickSelectBags:SetPosition(inventoryWindow.CharList:GetLeft()+inventoryWindow.CharList:GetWidth()+12,inventoryWindow.CharList:GetTop());
	inventoryWindow.QuickSelectBags:SetSize(20,20);
	inventoryWindow.QuickSelectBags:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.QuickSelectBags:SetBackground("GaranStuff/AltInventory/Resources/quickbag.tga");
	inventoryWindow.QuickSelectBags:SetVisible(false);
	inventoryWindow.QuickSelectBags.MouseEnter = function()
		inventoryWindow.QuickSelectBags:SetBackground("GaranStuff/AltInventory/Resources/quickbag_hover.tga");
	end
	inventoryWindow.QuickSelectBags.MouseLeave = function()
		inventoryWindow.QuickSelectBags:SetBackground("GaranStuff/AltInventory/Resources/quickbag.tga");
	end
	inventoryWindow.QuickSelectBags.MouseHover = function()
		local menuItems=toolTip:GetItems()
		menuItems:Clear()
		menuItems:Add(Turbine.UI.MenuItem(Resource[language][137]))
		toolTip:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
	end
	inventoryWindow.QuickSelectBags.MouseClick = function()
		if inventoryWindow.currentCharBagsIndex~=nil then
			inventoryWindow.CharList:SetSelectedIndex(inventoryWindow.currentCharBagsIndex)
		end
	end

	inventoryWindow.BottomBack=Turbine.UI.Control()
	inventoryWindow.BottomBack:SetParent(inventoryWindow);
	inventoryWindow.BottomBack:SetSize(inventoryWindow:GetWidth()-20,40);
	inventoryWindow.BottomBack:SetPosition(10,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomBack:SetBackColor(Turbine.UI.Color(0,0,0));
	inventoryWindow.BottomBack:SetMouseVisible(false);
	inventoryWindow.BottomPanelTiled=Turbine.UI.Control();
	inventoryWindow.BottomPanelTiled:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelTiled:SetSize(inventoryWindow:GetWidth()-20,40);
	inventoryWindow.BottomPanelTiled:SetPosition(10,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelTiled:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.BottomPanelTiled:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
	inventoryWindow.BottomPanelTiled:SetBackColor(Settings.panelBackColor);
	inventoryWindow.BottomPanelTiled:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_tiled_bw.tga");
	inventoryWindow.BottomPanelTiled:SetMouseVisible(false);

	inventoryWindow.BottomPanelTiledTrim=Turbine.UI.Control()
	inventoryWindow.BottomPanelTiledTrim:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelTiledTrim:SetSize(inventoryWindow:GetWidth()-20,5);
	inventoryWindow.BottomPanelTiledTrim:SetPosition(10,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelTiledTrim:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_tiled_trim.tga");
	inventoryWindow.BottomPanelTiledTrim:SetMouseVisible(false);

	inventoryWindow.BottomPanelLeft=Turbine.UI.Control();
	inventoryWindow.BottomPanelLeft:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelLeft:SetSize(151,40);
	inventoryWindow.BottomPanelLeft:SetPosition(5,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelLeft:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.BottomPanelLeft:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
	inventoryWindow.BottomPanelLeft:SetBackColor(Settings.panelBackColor);
	inventoryWindow.BottomPanelLeft:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_left.tga");
	inventoryWindow.BottomPanelLeft:SetMouseVisible(false);

	inventoryWindow.BottomPanelLeftTrim=Turbine.UI.Control();
	inventoryWindow.BottomPanelLeftTrim:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelLeftTrim:SetSize(151,40);
	inventoryWindow.BottomPanelLeftTrim:SetPosition(5,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelLeftTrim:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.BottomPanelLeftTrim:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_left_trim.tga");
	inventoryWindow.BottomPanelLeftTrim:SetMouseVisible(false);

	inventoryWindow.BottomPanelCenter=Turbine.UI.Control();
	inventoryWindow.BottomPanelCenter:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelCenter:SetSize(121,40);
	inventoryWindow.BottomPanelCenter:SetPosition((inventoryWindow:GetWidth()-inventoryWindow.BottomPanelCenter:GetWidth())/2+1,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelCenter:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.BottomPanelCenter:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_center.tga");
	inventoryWindow.BottomPanelCenter:SetMouseVisible(false);

	inventoryWindow.BottomPanelRight=Turbine.UI.Control();
	inventoryWindow.BottomPanelRight:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelRight:SetSize(151,40);
	inventoryWindow.BottomPanelRight:SetPosition(inventoryWindow:GetWidth()-inventoryWindow.BottomPanelRight:GetWidth()-4,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelRight:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.BottomPanelRight:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
	inventoryWindow.BottomPanelRight:SetBackColor(Settings.panelBackColor);
	inventoryWindow.BottomPanelRight:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_right.tga");
	inventoryWindow.BottomPanelRight:SetMouseVisible(false);

	inventoryWindow.BottomPanelRightTrim=Turbine.UI.Control();
	inventoryWindow.BottomPanelRightTrim:SetParent(inventoryWindow);
	inventoryWindow.BottomPanelRightTrim:SetSize(151,40);
	inventoryWindow.BottomPanelRightTrim:SetPosition(inventoryWindow:GetWidth()-inventoryWindow.BottomPanelRight:GetWidth()-4,inventoryWindow:GetHeight()-inventoryWindow.BottomMargin);
	inventoryWindow.BottomPanelRightTrim:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.BottomPanelRightTrim:SetBackground("GaranStuff/AltInventory/Resources/background_bottom_right_trim.tga");
	inventoryWindow.BottomPanelRightTrim:SetMouseVisible(false);

	inventoryWindow.OptionsButton=Turbine.UI.Control();
	inventoryWindow.OptionsButton:SetParent(inventoryWindow);
	inventoryWindow.OptionsButton:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.OptionsButton:SetBackground("GaranStuff/AltInventory/Resources/options.tga");
	inventoryWindow.OptionsButton:SetSize(32,32);
	inventoryWindow.OptionsButton:SetTop(inventoryWindow.BottomPanelLeft:GetTop()+6);
	inventoryWindow.OptionsButton:SetLeft(inventoryWindow.BottomPanelLeft:GetLeft()+25);
	inventoryWindow.OptionsButton.MouseEnter = function()
		inventoryWindow.OptionsButton:SetBackground("GaranStuff/AltInventory/Resources/options_hover.tga");
	end
	inventoryWindow.OptionsButton.MouseLeave = function()
		inventoryWindow.OptionsButton:SetBackground("GaranStuff/AltInventory/Resources/options.tga");
	end
	inventoryWindow.OptionsButton.MouseClick=function()
		setupWindow:SetVisible(true);
		setupWindow:Refresh();
	end

	inventoryWindow.ItemExplorerButton=Turbine.UI.Control();
	inventoryWindow.ItemExplorerButton:SetParent(inventoryWindow);
	inventoryWindow.ItemExplorerButton:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	inventoryWindow.ItemExplorerButton:SetBackground("GaranStuff/AltInventory/Resources/itemexplorer.tga");
	inventoryWindow.ItemExplorerButton:SetSize(32,32);
	inventoryWindow.ItemExplorerButton:SetTop(inventoryWindow.BottomPanelLeft:GetTop()+6);
	inventoryWindow.ItemExplorerButton:SetLeft(inventoryWindow.OptionsButton:GetLeft()+inventoryWindow.OptionsButton:GetWidth()+10);
	inventoryWindow.ItemExplorerButton.MouseEnter=function()
		inventoryWindow.ItemExplorerButton:SetBackground("GaranStuff/AltInventory/Resources/itemexplorer_hover.tga");
	end
	inventoryWindow.ItemExplorerButton.MouseLeave=function()
		inventoryWindow.ItemExplorerButton:SetBackground("GaranStuff/AltInventory/Resources/itemexplorer.tga");
	end
	inventoryWindow.ItemExplorerButton.MouseClick=function()
		itemExplorer:SetVisible(true)
	end

	inventoryWindow.Message=Turbine.UI.Label();
	inventoryWindow.Message:SetParent(inventoryWindow);
	inventoryWindow.Message:SetZOrder(1);
	inventoryWindow.Message:SetSize(inventoryWindow:GetWidth()-inventoryWindow.ItemExplorerButton:GetLeft()-inventoryWindow.ItemExplorerButton:GetWidth()-40,40);
	inventoryWindow.Message:SetPosition(inventoryWindow.ItemExplorerButton:GetLeft()+inventoryWindow.ItemExplorerButton:GetWidth()+5,inventoryWindow.ItemExplorerButton:GetTop())
	inventoryWindow.Message:SetForeColor(Turbine.UI.Color(1,0,0));
	inventoryWindow.Message:SetOutlineColor(Turbine.UI.Color(0,0,0));
	inventoryWindow.Message:SetFontStyle(Turbine.UI.FontStyle.Outline);

	inventoryWindow.FontSelect=FontSelect()
	inventoryWindow.FontSelect:SetParent(inventoryWindow)
--	inventoryWindow.FontSelect:SetStretchMode(3) -- this was causing issues with the graphic not displaying
	inventoryWindow.FontSelect:SetBackground(resourcePath.."FontSelect.tga")
	inventoryWindow.FontSelect:SetZOrder(2)
	inventoryWindow.FontSelect:SetPosition(inventoryWindow:GetWidth()-65,inventoryWindow.BottomPanelLeft:GetTop()+10)
	inventoryWindow.FontSelect:SetFont(Settings.fontFace)
	inventoryWindow.FontSelect.MouseHover=function()
		local tmpMenu=Turbine.UI.ContextMenu()
		tmpItems=tmpMenu:GetItems()
		tmpItems:Clear() -- should already be clear
		tmpItems:Add(Turbine.UI.MenuItem(Resource[language][131]))
		tmpItems:Get(1):SetEnabled(false)
		tmpMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX()-15,Turbine.UI.Display:GetMouseY()-30)
	end
	FontSelectList:SetStretchMode(3)
	inventoryWindow.FontSelect.FontChanged=function()
		if not inventoryWindow.settingFont then
			inventoryWindow.settingFont=true
			Settings.fontFace=inventoryWindow.FontSelect:GetFont()
			-- synch the setupWindow fontSelect control
			setupWindow.FontSelect:SetFont(Settings.fontFace)
			-- now need to apply new font... :p
			fontMetric:SetFont(Settings.fontFace)
			getItemEntryLayout()
			applyItemEntryLayout()
			inventoryPanel:Layout()
			inventoryPanel.cropDelay=Settings.cropDelay
			inventoryPanel:SetWantsUpdates(true)
			inventoryWindow.settingFont=false
		end
	end

	inventoryWindow.SizeChanged=function()
		if not inventoryWindow.InSizeChanged then
			if Settings.zoom==1 and not inventoryWindow.InSizeChanged then
				inventoryWindow.InSizeChanged=true
				-- might have been externally triggered by resizing, set the width and height just in case
				inventoryWindow:SetSize(Turbine.UI.Lotro.Window.GetSize(inventoryWindow))
				inventoryWindow.InSizeChanged=false
			end
			local width,height=inventoryWindow:GetSize();
			Settings.panelWidth=(inventoryWindow:GetWidth()-inventoryWindow.LeftMargin*2*Settings.zoom)/displayWidth
			Settings.panelHeight=(inventoryWindow:GetHeight()-(inventoryWindow.TopMargin+inventoryWindow.BottomMargin)*Settings.zoom)/displayHeight
	
			inventoryWindow.BottomPanelTiled:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomPanelTiledTrim:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomBack:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomPanelLeft:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomPanelLeftTrim:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomPanelCenter:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomPanelRight:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.BottomPanelRightTrim:SetTop(height-inventoryWindow.BottomMargin);
			inventoryWindow.OptionsButton:SetTop(inventoryWindow.BottomPanelLeft:GetTop()+6);
			inventoryWindow.ItemExplorerButton:SetTop(inventoryWindow.BottomPanelLeft:GetTop()+6);
			inventoryWindow.FontSelect:SetPosition(inventoryWindow:GetWidth()-65,inventoryWindow.BottomPanelLeft:GetTop()+10)
			inventoryWindow.Message:SetWidth(inventoryWindow:GetWidth()-inventoryWindow.ItemExplorerButton:GetLeft()-inventoryWindow.ItemExplorerButton:GetWidth()-40);
			inventoryWindow.Message:SetTop(inventoryWindow.ItemExplorerButton:GetTop())
			if language==1 then
				inventoryWindow.TitleIcon:SetPosition(inventoryWindow:GetWidth()/2-104,5);
			elseif language==2 then
				inventoryWindow.TitleIcon:SetPosition(inventoryWindow:GetWidth()/2-119,5);
			else
				inventoryWindow.TitleIcon:SetPosition(inventoryWindow:GetWidth()/2-100,5);
			end
			inventoryWindow.TopPanelTiled:SetWidth(width-30);
			inventoryWindow.TopPanelTiledTrim:SetWidth(width-30);
			inventoryWindow.TopPanelCenter:SetLeft((inventoryWindow:GetWidth()-inventoryWindow.TopPanelCenter:GetWidth())/2-3);
			inventoryWindow.TopPanelRight:SetLeft(inventoryWindow:GetWidth()-inventoryWindow.TopPanelRight:GetWidth()-5);
			inventoryWindow.TopPanelRightTrim:SetLeft(inventoryWindow:GetWidth()-inventoryWindow.TopPanelRightTrim:GetWidth()-5);
			inventoryWindow.CloseButton:SetLeft(inventoryWindow:GetWidth()-inventoryWindow.CloseButton:GetWidth()-6);
			inventoryWindow.RemoveCharButton:SetLeft(inventoryWindow.CharList:GetLeft()+inventoryWindow.CharList:GetWidth()+12)
			inventoryWindow.QuickSelectBags:SetLeft(inventoryWindow.RemoveCharButton:GetLeft()+inventoryWindow.RemoveCharButton:GetWidth()+12)

			inventoryWindow.CapacityPanel:SetLeft((inventoryWindow:GetWidth()-inventoryWindow.CapacityPanel.MaxWidth)/2);
			inventoryWindow.CapacityEmpty:SetLeft(inventoryWindow.CapacityPanel:GetLeft()+3)
			inventoryWindow.CapacityFull:SetLeft(inventoryWindow:GetWidth()-inventoryWindow.CapacityEmpty:GetLeft()-inventoryWindow.CapacityFull:GetWidth()-6,64)
			inventoryWindow.CapacityDisplay:SetLeft((inventoryWindow:GetWidth()-inventoryWindow.CapacityDisplay:GetWidth())/2)

			inventoryWindow.BottomPanelTiled:SetWidth(inventoryWindow:GetWidth()-20);
			inventoryWindow.BottomPanelTiledTrim:SetWidth(inventoryWindow:GetWidth()-20);
			inventoryWindow.BottomBack:SetWidth(inventoryWindow:GetWidth()-20);
			inventoryWindow.BottomPanelCenter:SetLeft((inventoryWindow:GetWidth()-inventoryWindow.BottomPanelCenter:GetWidth())/2+1);
			inventoryWindow.BottomPanelRight:SetLeft(inventoryWindow:GetWidth()-inventoryWindow.BottomPanelRight:GetWidth()-4);
			inventoryWindow.BottomPanelRightTrim:SetLeft(inventoryWindow:GetWidth()-inventoryWindow.BottomPanelRightTrim:GetWidth()-4);

			inventoryPanel:Layout()
			equipmentPanel:Layout()
			allEIPanel:Layout()
		end
	end
end
createInventoryWindow()
function createMinimalWindow()
	minimalWindow=nil
	-- this is the "minimal" display
	minimalWindow=Turbine.UI.Window()
	minimalWindow:SetVisible(Settings.useMinimalHeader and not Settings.loadMinimized)
	minimalWindow:SetBackColor(Settings.backColor);
	minimalWindow.LeftMargin=3;
	minimalWindow.TopMargin=20;
	minimalWindow.BottomMargin=3;
	minimalWindow.minimumWidth=Settings.panelMinWidth*displayWidth+minimalWindow.LeftMargin*2;
	minimalWindow.minimumHeight=Settings.panelMinHeight*displayHeight+minimalWindow.TopMargin+minimalWindow.BottomMargin;
	minimalWindow:SetSize(Settings.panelWidth*displayWidth+minimalWindow.LeftMargin*2*Settings.zoom,Settings.panelHeight*displayHeight+(minimalWindow.TopMargin+minimalWindow.BottomMargin)*Settings.zoom);
	local top=Settings.panelTop*displayHeight-minimalWindow.TopMargin*Settings.zoom
	local left=Settings.panelLeft*displayWidth-minimalWindow.LeftMargin*Settings.zoom
	if left<0 then left=0 end
	if top<0 then top=0 end
	minimalWindow:SetPosition(left,top)
	minimalWindow.CharMenu=Turbine.UI.ContextMenu();
	minimalWindow.MoveLabel=Turbine.UI.Label();
	minimalWindow.MoveLabel:SetParent(minimalWindow);
	minimalWindow.MoveLabel:SetMouseVisible(false);
	minimalWindow.MoveLabel:SetFont(Turbine.UI.Lotro.Font.Arial12)
	minimalWindow.MoveLabel:SetText("<>");
	minimalWindow.CharText=Turbine.UI.Label();
	minimalWindow.CharText:SetParent(minimalWindow);
	minimalWindow.CharText:SetSize(minimalWindow:GetWidth()-280,20);
	minimalWindow.CharText:SetPosition(140,0);
	minimalWindow.CharText:SetForeColor(Settings.fontColor);
	minimalWindow.CharText:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter);
	minimalWindow.CharText:SetText(Resource[language][13]..": "..charName);

	minimalWindow.CharText.MouseClick=function()
		minimalWindow.CharMenu:ShowMenu();
	end
	minimalWindow.DelButton=Turbine.UI.Label();
	minimalWindow.DelButton:SetParent(minimalWindow);
	minimalWindow.DelButton:SetSize(20,20);
	minimalWindow.DelButton:SetPosition(minimalWindow:GetWidth()-120,0);
	minimalWindow.DelButton:SetBackground("GaranStuff/AltInventory/Resources/skull.tga")
	minimalWindow.DelButton.MouseEnter = function()
		minimalWindow.DelButton:SetBackground("GaranStuff/AltInventory/Resources/skull_hover.tga");
	end
	minimalWindow.DelButton.MouseLeave = function()
		minimalWindow.DelButton:SetBackground("GaranStuff/AltInventory/Resources/skull.tga");
	end

	minimalWindow.DelButton.MouseClick=function()
		inventoryWindow.RemoveCharButton:MouseClick();
	end

	minimalWindow.OptionsButton=Turbine.UI.Label();
	minimalWindow.OptionsButton:SetParent(minimalWindow);
	minimalWindow.OptionsButton:SetSize(100,20);
	minimalWindow.OptionsButton:SetPosition(minimalWindow:GetWidth()-minimalWindow.OptionsButton:GetWidth(),0);
	minimalWindow.OptionsButton:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter);
	minimalWindow.OptionsButton:SetText(Resource[language][48]);
	minimalWindow.OptionsButton.MouseClick=function()
		setupWindow:SetVisible(true);
		setupWindow:Refresh();
	end

	minimalWindow.CloseButton=Turbine.UI.Label();
	minimalWindow.CloseButton:SetParent(minimalWindow);
	minimalWindow.CloseButton:SetSize(16,16);
	minimalWindow.CloseButton:SetPosition(minimalWindow:GetWidth()-18,2);
	minimalWindow.CloseButton:SetBackground(0x41000196)
	minimalWindow.CloseButton.MouseEnter=function()
		minimalWindow.CloseButton:SetBackground(0x41000198);
	end
	minimalWindow.CloseButton.MouseLeave=function()
		minimalWindow.CloseButton:SetBackground(0x41000196);
	end
	minimalWindow.CloseButton.MouseDown=function()
		minimalWindow.CloseButton:SetBackground(0x41000197);
	end
	minimalWindow.CloseButton.MouseClick=function()
		minimalWindow:Close();
		if Settings.showIcon==1 or Settings.showIcon==2 then
			iconWindow:SetVisible(true);
		end
	end

	minimalWindow.Moving=false;
	minimalWindow.MovingIcon=Turbine.UI.Control();
	minimalWindow.MovingIcon:SetParent(minimalWindow);
	minimalWindow.MovingIcon:SetSize(32,32);
	minimalWindow.MovingIcon:SetBackground(0x410081c0)
	minimalWindow.MovingIcon:SetStretchMode(1);
	-- for some reason, setting the control's position or size AFTER loading the resource background makes it appear... WEIRD
	minimalWindow.MovingIcon:SetPosition(minimalWindow:GetWidth()/2-15,minimalWindow:GetHeight()-21);
	minimalWindow.MovingIcon:SetVisible(false);

-- old sizing stuff - keeping this for now since it probably works better than the generic AND includes a Move feature. just need to tie it into the overridden SetSize
	minimalWindow.SizingV=false
	minimalWindow.SizingH=false
	minimalWindow.MouseDown=function(sender,args)
		minimalWindow.StartX=args.X;
		minimalWindow.StartY=args.Y;
		if (args.Y<16) then
			-- moving
			minimalWindow.Moving=true
			minimalWindow.MovingIcon:SetLeft(args.X-12);
			minimalWindow.MovingIcon:SetTop(args.Y-12);
			minimalWindow.MovingIcon:SetSize(32,32);
			minimalWindow.MovingIcon:SetBackground(0x410000dd)
			minimalWindow.MovingIcon:SetVisible(true);
		else
			if minimalWindow:IsAltKeyDown() or minimalWindow:IsShiftKeyDown() or minimalWindow:IsControlKeyDown() then
				minimalWindow.StartH=minimalWindow:GetWidth()
				minimalWindow.StartV=minimalWindow:GetHeight()
				if (args.Y>minimalWindow:GetHeight()-15) and (args.X>minimalWindow:GetWidth()-15) then
					-- diagonal sizing
					minimalWindow.SizingV=true
					minimalWindow.SizingH=true
					minimalWindow.MovingIcon:SetLeft(args.X-12);
					minimalWindow.MovingIcon:SetTop(args.Y-12);
					minimalWindow.MovingIcon:SetSize(32,32);
					minimalWindow.MovingIcon:SetBackground(0x41007e20)
					minimalWindow.MovingIcon:SetVisible(true);
				elseif (args.Y>minimalWindow:GetHeight()-12) then
					-- vertical sizing
					minimalWindow.SizingV=true
					minimalWindow.MoveY=args.Y;
					minimalWindow.MovingIcon:SetLeft(args.X-22);
					minimalWindow.MovingIcon:SetTop(args.Y-12);
					minimalWindow.MovingIcon:SetSize(32,32);
					minimalWindow.MovingIcon:SetBackground(0x410081c0)
					minimalWindow.MovingIcon:SetVisible(true);
				elseif (args.X>minimalWindow:GetWidth()-12) then
					-- horizontal sizing
					minimalWindow.SizingH=true
					minimalWindow.MoveX=args.X;
					minimalWindow.MovingIcon:SetLeft(args.X-12);
					minimalWindow.MovingIcon:SetTop(args.Y-22);
					minimalWindow.MovingIcon:SetSize(32,32);
					minimalWindow.MovingIcon:SetBackground(0x410081bf)
					minimalWindow.MovingIcon:SetVisible(true);
					minimalWindow.MoveX=-1;
					minimalWindow.MoveY=-1;
				end
			end
		end
	end

	minimalWindow.MouseMove=function(sender,args)
		if minimalWindow.Moving then
			local newLeft=minimalWindow:GetLeft()+(args.X-minimalWindow.StartX)*Settings.zoom
			local newTop=minimalWindow:GetTop()+(args.Y-minimalWindow.StartY)*Settings.zoom
			if newLeft<0 then newLeft=0 end;
			if newLeft>(Turbine.UI.Display.GetWidth()-minimalWindow:GetWidth()) then newLeft=Turbine.UI.Display.GetWidth()-minimalWindow:GetWidth() end;
			if newTop<0 then newTop=0 end;
			if newTop>(Turbine.UI.Display.GetHeight()-minimalWindow:GetHeight()) then newTop=Turbine.UI.Display.GetHeight()-minimalWindow:GetHeight() end;
			minimalWindow:SetPosition(newLeft,newTop)		
		elseif minimalWindow.SizingV or minimalWindow.SizingH then
			local newHeight=minimalWindow:GetHeight()
			local newWidth=minimalWindow:GetWidth()
			if minimalWindow.SizingV then
				newHeight=minimalWindow.StartV+(args.Y-minimalWindow.StartY)*Settings.zoom
				if newHeight<minimalWindow:GetMinimumHeight() then newHeight=minimalWindow:GetMinimumHeight() end
				if newHeight>(Turbine.UI.Display.GetHeight()-minimalWindow:GetTop()) then newHeight=Turbine.UI.Display.GetHeight()-minimalWindow:GetTop() end;
				minimalWindow.MovingIcon:SetTop(args.Y-22)
			end
			if minimalWindow.SizingH then
				newWidth=minimalWindow.StartH+(args.X-minimalWindow.StartX)*Settings.zoom
				if newWidth<minimalWindow:GetMinimumWidth() then newWidth=minimumWindow:GetMinimumWidth() end
				if newWidth>(Turbine.UI.Display.GetWidth()-minimalWindow:GetLeft()) then newWidth=Turbine.UI.Display.GetWidth()-minimalWindow:GetLeft() end;
				minimalWindow.MovingIcon:SetLeft(args.X-22)
			end
			minimalWindow:SetSize(newWidth,newHeight)
		end
	end
	minimalWindow.MouseUp=function(sender,args)
		minimalWindow.Moving=false
		minimalWindow.SizingV=false
		minimalWindow.SizingH=false
		minimalWindow.MovingIcon:SetVisible(false);
	end
	minimalWindow.SizeChanged=function()
		local width,height=minimalWindow:GetSize();
		Settings.panelWidth=(minimalWindow:GetWidth()-minimalWindow.LeftMargin*2*Settings.zoom)/displayWidth
		Settings.panelHeight=(minimalWindow:GetHeight()-(minimalWindow.TopMargin+minimalWindow.BottomMargin)*Settings.zoom)/displayHeight
		minimalWindow.OptionsButton:SetLeft(width-minimalWindow.OptionsButton:GetWidth()-20);
		minimalWindow.CharText:SetWidth(width-280);
		minimalWindow.DelButton:SetLeft(width-140);
		minimalWindow.CloseButton:SetLeft(width-18);
		inventoryPanel:Layout()
		equipmentPanel:Layout()
		allEIPanel:Layout()
	end
end
createMinimalWindow()

--********************* Group/Item functions *********************************************************
function cropAllIcons()
	local container=getCurrentContainer()
	for k,v in ipairs(displayTabs[container]) do
		if v.docked then
			cropPanelIcons(v.panel)
		end
	end
end
function cropPanelIcons(panel)
	local dt=panel.tabHandle
	local panelTop=panel:GetTop()
	-- no need for crop if zoom==1

	if Settings.zoom>1 then
		local min=-32*Settings.zoom/2
		local max=inventoryPanel.ItemList:GetHeight()-32*Settings.zoom
		for k=1,panel.ItemList:GetItemCount() do
			local item=panel.ItemList:GetItem(k)
			if not dt.expanded then
				item.IconBack:SetVisible(false)
				item.IconBack:SetHeight(0)
			else
				local top=panelTop+item:GetTop()
				if item.isFiltered or top<min or top>max then
					item.IconBack:SetVisible(false)
					item.IconBack:SetHeight(0)
				else
					item.IconBack:SetVisible(true)
					item.IconBack:SetSize(32*Settings.zoom,32*Settings.zoom)
				end
			end
		end
	end
end
function getItemEntryLayout()
	-- determine the settings for ItemEntryIconVisible, ItemEntryTextVisible, ItemEntryIconWidth, ItemEntryIconTop, ItemEntryTextWidth, ItemEntryTextTop, ItemEntryBackWidth, ItemEntryBackHeight
	-- this way, we do the calculations once and can apply them to all ItemEntry instances

	local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
	inventoryWindow.ItemEntryIconWidth=32*Settings.zoom
	local textHeight=fontSize*2
	local newHeight=textHeight
	if Settings.panelViewMode==3 then
		-- icon only
		inventoryWindow.ItemEntryIconVisible=true
		inventoryWindow.ItemEntryTextVisible=false
		inventoryWindow.ItemEntryIconTop=0
		inventoryWindow.ItemEntryBackWidth=inventoryWindow.ItemEntryIconWidth
		inventoryWindow.ItemEntryBackHeight=inventoryWindow.ItemEntryIconWidth
	elseif Settings.panelViewMode==1 then
		-- text only
		fontMetric:SetFont(Settings.fontFace)
		inventoryWindow.ItemEntryIconVisible=false
		inventoryWindow.ItemEntryTextVisible=true
		inventoryWindow.ItemEntryTextLeft=fontMetric:GetTextWidth("888888") -- leave space for qty display
		inventoryWindow.ItemEntryTextTop=0
		inventoryWindow.ItemEntryTextHeight=textHeight
		inventoryWindow.ItemEntryTextWidth=fontMetric:GetTextWidth("MWMWMWMWMWMWMWMWMWMW")
		inventoryWindow.ItemEntryBackHeight=textHeight
		inventoryWindow.ItemEntryBackWidth=inventoryWindow.ItemEntryTextWidth+2+inventoryWindow.ItemEntryTextLeft
	else

		inventoryWindow.ItemEntryIconVisible=true
		inventoryWindow.ItemEntryTextVisible=true
		if inventoryWindow.ItemEntryIconWidth>newHeight then
			--if icon width is > newHeight then icon determines height (since it is square, height must be >= to width )
			newHeight=inventoryWindow.ItemEntryIconWidth
			inventoryWindow.ItemEntryIconTop=0
			inventoryWindow.ItemEntryTextTop=(newHeight-textHeight)/2
		else
			inventoryWindow.ItemEntryIconTop=(newHeight-inventoryWindow.ItemEntryIconWidth)/2
			inventoryWindow.ItemEntryTextTop=0
		end
		inventoryWindow.ItemEntryTextLeft=inventoryWindow.ItemEntryIconWidth+1 -- 1 pixel between icon and text
		fontMetric:SetFont(Settings.fontFace)
		inventoryWindow.ItemEntryTextWidth=fontMetric:GetTextWidth("MWMWMWMWMWMWMWMWMWMW")		
		inventoryWindow.ItemEntryBackWidth=inventoryWindow.ItemEntryTextLeft+inventoryWindow.ItemEntryTextWidth
		inventoryWindow.ItemEntryBackHeight=newHeight
	end
	inventoryWindow.ColumnWidth=inventoryWindow.ItemEntryBackWidth+4
end
function applyItemEntryLayout()
	local container=getCurrentContainer()
	for k,v in ipairs(displayTabs[container]) do
		if v.panel~=nil then
			v.panel:ApplyLayout()
		end
	end
end
function destroyOldItemEntries(container)
	if container==nil then container=getCurrentContainer() end
	-- make sure we don't leave any orphans and hide any old bits until garbage collection gets around to cleaning them up
	for k,v in ipairs(displayTabs[container]) do
		if v.panel~=nil then
			if v.panel.ItemList~=nil then
				for index=1,v.panel.ItemList:GetItemCount() do
					local tmpItem=v.panel.ItemList:GetItem(index)
					if tmpItem~=nil then
						tmpItem.IconBack:SetVisible(false)
					end
				end
				v.panel.ItemList:ClearItems()
			end
			v.panel:SetVisible(false)
			v.panel=nil
		end
		if v.window~=nil then
			v.window:SetVisible(false)
			v.window=nil
		end
	end
end
getCurrentContainer=function(ignoreOverride)
	if ignoreOverride==nil then ignoreOverride=false end
	local container="bags"
	if inventoryWindow.CharList:GetSelectedIndex()==1 then
		container="all" -- for now we use "bags"
	elseif inventoryWindow.CharList:GetSelectedIndex()==2 then
		container="shared"
	elseif string.find(inventoryWindow.CharList:GetText(),"(Vault)")~=nil then
		container="vault"
	end
	if not ignoreOverride then
		-- check for container override
		if displayTabs[container].useBagTags then container="bags" end
	end
	return container
end
getDisplayTabIndexFromName=function(container,name)
	local index
	if container==nil then container=getCurrentContainer() end
	for k,v in ipairs(displayTabs[container]) do
		if v.name==name then
			index=k
			break
		end
	end
	return index
end
priorContainer=nil
deleteGroup=function(container,displayTab)
	if container==nil then
		-- determine current container
		container=getCurrentContainer()
	end

	-- we use the displayTab.Name to get the name
	local index
	for k,v in ipairs(displayTabs[container]) do
		if v.name==displayTab.name then
			index=k
		end
	end
	if index==nil then
		-- popup message indicating error
		popup=PopUpDialog(Resource[language][57],string.gsub(Resource[language][58],"@group",tostring(displayTab.name)),3,Resource[language][61])
	else
		popup=PopUpDialog(Resource[language][59],string.gsub(Resource[language][60],"@group",tostring(displayTab.name)),2,Resource[language][62],Resource[language][63],nil,false,verifiedDeleteGroup)
		popup.container=container
		popup.index=index
	end
end
verifiedDeleteGroup=function()
	-- popup.container is container, popup.index is displayTab index
	local container=popup.container
	local index=popup.index
	displayTabs[container][index].panel:SetVisible(false)
	if displayTabs[container][index].docked then
		inventoryPanel.ItemList:RemoveItem(displayTabs[container][index].panel)
	end
	if displayTabs[container][index].window~=nil then
		displayTabs[container][index].window:SetVisible(false)
	end
	displayTabs[container][index].panel=nil
	displayTabs[container][index].window=nil
	table.remove(displayTabs[container],index)
	-- need to regenerate displayTabXref and then refresh display so that the items that used to be in the deleted tab go elsewhere
	updateDisplayTabXref()
	inventoryPanel:Refresh()
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout()
	if groupMaint:IsVisible() then
		groupMaint:Show()
	end

	-- temporary bug workaround, reload the plugin to fix floating panel issues
	ReloadAltInventory()
end
critContains=function(srcCritTable,str)
	local found=false
	for k,v in pairs(srcCritTable) do
		if string.find(str,v,nil,true)~=nil then
			found=true
			break
		end
	end
	return found
end
updateDisplayTabXref=function(priorOK)
	if priorOK==nil then usePrior=false end
	-- need to rebuild the xref that determines which tab items belong on for CURRENT container
	-- this gets called any time the display container changes since each container type can have its own set of display tabs
	-- assign the default tab based on current container
	local container=getCurrentContainer()
	defaultDisplayTabIndex=nil
	if displayTabs[container][1]~=nil then defaultDisplayTabIndex=1 end -- default it to the first tab
	for k,v in ipairs(displayTabs[container]) do
		if v.isMain then defaultDisplayTabIndex=k end
	end
	displayTabXref={} -- create new empty table - old one will get garbage collected
	-- check if we are using the same container as was previously used (priorContainer will be nil if never run or a container definition was altered)
	if not priorOK or container~=priorContainer then
		for itemName,itemValue in pairs(accountItemQty) do
local tmpDebug=(itemName=="something really bogus") -- replace with a specific item name to track the resolution of dt
			local dtResolved=false
			displayTabXref[itemName]=defaultDisplayTabIndex
			for k,v in ipairs(displayTabs[container]) do
				-- only use pattern match if criteria contains a pattern (pattern match is much slower)
				if v.criteria.ItemNamesContainPattern then
					if critContains(v.criteria.ItemNames,itemName) then
						displayTabXref[itemName]=k
						dtResolved=true
					end
				else
					if Table.Contains(v.criteria.ItemNames,itemName,true) then
						displayTabXref[itemName]=k
						dtResolved=true
					end
				end
if tmpDebug then Turbine.Shell.WriteLine("dtResolved after name:"..tostring(dtResolved)) end
				if not dtResolved then
					local itemInfoName=itemValue["InfoName"]
					if itemInfoName~=nil then
						if v.criteria.ItemInfoNamesContainPattern then
							if critContains(v.criteria.ItemInfoNames,itemInfoName) then
								displayTabXref[itemName]=k
								dtResolved=true
							end
						else
							if Table.Contains(v.criteria.ItemInfoNames,itemInfoName,true) then
								displayTabXref[itemName]=k
								dtResolved=true
							end
						end
					end
if tmpDebug then Turbine.Shell.WriteLine("dtResolved after item name:"..tostring(dtResolved)) end
					if not dtResolved and Table.Contains(v.criteria.CIDValues,itemValue.Category) then
						-- temporarilly assign item to this tab (it may yet be overridden by a specific name criteria in another tab)
						displayTabXref[itemName]=k
					end
				end
			end
		end
		priorContainer=container
	end
end

recalcItemDisplayTabXref=function(itemName,itemCategory)
	local dtResolved=false
	-- forces a re-calculate for the displayTabXref for an item
	local container=getCurrentContainer()
	local retVal=defaultDisplayTabIndex
	local infoNameFound -- track with flag since checking existance of key is less efficient and speed is essential
	for k,v in ipairs(displayTabs[container]) do

		if v.criteria.ItemNamesContainPattern then
			if critContains(v.criteria.ItemNames,itemName) then
				retVal=k
				dtResolved=true
			end
		else
			if Table.Contains(v.criteria.ItemNames,itemName,true) then
				retVal=k
				dtResolved=true
			end
		end
		if not dtResolved then
			-- try resolving name against generic criteria
			if v.criteria.ItemInfoNamesContainPattern then
				if critContains(v.criteria.ItemInfoNames,itemName) then
					retVal=k
					dtResolved=true
				end
			else
				if Table.Contains(v.criteria.ItemInfoNames,itemName,true) then
					retVal=k
					dtResolved=true
				end
			end
			if not dtResolved and Table.Contains(v.criteria.CIDValues,itemCategory) then
				-- temporarilly assign item to this tab (it may yet be overridden by a specific name criteria in another tab)
				retVal=k
			end
		end
	end
	displayTabXref[itemName]=retVal
	return retVal
end
createDTWindow=function(dt)
	dt.window=Turbine.UI.Window()
	dt.window.dt=dt
	dt.window:SetBackColor(Turbine.UI.Color(1,.8,.6,0))

	dt.window.MouseDown=function(sender,args)
		sender.X=args.X
		sender.Y=args.Y
		if args.X>sender:GetWidth()-5 then
			if args.Button==1 then
				sender.sizingX=true
			end
		end
		if args.Y>sender:GetHeight()-5 then
			if args.Button==2 then
				local tmpMenu=Turbine.UI.ContextMenu()
				tmpItems=tmpMenu:GetItems()
				tmpItems:Clear() -- should already be clear
				local menuItem=Turbine.UI.MenuItem(Resource[language][137])
				menuItem.dt=sender.dt
				menuItem.Click=function(sender,args)
					sender.dt.lockHeight=false
					sender.dt.panel.ItemList:Layout()
				end
				tmpItems:Add(menuItem)
				tmpMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX()-15,Turbine.UI.Display:GetMouseY()-30)
			elseif args.Button==1 then
				sender.sizingY=true
			end
		end
	end
	dt.window.MouseUp=function(sender,args)
		sender.sizingX=false
		sender.sizingY=false
	end
	dt.window.MouseMove=function(sender,args)
		if sender.sizingX or sender.sizingY then
			if sender.sizingX then
				local newWidth=sender:GetWidth()+(args.X-sender.X)
				sender.X=args.X
				if newWidth<inventoryPanel.EntryNormalWidth+14 then newWidth=inventoryPanel.EntryNormalWidth+14 end
				if newWidth>displayWidth then newWidth=displayWidth end
				sender.dt.panel:SetWidth(newWidth-4)
				sender.dt.width=newWidth-4
				sender:SetWidth(newWidth)
			end
			if sender.sizingY then
				local newHeight=sender:GetHeight()+(args.Y-sender.Y)
				sender.Y=args.Y
				if newHeight<24 then newHeight=24 end
				if newHeight>displayHeight then newHeight=displayHeight end
				sender.dt.panel:SetHeight(newHeight-4)
				sender.dt.height=newHeight-4
			end
			if groupMaint:IsVisible() then
				local index=getDisplayTabIndexFromName(nil,dt.name)
				if index~=nil then
					updateGroupMaint(index)
				end
			end
			-- need to relayout the columns
			sender.dt.panel.ItemList:Layout()
		end
	end
	dt.window.SizeChanged=function()
		local newWidth=dt.window:GetWidth()
		local newHeight=dt.window:GetHeight()
	end
	return dt.window
end
createDisplayTabPanel=function(dt)
	-- dt is a displayTab entry
	local panel=Turbine.UI.Control()
	panel.tabHandle=dt -- makes referencing the displayTab attributes easier
	panel:SetBackColor(Turbine.UI.Color.Black)
	if dt.docked then
		panel:SetParent(inventoryPanel.ItemList)
		panel:SetSize(inventoryPanel.ItemList:GetWidth(),20)
	else
		if dt.window==nil then
			dt.window=createDTWindow(dt)
		end
		if dt.width==nil then dt.width=inventoryPanel.ItemList:GetWidth() end
		dt.window:SetWidth(dt.width+4)
		if dt.top==nil then dt.top=displayHeight/2 end
		if dt.top>displayHeight-20 then dt.top=displayHeight-20 end
		if dt.left==nil then dt.left=displayWidth/2 end
		if dt.left>displayWidth-40 then dt.left=displayWidth-40 end
		panel:SetParent(dt.window)
		panel:SetSize(dt.width,20) -- there are no items yet, so just account for header height
		dt.window:SetPosition(dt.left,dt.top)
	end
	panel.TitleBar=Turbine.UI.Label()
	panel.TitleBar:SetParent(panel)
	panel.TitleBar:SetSize(panel:GetWidth()-40,20)
	panel.TitleBar:SetMultiline(false)
	panel.TitleBar:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleCenter)
	panel.TitleBar:SetFont(Turbine.UI.Lotro.Font.Verdana20)
	panel.TitleBar:SetText(dt.name)
	panel.TitleBar.MouseDown=function(sender, args)
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		-- only care about mouse (move) when undocked
		if not dt.docked then
			panel.x=args.X
			panel.y=args.Y
			panel.moving=true
		end
	end
	panel.TitleBar.MouseMove=function(sender, args)
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		local settingsSource=inventoryWindow
		if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
			settingsSource=panel
		end
		if panel.moving and not dt.docked then
			local newX=dt.window:GetLeft()-(panel.x-args.X)
			local newY=dt.window:GetTop()-(panel.y-args.Y)
			if newX<0 then newX=0 end
			if newY<0 then newY=0 end
			if newX>displayWidth-40 then newX=displayWidth-40 end
			if newY>displayHeight-20-settingsSource.ItemEntryBackHeight-4 then newY=displayHeight-20-settingsSource.ItemEntryBackHeight-4 end
			dt.window:SetPosition(newX,newY)
			dt.left=newX
			dt.top=newY
			if groupMaint:IsVisible() then
				local index=getDisplayTabIndexFromName(nil,dt.name)
				if index~=nil then
					updateGroupMaint(index)
				end
			end
			-- check max height
			local numColumns=math.floor((panel.ItemList:GetWidth()-10)/settingsSource.ColumnWidth)

			local itemCount=panel.ItemList.VisibleCount
			if itemCount==nil then itemCount=panel.ItemList:GetItemCount() end

			local numRows=math.ceil(itemCount/numColumns)
			
			local newHeight=numRows*(settingsSource.ItemEntryBackHeight+4)
			local maxHeight=displayHeight-dt.window:GetTop()-20
			if dt.lockHeight then
				-- note, we calculated the list height but dt.height is PANEL height, not list height, have to account for header (20 pixels)
				if dt.height==nil then
					if newHeight<maxHeight then
						dt.height=newHeight+20
					else
						dt.height=maxHeight+20
					end
				else
					newHeight=dt.height-20
				end
			end
			if newHeight>maxHeight then
				--set new height so that vscroll will show if needed
				newHeight=maxHeight
			end
			panel.ItemList:SetSize(panel.ItemList:GetWidth(),newHeight)
		end
	end
	panel.TitleBar.MouseUp=function(sender, args)
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		panel.moving=false
	end

	panel.DisplayMode=Turbine.UI.Control()
	panel.DisplayMode:SetVisible(not dt.docked)
	panel.DisplayMode:SetParent(panel)
	panel.DisplayMode:SetSize(19,19)
	panel.DisplayMode:SetPosition(panel:GetWidth()-60,0)
	panel.DisplayMode:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	if dt.displayMode==nil then dt.displayMode=0 end
	if dt.displayMode==0 then
		panel.DisplayMode:SetBackground(resourcePath.."displayDefault_Down.jpg")
	elseif dt.displayMode==1 then
		panel.DisplayMode:SetBackground(resourcePath.."displayListOnly_Down.jpg")
	elseif dt.displayMode==2 then
		panel.DisplayMode:SetBackground(resourcePath.."displayIconList_Down.jpg")
	else
		panel.DisplayMode:SetBackground(resourcePath.."displayIconOnly_Down.jpg")
	end
	panel.DisplayMode.MouseClick=function(sender,args)
		-- should not be able to be clicked when docked, but...
		if not dt.docked then
			-- toggle to next display mode
			local panel=sender:GetParent()
			local dt=panel.tabHandle
			if dt.displayMode==nil then dt.displayMode=0 end
			dt.displayMode=dt.displayMode+1
			if dt.displayMode>3 then dt.displayMode=0 end
			if dt.displayMode==0 then
				panel.DisplayMode:SetBackground(resourcePath.."displayDefault_Down.jpg")
			elseif dt.displayMode==1 then
				panel.DisplayMode:SetBackground(resourcePath.."displayListOnly_Down.jpg")
			elseif dt.displayMode==2 then
				panel.DisplayMode:SetBackground(resourcePath.."displayIconList_Down.jpg")
			else
				panel.DisplayMode:SetBackground(resourcePath.."displayIconOnly_Down.jpg")
			end
			-- now refresh just this one panel
			panel:ApplyLayout()
		end
	end
	panel.GetLayout=function(sender)
		local panel=sender
		local dt=panel.tabHandle
		if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
			-- calc the panel specific layout values first
			local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
			panel.ItemEntryIconWidth=32*Settings.zoom
			local textHeight=fontSize*2
			local newHeight=textHeight
			if dt.displayMode==nil then dt.displayMode=0 end
			if dt.displayMode==3 then
				-- icon only
				panel.ItemEntryIconVisible=true
				panel.ItemEntryTextVisible=false
				panel.ItemEntryIconTop=0
				panel.ItemEntryBackWidth=panel.ItemEntryIconWidth
				panel.ItemEntryBackHeight=panel.ItemEntryIconWidth
			elseif dt.displayMode==1 then
				-- text only
				panel.ItemEntryIconVisible=false
				panel.ItemEntryTextVisible=true
				panel.ItemEntryTextLeft=fontMetric:GetTextWidth("888888") -- leave space for qty display
				panel.ItemEntryTextTop=0
				panel.ItemEntryTextHeight=textHeight
				fontMetric:SetFont(Settings.fontFace)
				panel.ItemEntryTextWidth=fontMetric:GetTextWidth("MWMWMWMWMWMWMWMWMWMW")
				panel.ItemEntryBackHeight=textHeight
				panel.ItemEntryBackWidth=panel.ItemEntryTextWidth+2+panel.ItemEntryTextLeft
			else
				panel.ItemEntryIconVisible=true
				panel.ItemEntryTextVisible=true
				if panel.ItemEntryIconWidth>newHeight then
					--icon determines height
					newHeight=panel.ItemEntryIconWidth
					panel.ItemEntryIconTop=0
					panel.ItemEntryTextTop=(newHeight-textHeight)/2
				else
					panel.ItemEntryIconTop=(newHeight-panel.ItemEntryIconWidth)/2
					panel.ItemEntryTextTop=0
				end
				panel.ItemEntryTextLeft=panel.ItemEntryIconWidth+1 -- 1 pixel between icon and text
				fontMetric:SetFont(Settings.fontFace)
				panel.ItemEntryTextWidth=fontMetric:GetTextWidth("MWMWMWMWMWMWMWMWMWMW")		
				panel.ItemEntryBackWidth=panel.ItemEntryTextLeft+panel.ItemEntryTextWidth
				panel.ItemEntryBackHeight=newHeight
			end
			panel.ColumnWidth=panel.ItemEntryBackWidth+4
		end
	end
	panel:GetLayout() -- set default display settings if undocked
	panel.ApplyLayout=function(sender)
		local panel=sender
		local dt=panel.tabHandle
		panel:GetLayout()
		if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
			for index=1,panel.ItemList:GetItemCount() do
				local item=panel.ItemList:GetItem(index)
				item:Layout(panel)
			end
		else
			for index=1,panel.ItemList:GetItemCount() do
				local item=panel.ItemList:GetItem(index)
				item:Layout()
			end
		end
	end
	--4114b104, 41152250 -- meh. created custom
	panel.DockButton=Turbine.UI.Control()
	panel.DockButton:SetParent(panel)
	panel.DockButton:SetSize(20,20)
	panel.DockButton:SetPosition(panel:GetWidth()-40,0)
	panel.DockButton:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	if dt.docked then
		panel.DockButton:SetBackground(resourcePath.."undocked.jpg")
	else
		panel.DockButton:SetBackground(resourcePath.."docked.jpg")
	end
	panel.DockButton.MouseClick=function(sender, args)
		-- swap the panel between docked and undocked mode
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		dt.docked=not dt.docked
		if dt.docked then
			-- determine index for where to insert
			local index=2
			local container=getCurrentContainer()

			for k,v in ipairs(displayTabs[container]) do
				if v==dt then
					break
				elseif v.docked then
					index=index+1
				end
			end
			panel:SetParent(inventoryPanel.ItemList)
			inventoryPanel.ItemList:InsertItem(index,panel)
			local tmpWidth=inventoryPanel.ItemList:GetWidth()
			if dt.expanded then
				panel:SetSize(tmpWidth, panel.ItemList:GetTop()+panel.ItemList:GetHeight())
			else
				panel:SetSize(tmpWidth,panel.TitleBar:GetHeight())
			end
			panel.ItemList:SetWidth(tmpWidth)
			panel.TitleBar:SetWidth(tmpWidth-60)
			panel.DisplayMode:SetLeft(tmpWidth-60)
			panel.DisplayMode:SetVisible(false)
			panel.DockButton:SetLeft(tmpWidth-40)
			panel.ExpandButton:SetLeft(tmpWidth-19)
			panel.DockButton:SetBackground(resourcePath.."undocked.jpg")
			dt.window:SetVisible(false)
			panel.ItemList:SetVerticalScrollBar(nil)
			panel.vScroll:SetPosition(dt.panel:GetWidth(),20) -- out of sight
			panel.vScroll:SetHeight(0)
			panel.vScroll:SetVisible(false)
		else
			inventoryPanel.ItemList:RemoveItem(panel)
			if dt.window==nil then
				dt.window=createDTWindow(dt)
			end
			if dt.left==nil then dt.left=displayWidth/2 end
			if dt.left>displayWidth-40 then dt.left=displayWidth-40 end
			if dt.top==nil then dt.top=displayHeight/2 end
			if dt.top>displayHeight-20 then dt.top=displayHeight-20 end
			dt.window:SetPosition(dt.left,dt.top)
			local tmpWidth=dt.width
			if tmpWidth==nil then
				tmpWidth=inventoryPanel.ItemList:GetWidth()
			end
			panel.ItemList:SetWidth(tmpWidth)
			if dt.lockHeight then
				-- list height is taken care of in layout
				panel:SetSize(tmpWidth,dt.height)
				panel.ItemList:Layout()
			else
				if dt.expanded then
					panel:SetSize(tmpWidth, panel.ItemList:GetTop()+panel.ItemList:GetHeight())
				else
					panel:SetSize(tmpWidth,panel.TitleBar:GetHeight())
				end
			end
			if dt.expanded then
				dt.window:SetSize(tmpWidth+4,panel:GetHeight()+4)
			else
				dt.window:SetSize(tmpWidth+4,20)
			end
			panel.TitleBar:SetWidth(tmpWidth-60)
			panel.DisplayMode:SetLeft(tmpWidth-60)
			panel.DisplayMode:SetVisible(true)
			panel.DockButton:SetLeft(tmpWidth-40)
			panel.ExpandButton:SetLeft(tmpWidth-19)
			panel:SetParent(dt.window)
			panel:SetTop(0) -- the listbox was previously controlling this
			panel.DockButton:SetBackground(resourcePath.."docked.jpg")
			panel.vScroll:SetPosition(panel:GetWidth()-10,20)
			panel.vScroll:SetHeight(panel.ItemList:GetHeight())
			panel.vScroll:SetVisible(true)
			panel.ItemList:SetVerticalScrollBar(dt.panel.vScroll)
			dt.window:SetVisible(true)
		end
		if groupMaint:IsVisible() then
			local index=getDisplayTabIndexFromName(nil,dt.name)
			if index~=nil then
				updateGroupMaint(index)
			end
		end
		inventoryPanel.Refresh()
		getItemEntryLayout()
		applyItemEntryLayout()
		inventoryPanel:Layout()
--		panel.ItemList:Layout()
		inventoryPanel.cropDelay=Settings.cropDelay
		inventoryPanel:SetWantsUpdates(true)
	end
	panel.DockButton.MouseHover=function()
		local hoverMenu=Turbine.UI.ContextMenu()
		local menuItems=hoverMenu:GetItems()
		menuItems:Clear()
		if dt.docked then
			menuItems:Add(Turbine.UI.MenuItem(Resource[language][128]))
		else
			menuItems:Add(Turbine.UI.MenuItem(Resource[language][127]))
		end
		hoverMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
	end

	panel.ExpandButton=Turbine.UI.Control()
	panel.ExpandButton:SetParent(panel)
	panel.ExpandButton:SetSize(19,19)
	panel.ExpandButton:SetPosition(panel:GetWidth()-19,0)
	panel.ExpandButton:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	if dt.expanded==nil then dt.expanded=true end
	if dt.expanded then
		panel.ExpandButton:SetBackground(0x41110b60)--(0x41007f87)
	else
		panel.ExpandButton:SetBackground(0x41110b5d)--(0x41007f88)
	end
	panel.ExpandButton.MouseClick=function(sender,args)
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		dt.expanded=not dt.expanded
		panel.ItemList:Layout() -- this will set up the right columns and list height
		if dt.expanded then
			panel:SetSize(panel:GetWidth(),panel.ItemList:GetHeight()+20)
			panel.ExpandButton:SetBackground(0x41110b60)--(0x41007f87)
		else
			panel.ExpandButton:SetBackground(0x41110b5d)--(0x41007f88)
			panel:SetSize(panel:GetWidth(),20)
		end
		if not dt.docked then
			local parent=panel:GetParent()
			if parent~=nil then
				if dt.expanded then
					if dt.lockHeight then
						-- list height is taken care of in layout
						panel:SetSize(panel.ItemList:GetWidth(),dt.height)
						panel.ItemList:Layout()
					else
						panel:GetParent():SetSize(dt.panel:GetWidth()+4,dt.panel:GetHeight())
					end
				else
					panel:GetParent():SetSize(dt.panel:GetWidth()+4,20)
				end
				panel:SetWantsUpdates(true)
			end
		else
			inventoryPanel.cropDelay=Settings.cropDelay
			inventoryPanel:SetWantsUpdates(true)
		end
		if groupMaint:IsVisible() then
			local index=getDisplayTabIndexFromName(nil,dt.name)
			if index~=nil then
				updateGroupMaint(index)
			end
		end
	end

	panel.ItemList=Turbine.UI.ListBox()
	panel.ItemList:SetParent(panel)
	panel.ItemList:SetMouseVisible(false)
	panel.ItemList:SetPosition(0,20)
	panel.ItemList:SetHeight(0) -- no height until it has items
	panel.ItemList:SetMaxColumns(1) -- this will get overridden when the width gets changed
	panel.ItemList:SetOrientation( Turbine.UI.Orientation.Horizontal)
	panel.ItemList:SetWidth(panel:GetWidth())

--*** double check that this is called when an item is added/removed and is sizing correctly
	panel.ItemList.Layout=function(sender,args)
if Settings.debug then
	Turbine.Shell.WriteLine("in panel.Layout. args:")
end
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		local settingsSource=inventoryWindow
		if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
			settingsSource=panel
		end
		-- need to determine number of columns
		local numColumns=math.floor((sender:GetWidth()-10)/settingsSource.ColumnWidth)
		if numColumns<1 then numColumns=1 end
		sender:SetMaxColumns(numColumns)

		local itemCount=panel.ItemList.VisibleCount
if Settings.debug then
	Turbine.Shell.WriteLine(".VisibleCount:"..tostring(itemCount))
	Turbine.Shell.WriteLine(".ItemList:GetItemCount():"..tostring(panel.ItemList:GetItemCount()))
end		if itemCount==nil then itemCount=panel.ItemList:GetItemCount() end

		local numRows=math.ceil(itemCount/numColumns)
		local newHeight=numRows*(settingsSource.ItemEntryBackHeight+4)
		if not dt.docked then
			panel.vScroll:SetPosition(panel:GetWidth()-10,20)
			local maxHeight=displayHeight-dt.top-20
			if dt.lockHeight then
				-- dt.height is panel height, have to adjust by title height
				if dt.height==nil then dt.height=maxHeight+20 end
				newHeight=dt.height-20
			else
				-- make sure new height is less than displayHeight-dt.top-20
				if newHeight>maxHeight then
					--set new height so that vscroll will show
					newHeight=maxHeight
				end
			end
		else
			panel.vScroll:SetPosition(panel:GetWidth(),20)
			panel.vScroll:SetHeight(0)
			panel.vScroll:SetVisible(false)
		end
		sender:SetHeight(newHeight)
		-- changing height is not considered changing size...
		sender:SizeChanged()
	end

	panel.ItemList.SizeChanged=function(sender,args)
		local panel=sender:GetParent()
		-- make sure we aren't deleting old panels (won't have a parent)
		if panel~=nil then
			local dt=panel.tabHandle
			if dt.expanded then
				panel:SetHeight(panel.ItemList:GetHeight()+20)
			else
				panel:SetHeight(20)
			end
			if not dt.docked then
				panel.vScroll:SetHeight(panel.ItemList:GetHeight())
				if dt.window~=nil then
					if dt.expanded then
						if dt.lockHeight then
							dt.window:SetHeight(panel:GetHeight()+4)
						else
							dt.window:SetHeight(panel:GetHeight())
						end
					else
						dt.window:SetHeight(20)
					end
				end
			end
		end
	end
--*** tring to get ApplySearch called asynchronously when items are moved/added/removed
	panel.ItemList.ItemAdded=function(sender,args)
		sender:Layout()
		inventoryPanel.SearchCaption:SetWantsUpdates(true)-- this will call ApplySearch on next screen refresh
	end
	panel.ItemList.ItemRemoved=function(sender,args)
		sender:Layout()
		inventoryPanel.SearchCaption:SetWantsUpdates(true)-- this will call ApplySearch on next screen refresh
	end
	panel.ItemList.SortList=function(sender,sortFunct)
		local panel=sender:GetParent()
		local dt=panel.tabHandle
		-- sender will be a panel.ItemList
		-- sortFunct is a function which sorts a table of items
		local tmpTable={};
		local index;
		local tmpHandle;

		for index=1,sender:GetItemCount() do
			sender:GetItem(index).oldIndex=index
			tmpTable[index]=sender:GetItem(index)
		end
		table.sort(tmpTable,sortFunct)
		local dummy=Turbine.UI.Control() -- not sure we still need to use the placeholder
		if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
			if dt.displayMode==3 then
				-- icon only
				dummy:SetSize(inventoryPanel.EntryNormalHeight,inventoryPanel.EntryNormalHeight);
			elseif dt.displayMode==1 then
				dummy:SetSize(inventoryPanel.EntryWideWidth,inventoryPanel.EntryNormalHeight);
			else
				-- icon and text
				dummy:SetSize(inventoryPanel.EntryNormalWidth,inventoryPanel.EntryNormalHeight);
			end
		else
			if Settings.panelViewMode==3 then
				-- icon only
				dummy:SetSize(inventoryPanel.EntryNormalHeight,inventoryPanel.EntryNormalHeight);
			elseif Settings.paneViewMode==1 then
				dummy:SetSize(inventoryPanel.EntryWideWidth,inventoryPanel.EntryNormalHeight);
			else
				-- icon and text
				dummy:SetSize(inventoryPanel.EntryNormalWidth,inventoryPanel.EntryNormalHeight);
			end
		end

		tmpHandle=1;
		index=1;
		-- using "for k,v in pairs" does not work - probably because we are manipulating the table as we iterate over it
		for index=1,#tmpTable do
			if sender:GetItem(index).Name~=tmpTable[index].Name then
				-- swap
				tmpHandle=sender:GetItem(index); -- grab a handle to the item that is in the index that we want the new tmpTable item to go
				tmpHandle.oldIndex=tmpTable[index].oldIndex;
				sender:SetItem(tmpTable[index].oldIndex,dummy) -- need to put a dummy at the old index so the table has the same number of elements when we move the item at tmpTable[index] to its new index
				sender:SetItem(index,tmpTable[index])
				sender:SetItem(tmpTable[index].oldIndex,tmpHandle)
				tmpHandle=nil;
			end
		end
	end
	panel.vScroll=Turbine.UI.Lotro.ScrollBar()
	panel.vScroll:SetParent(panel)
	panel.vScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
	panel.vScroll:SetWidth(10)
	if not dt.docked then
		panel.vScroll:SetPosition(panel:GetWidth()-10,20)
		panel.vScroll:SetHeight(panel:GetHeight())
		panel.ItemList:SetVerticalScrollBar(panel.vScroll)
	else
		panel.vScroll:SetPosition(panel:GetWidth(),20) -- move it out of sight
		panel.vScroll:SetHeight(0)
		panel.vScroll:SetVisible(false)
	end
	panel.PositionChanged=function(sender,args)
		-- this will fire whenever the panel is in inventoryPanel.ItemList and the list is scrolled
		local panel=sender
		local dt=panel.tabHandle
		if dt.docked and Settings.zoom>1 then
			-- use position changed to fire crop when docked
			inventoryPanel:SetWantsUpdates(true)
--			Turbine.Shell.WriteLine("panel position changed, top:"..tostring(sender:GetTop()))
		end
	end
	panel.SizeChanged=function(sender,args)
		local width=sender:GetWidth()
		sender.TitleBar:SetSize(width-60,20)
		sender.DisplayMode:SetPosition(width-60,0)
		sender.DockButton:SetPosition(width-40,0)
		sender.ExpandButton:SetPosition(width-19,0)
		sender.ItemList:SetWidth(width)
		sender.vScroll:SetLeft(width-10)
	end
	panel.Update=function(sender,args)
		cropPanelIcons(sender)
		sender:SetWantsUpdates(false)
	end
	dt.panel=panel
	return panel
end

--***** inventoryPanel *************************************************************************
-- this is the main panel that displays the various inventories and is moved between the full window and the minimal window as needed
inventoryPanel=Turbine.UI.Control();
-- not sure this SetSize is even used. it doesn't account for Settings.zoom
inventoryPanel:SetSize(Settings.panelWidth*displayWidth,Settings.panelHeight*displayHeight)
if Settings.useMinimalHeader then
	inventoryPanel:SetParent(minimalWindow)
	inventoryPanel:SetPosition(minimalWindow.LeftMargin,minimalWindow.TopMargin)
else
	inventoryPanel:SetParent(inventoryWindow)
	inventoryPanel:SetPosition(inventoryWindow.LeftMargin,inventoryWindow.TopMargin)
end
inventoryPanel:SetMouseVisible(false)
inventoryPanelBack=Turbine.UI.Control()
inventoryPanelBack:SetParent(inventoryPanel)
inventoryPanelBack:SetBackground("GaranStuff/AltInventory/Resources/background_mid.jpg")
inventoryPanelBack:SetBlendMode(Turbine.UI.BlendMode.Overlay)
inventoryPanelBack:SetSize(inventoryWindow:GetWidth(),80)
inventoryPanelBack:SetMouseVisible(false)
inventoryPanel.FilterCaption=Turbine.UI.Label()
inventoryPanel.FilterCaption:SetParent(inventoryPanel)
inventoryPanel.FilterCaption:SetPosition(9,6)
inventoryPanel.FilterCaption:SetSize(70,20)
inventoryPanel.FilterCaption:SetFont(fontFace)
inventoryPanel.FilterCaption:SetForeColor(Settings.fontColor)
inventoryPanel.FilterCaption:SetOutlineColor(Turbine.UI.Color(0,0,0))
inventoryPanel.FilterCaption:SetFontStyle(Turbine.UI.FontStyle.Outline)
inventoryPanel.FilterCaption:SetText(Resource[language][11]..":")
inventoryPanel.FilterList=GaranStuff.AltInventory.DropDownList()
inventoryPanel.FilterList:SetParent(inventoryPanel)
inventoryPanel.FilterList:SetPosition(inventoryPanel.FilterCaption:GetLeft()+inventoryPanel.FilterCaption:GetWidth()+3,inventoryPanel.FilterCaption:GetTop()-3)
inventoryPanel.FilterList:SetSize(inventoryPanel:GetWidth()-(inventoryPanel.FilterCaption:GetWidth()+inventoryPanel.FilterCaption:GetLeft())-40,20)
inventoryPanel.FilterList:SetBorderColor(Settings.trimColor)
inventoryPanel.FilterList:SetBackColor(Settings.backColor)
inventoryPanel.FilterList:SetTextColor(Settings.listTextColor)
inventoryPanel.FilterList:SetCurrentBackColor(Settings.backColor)
inventoryPanel.FilterList:SetDropRows(6)
inventoryPanel.FilterList:SetZOrder(2)
inventoryPanel.FilterList:SetMatchType(2)
inventoryPanel.FilterList.ValueMask:SetVisible(false) -- allow typing in the entry field

inventoryPanel.FilterList:AddItem(Resource[language][12],-1)
for k,v in ipairs(ItemCategory) do
	if v[1]==Turbine.Gameplay.ItemCategory.Undefined then
		undefinedCategoryIndex=k+1 -- need to keep track of where the "undefined" category winds up in the list
	end
end
inventoryPanel.FilterList.ListData:SetMaxColumns(1)

for k,v in ipairs(ItemCategory) do
	if v[1]==nil then v[1]=Turbine.Gameplay.ItemCategory.Undefined end
	categorySortOrder[v[1]]=k
	inventoryPanel.FilterList:AddItem(v[2][language],v[1])
end

inventoryPanel.FilterList.SelectedIndexChanged = function ()
	ApplySearch()
end

inventoryPanel.SearchCaption=Turbine.UI.Label();
inventoryPanel.SearchCaption:SetParent(inventoryPanel);
inventoryPanel.SearchCaption:SetPosition(9,31);
inventoryPanel.SearchCaption:SetSize(70,20);
inventoryPanel.SearchCaption:SetFont(fontFace);
inventoryPanel.SearchCaption:SetForeColor(Settings.fontColor);
inventoryPanel.SearchCaption:SetOutlineColor(Turbine.UI.Color(0,0,0));
inventoryPanel.SearchCaption:SetFontStyle(Turbine.UI.FontStyle.Outline);
inventoryPanel.SearchCaption:SetText(Resource[language][14]..":");
inventoryPanel.SearchCaption:SetWantsUpdates(false)
inventoryPanel.SearchCaption.Update=function()
-- this is SOOOO weird. The item gets added but is just a frame, no actual item. Second item adds and updates BOTH.
-- increasing the delay does not help. this would indicate that something odd is happening when adding the first item to the panel
-- note, when loading initially, panels that have one item are created correctly and display correctly.
	if inventoryPanel.SearchCaption.Delay==nil or inventoryPanel.SearchCaption.Delay==0 then
		inventoryPanel.SearchCaption.Delay=1
	else
		inventoryPanel.SearchCaption.Delay=inventoryPanel.SearchCaption.Delay-1
	end
	if inventoryPanel.SearchCaption.Delay==0 then
		ApplySearch()
		inventoryPanel.SearchCaption:SetWantsUpdates(false)
	end
end

inventoryPanel.SearchBorder=Turbine.UI.Control();
inventoryPanel.SearchBorder:SetParent(inventoryPanel);
inventoryPanel.SearchBorder:SetPosition(inventoryPanel.SearchCaption:GetLeft()+inventoryPanel.SearchCaption:GetWidth()+2,inventoryPanel.SearchCaption:GetTop()-5);
inventoryPanel.SearchBorder:SetSize(inventoryPanel:GetWidth()/2-(inventoryPanel.SearchCaption:GetWidth()+inventoryPanel.SearchCaption:GetLeft()),20);
inventoryPanel.SearchBorder:SetBackColor(Turbine.UI.Color(.15,.25,.45))

inventoryPanel.SearchText=Turbine.UI.Lotro.TextBox();
inventoryPanel.SearchText:SetParent(inventoryPanel);
inventoryPanel.SearchText:SetPosition(inventoryPanel.SearchCaption:GetLeft()+inventoryPanel.SearchCaption:GetWidth()+3,inventoryPanel.SearchCaption:GetTop()-4);
inventoryPanel.SearchText:SetSize(inventoryPanel.SearchBorder:GetWidth()-2,18);
inventoryPanel.SearchText:SetBackColor(Turbine.UI.Color(.25,.35,.55));
inventoryPanel.SearchText:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft);
inventoryPanel.SearchText:SetFont(Turbine.UI.Lotro.Font.TrajanPro16);
inventoryPanel.SearchText:SetForeColor(Settings.fontColor);
inventoryPanel.SearchText:SetOutlineColor(Turbine.UI.Color(0,0,0));
inventoryPanel.SearchText:SetFontStyle(Turbine.UI.FontStyle.Outline);
inventoryPanel.SearchText:SetWantsUpdates(false);
inventoryPanel.SearchText.Text="";
inventoryPanel.SearchText.NeedsUpdate=false
inventoryPanel.SearchText:SetAllowDrop(true)
inventoryPanel.SearchText.FocusGained=function()
	inventoryPanel.SearchText:SetWantsUpdates(true);
end
inventoryPanel.SearchText.FocusLost=function()
	inventoryPanel.SearchText:SetWantsUpdates(false);
end
inventoryPanel.SearchText.DragDrop=function(sender,args)
	local ddi=args.DragDropInfo
	if ddi~=nil then
		if ddi.GetShortcut~=nil then
			local sc=ddi:GetShortcut()
			local scType=sc:GetType()
			if scType==Turbine.UI.Lotro.ShortcutType.Item then
				local scItem=sc:GetItem()
				local scData=sc:GetData() -- will be string "0xnnnn,0xnn" where 0xnnnn is the ItemUID (or 0) and 0xnn is the ItemGID (or 0)
				if scItem ~=nil then
					local customName=tostring(scItem:GetName())
--					customName=string.gsub(customName,"-",".-")
					inventoryPanel.SearchText:SetText(customName)
					inventoryPanel.SearchText.Text=inventoryPanel.SearchText:GetText()
					ApplySearch()
				end
			end
		end
	end
end
inventoryPanel.SearchText.Update=function()
	if inventoryPanel.SearchText.Text~=inventoryPanel.SearchText:GetText() then
		inventoryPanel.SearchText.Text=inventoryPanel.SearchText:GetText()
		ApplySearch()
	end
end

function ApplySearch()
	if Settings.debug then
		Turbine.Shell.WriteLine("in ApplySearch")
	end
	inApplySearch=true
	local container=getCurrentContainer()	
	local category=inventoryPanel.FilterList:GetValue()
	local cmpText=string.lower(string.stripaccent(inventoryPanel.SearchText.Text))
	for k,v in ipairs(displayTabs[container]) do
		local tmpCount=0
		local panel=v.panel
		if panel~=nil then
			local dt=panel.tabHandle
			local settingsSource=inventoryWindow
			if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
				settingsSource=panel
			end
			local index
			local panelTop=panel:GetTop()
			local min=-32*Settings.zoom/2
			local max=inventoryPanel.ItemList:GetHeight()-32*Settings.zoom
			if not v.docked then
				max=panel.ItemList:GetHeight()-32*Settings.zoom
			end
			if max<0 then max=0 end
			for index=1,panel.ItemList:GetItemCount() do
				local tmpItem=panel.ItemList:GetItem(index)
				local tmpHeight=settingsSource.ItemEntryBackHeight+4
				local forceSort=1
				local category=inventoryPanel.FilterList:GetValue()
				if category~=-1 and tmpItem.SortCategory~=category then
						tmpHeight=0
						forceSort=2
				elseif inventoryPanel.SearchText.Text~="" then
					success,result=pcall(string.find,string.lower(string.stripaccent(tmpItem.Name)),cmpText,nil,true)
					if result==nil then
						tmpHeight=0
						forceSort=2
					end
				end
				if tmpHeight>0 then tmpCount=tmpCount+1 end
				tmpItem:SetHeight(tmpHeight)
				if tmpHeight==0 then
					tmpItem.isFiltered=true
					tmpItem.IconBack:SetHeight(0)
				else
					tmpItem.isFiltered=false
					-- need to make sure it isn't cropped
					local top=panelTop+tmpItem:GetTop()
					if Settings.zoom>1 and (top<min or top>max) then
						tmpItem.IconBack:SetHeight(0)
					else
						tmpItem.IconBack:SetSize(32*Settings.zoom,32*Settings.zoom)
					end
				end
				tmpItem.ForceSort=forceSort
			end
			-- need to track visible count so that we can refresh faster. apparently a ListBox scan is resource intensive so we take the hit up front
			panel.ItemList.VisibleCount=tmpCount
			panel.ItemList:Layout()
		end
	end
	inventoryPanel.SortList:SelectedIndexChanged(); -- reapply sort
	inApplySearch=false
	-- now crop
	if Settings.zoom>1 then
		inventoryPanel:SetWantsUpdates(true)
	end
end
inventoryPanel.ShowAllButton=Turbine.UI.Lotro.Button();
inventoryPanel.ShowAllButton:SetParent(inventoryPanel);
inventoryPanel.ShowAllButton:SetSize(120,18);
inventoryPanel.ShowAllButton:SetPosition(inventoryPanel.FilterList:GetLeft()+inventoryPanel.FilterList:GetWidth()-inventoryPanel.ShowAllButton:GetWidth(),inventoryPanel.SearchText:GetTop()-1);
inventoryPanel.ShowAllButton:SetText(Resource[language][23]);
inventoryPanel.ShowAllButton.Click=function()
	inventoryPanel.SearchText:SetWantsUpdates(false)
	inventoryPanel.SearchText:SetText("")
	inventoryPanel.SearchText.Text=""
	inventoryPanel.FilterList:SetSelectedIndex(1)
	inventoryPanel.FilterList:SelectedIndexChanged()
--	ApplySearch()
end

inventoryPanel.SortCaption=Turbine.UI.Label();
inventoryPanel.SortCaption:SetParent(inventoryPanel);
inventoryPanel.SortCaption:SetPosition(9,54);
inventoryPanel.SortCaption:SetSize(70,20);
inventoryPanel.SortCaption:SetFont(fontFace);
inventoryPanel.SortCaption:SetForeColor(Settings.fontColor);
inventoryPanel.SortCaption:SetOutlineColor(Turbine.UI.Color(0,0,0));
inventoryPanel.SortCaption:SetFontStyle(Turbine.UI.FontStyle.Outline);
inventoryPanel.SortCaption:SetText(Resource[language][15]..":");

inventoryPanel.SortList=DropDownList();
inventoryPanel.SortList:SetParent(inventoryPanel);
inventoryPanel.SortList:SetPosition(inventoryPanel.SortCaption:GetLeft()+inventoryPanel.SortCaption:GetWidth()+3,inventoryPanel.SortCaption:GetTop()-3);
inventoryPanel.SortList:SetSize(inventoryPanel:GetWidth()/2-(inventoryPanel.SortCaption:GetWidth()+inventoryPanel.SortCaption:GetLeft())-1,20);
inventoryPanel.SortList:SetBorderColor(Settings.trimColor);
inventoryPanel.SortList:SetCurrentBackColor(colorDarkGrey);
inventoryPanel.SortList:SetBackColor(Settings.backColor);
inventoryPanel.SortList:SetTextColor(Settings.listTextColor);
inventoryPanel.SortList:SetCurrentBackColor(Settings.backColor);
inventoryPanel.SortList:SetDropRows(5);  -- increase as we implement the last 2 options
inventoryPanel.SortList:SetZOrder(1);
--inventoryPanel.SortList:AddItem(Resource[language][16],0); -- location in bag is no longer supported since the vault and shared storage do not support this
inventoryPanel.SortList:AddItem(Resource[language][17],1);
inventoryPanel.SortList:AddItem(Resource[language][18],2);
inventoryPanel.SortList:AddItem(Resource[language][19],3);
inventoryPanel.SortList:AddItem(Resource[language][20],4);
--inventoryPanel.SortList:AddItem(Resource[language][21],5); -- did this really serve any use other than a mildly interesting programming exercise?  nope.
inventoryPanel.SortList.SelectedIndexChanged=function()
	inSort=true
	-- there is a major bug in the ListBox:Sort() method which totally fouls up the internal indexing for the listbox entries
	-- this requires the workaround below. if Turbine ever fixes the :Sort method then the workaround can be scrapped and :Sort reenabled
	local container=getCurrentContainer()
	local index=inventoryPanel.SortList:GetValue();

	if index==1 then
		for k,v in ipairs(displayTabs[container]) do
			if v.panel~=nil then

				v.panel.ItemList:SortList(function(arg1,arg2) if arg1.ForceSort<arg2.ForceSort then return true elseif arg1.ForceSort==arg2.ForceSort and arg1.Name<arg2.Name then return true end end)
			end
		end
	elseif index==2 then
		for k,v in ipairs(displayTabs[container]) do
			if v.panel~=nil then
				v.panel.ItemList:SortList(function(arg1,arg2) if arg1.ForceSort<arg2.ForceSort then return true elseif arg1.ForceSort==arg2.ForceSort and arg1.Name>arg2.Name then return true end end)
			end
		end
	elseif index==3 then
		for k,v in ipairs(displayTabs[container]) do
			if v.panel~=nil then
				for tmpIndex=1,v.panel.ItemList:GetItemCount() do
					if categorySortOrder[v.panel.ItemList:GetItem(tmpIndex).Category]==nil then
						-- set it to the same sort order as the Undefined category
						categorySortOrder[v.panel.ItemList:GetItem(tmpIndex).Category]=categorySortOrder[Turbine.Gameplay.ItemCategory.Undefined]
					end
				end
				v.panel.ItemList:SortList(function(arg1,arg2) if arg1.ForceSort<arg2.ForceSort then return true elseif arg1.ForceSort==arg2.ForceSort and categorySortOrder[arg1.Category]<categorySortOrder[arg2.Category] then return true elseif arg1.ForceSort==arg2.ForceSort and categorySortOrder[arg1.Category]==categorySortOrder[arg2.Category] and arg1.Name< arg2.Name then return true end end)
			end
		end
	else
--*--		inventoryPanel.ItemList:Sort(function(arg1,arg2) if arg1.Quality<arg2.Quality then return true end end)
--		inventoryPanel.ItemList:SortList(function(arg1,arg2) if arg1.ForceSort<arg2.ForceSort then return true elseif arg1.ForceSort==arg2.ForceSort and arg1.Quality<arg2.Quality then return true elseif arg1.ForceSort==arg2.ForceSort and arg1.Quality==arg2.Quality and arg1.Name< arg2.Name then return true end end)
		for k,v in ipairs(displayTabs[container]) do
			if v.panel~=nil then
				v.panel.ItemList:SortList(function(arg1,arg2) if arg1.ForceSort<arg2.ForceSort then return true elseif arg1.ForceSort==arg2.ForceSort and arg1.Quality<arg2.Quality then return true elseif arg1.ForceSort==arg2.ForceSort and arg1.Quality==arg2.Quality and arg1.Name< arg2.Name then return true end end)
			end
		end
	end
	inSort=false
	if Settings.zoom>1 then
		inventoryPanel.cropDelay=Settings.cropDelay
		inventoryPanel:SetWantsUpdates(true)
	end
end

inventoryPanel.DisplayCaption=Turbine.UI.Label();
inventoryPanel.DisplayCaption:SetParent(inventoryPanel);
inventoryPanel.DisplayCaption:SetPosition(inventoryPanel:GetWidth()/2+10,54);
inventoryPanel.DisplayCaption:SetSize(70,20);
inventoryPanel.DisplayCaption:SetFont(fontFace);
inventoryPanel.DisplayCaption:SetForeColor(Settings.fontColor);
inventoryPanel.DisplayCaption:SetOutlineColor(Turbine.UI.Color(0,0,0));
inventoryPanel.DisplayCaption:SetFontStyle(Turbine.UI.FontStyle.Outline);
inventoryPanel.DisplayCaption:SetText(Resource[language][22]..":");

inventoryPanel.EntryNormalWidth=185;
inventoryPanel.EntryWideWidth=370;
inventoryPanel.EntryNormalHeight=38;
inventoryPanel.DisplayMode1=Turbine.UI.Control();
inventoryPanel.DisplayMode1:SetParent(inventoryPanel);
inventoryPanel.DisplayMode1:SetPosition(inventoryPanel.DisplayCaption:GetLeft()+inventoryPanel.DisplayCaption:GetWidth()+10,52);
inventoryPanel.DisplayMode1:SetSize(19,19);
inventoryPanel.DisplayMode1.MouseClick=function()
	inventoryPanel.DisplayMode1:SetBackground("GaranStuff/AltInventory/Resources/displayListOnly_Down.jpg");
	inventoryPanel.DisplayMode2:SetBackground("GaranStuff/AltInventory/Resources/displayIconList_Up.jpg");
	inventoryPanel.DisplayMode3:SetBackground("GaranStuff/AltInventory/Resources/displayIconOnly_Up.jpg");
	Settings.panelViewMode=1;
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout();
end

inventoryPanel.DisplayMode2=Turbine.UI.Control();
inventoryPanel.DisplayMode2:SetParent(inventoryPanel);
inventoryPanel.DisplayMode2:SetPosition(inventoryPanel.DisplayMode1:GetLeft()+inventoryPanel.DisplayMode1:GetWidth()+10,52);
inventoryPanel.DisplayMode2:SetSize(19,19);
inventoryPanel.DisplayMode2.MouseClick=function()
	inventoryPanel.DisplayMode1:SetBackground("GaranStuff/AltInventory/Resources/displayListOnly_Up.jpg");
	inventoryPanel.DisplayMode2:SetBackground("GaranStuff/AltInventory/Resources/displayIconList_Down.jpg");
	inventoryPanel.DisplayMode3:SetBackground("GaranStuff/AltInventory/Resources/displayIconOnly_Up.jpg");
	Settings.panelViewMode=2;
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout();
end

inventoryPanel.DisplayMode3=Turbine.UI.Control();
inventoryPanel.DisplayMode3:SetParent(inventoryPanel);
inventoryPanel.DisplayMode3:SetPosition(inventoryPanel.DisplayMode2:GetLeft()+inventoryPanel.DisplayMode2:GetWidth()+10,52);
inventoryPanel.DisplayMode3:SetSize(19,19);
inventoryPanel.DisplayMode3.MouseClick=function()
	inventoryPanel.DisplayMode1:SetBackground("GaranStuff/AltInventory/Resources/displayListOnly_Up.jpg");
	inventoryPanel.DisplayMode2:SetBackground("GaranStuff/AltInventory/Resources/displayIconList_Up.jpg");
	inventoryPanel.DisplayMode3:SetBackground("GaranStuff/AltInventory/Resources/displayIconOnly_Down.jpg");
	Settings.panelViewMode=3;
	getItemEntryLayout()
	applyItemEntryLayout()
	inventoryPanel:Layout();
end

-- .ItemList is the listbox that holds the displayTabs .panels when they are docked
inventoryPanel.ItemList=Turbine.UI.ListBox()
inventoryPanel.ItemList:SetMouseVisible(false);
inventoryPanel.ItemList:SetParent(inventoryPanel)
inventoryPanel.ItemList:SetPosition(0,80)
inventoryPanel.ItemList:SetMaxColumns(1)
-- this is the workaround for the built in ListBox:Sort() method that is buggy
inventoryPanel.VScroll=Turbine.UI.Lotro.ScrollBar()
inventoryPanel.VScroll:SetParent(inventoryPanel);
inventoryPanel.VScroll:SetSize(10,inventoryPanel.ItemList:GetHeight());
inventoryPanel.VScroll:SetPosition(inventoryPanel:GetWidth()-10,inventoryPanel.ItemList:GetTop());
inventoryPanel.VScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
inventoryPanel.ItemList:SetVerticalScrollBar(inventoryPanel.VScroll)
inventoryPanel.ItemList:SetOrientation( Turbine.UI.Orientation.Vertical )
inventoryPanel.ItemList:SetAllowDrop(true);
inventoryPanel.ItemList.DragDrop=function(sender,args)
	if inventoryWindow.CharList:GetText()==charName then
		local tmpIndex;
		local dropIndex;
		for tmpIndex=1,backPack:GetSize() do
			if backPack:GetItem(tmpIndex)==nil then
				dropIndex=tmpIndex;
				break
			end
		end
		if dropIndex==nil then
			inventoryWindow.Message:SetText(Resource[language][50])
		else
			backPack:PerformShortcutDrop(args.DragDropInfo:GetShortcut(),dropIndex);
		end
	else
		inventoryWindow.Message:SetText(Resource[language][51])
	end
end

if Settings.panelViewMode==3 then
	inventoryPanel.DisplayMode3:SetBackground("GaranStuff/AltInventory/Resources/displayIconOnly_Down.jpg");
	inventoryPanel.DisplayMode2:SetBackground("GaranStuff/AltInventory/Resources/displayIconList_Up.jpg");
	inventoryPanel.DisplayMode1:SetBackground("GaranStuff/AltInventory/Resources/displayListOnly_Up.jpg");
elseif Settings.panelViewMode==2 then
	inventoryPanel.DisplayMode2:SetBackground("GaranStuff/AltInventory/Resources/displayIconList_Down.jpg");
	inventoryPanel.DisplayMode1:SetBackground("GaranStuff/AltInventory/Resources/displayListOnly_Up.jpg");
	inventoryPanel.DisplayMode3:SetBackground("GaranStuff/AltInventory/Resources/displayIconOnly_Up.jpg");
else
	inventoryPanel.DisplayMode1:SetBackground("GaranStuff/AltInventory/Resources/displayListOnly_Down.jpg");
	inventoryPanel.DisplayMode2:SetBackground("GaranStuff/AltInventory/Resources/displayIconList_Up.jpg");
	inventoryPanel.DisplayMode3:SetBackground("GaranStuff/AltInventory/Resources/displayIconOnly_Up.jpg");
end

inventoryPanel.Layout=function()
	if debug then
		Turbine.Shell.WriteLine("in inventoryPanel.Layout")
	end
	local parent=inventoryPanel:GetParent()
	if true then
		local width,height
		if parent.LeftMargin~=nil then
			width=parent:GetWidth()-parent.LeftMargin*2
			height=parent:GetHeight()-parent.TopMargin-parent.BottomMargin
		else
			width=parent:GetWidth()
			height=parent:GetHeight()
		end
		inventoryPanel:SetSize(width,height);
		inventoryPanelBack:SetWidth(width);
		local numColumns
		if Settings.panelViewMode==3 then
			inventoryPanel.ItemList.ColWidth=inventoryPanel.EntryNormalHeight
		elseif Settings.panelViewMode==1 then
			inventoryPanel.ItemList.ColWidth=inventoryPanel.EntryWideWidth
		else
			inventoryPanel.ItemList.ColWidth=inventoryPanel.EntryNormalWidth
		end
		numColumns=math.floor((inventoryPanel:GetWidth()-10)/inventoryPanel.ItemList.ColWidth)

		inventoryPanel.FilterList:SetWidth(width-inventoryPanel.FilterList:GetLeft()-37);
		inventoryPanel.SearchBorder:SetWidth(width/2-inventoryPanel.SearchBorder:GetLeft());
		inventoryPanel.SearchText:SetWidth(inventoryPanel.SearchBorder:GetWidth()-2);
		inventoryPanel.ShowAllButton:SetLeft(width*3/4-inventoryPanel.ShowAllButton:GetWidth()/2);
		inventoryPanel.SortList:SetWidth(width/2-inventoryPanel.SortList:GetLeft());
		inventoryPanel.DisplayCaption:SetLeft(width*3/4-(inventoryPanel.DisplayCaption:GetWidth()+30+inventoryPanel.DisplayMode1:GetWidth()*3)/2);
		inventoryPanel.DisplayMode1:SetLeft(inventoryPanel.DisplayCaption:GetLeft()+inventoryPanel.DisplayCaption:GetWidth()+10);
		inventoryPanel.DisplayMode2:SetLeft(inventoryPanel.DisplayMode1:GetLeft()+inventoryPanel.DisplayMode1:GetWidth()+10);
		inventoryPanel.DisplayMode3:SetLeft(inventoryPanel.DisplayMode2:GetLeft()+inventoryPanel.DisplayMode2:GetWidth()+10);
		inventoryPanel.ItemList:SetSize(inventoryPanel:GetWidth()-10,inventoryPanel:GetHeight()-inventoryPanelBack:GetHeight())
		inventoryPanel.VScroll:SetLeft(inventoryPanel:GetWidth()-10);
		inventoryPanel.VScroll:SetHeight(inventoryPanel.ItemList:GetHeight())
		local container=getCurrentContainer()
		for k,v in ipairs(displayTabs[container]) do
			if v.panel~=nil then
				if v.docked then
					-- set width by inventoryPanel
					v.panel:SetWidth(inventoryPanel.ItemList:GetWidth())
				end
				v.panel.ItemList:Layout()
			end
		end
	end
end

inventoryPanel.RemoveConfirmation=Turbine.UI.Label();
inventoryPanel.RemoveConfirmation:SetParent(inventoryPanel);
inventoryPanel.RemoveConfirmation:SetSize(200,200);
inventoryPanel.RemoveConfirmation:SetPosition((inventoryWindow:GetWidth()-inventoryPanel.RemoveConfirmation:GetWidth())/2,(inventoryWindow:GetHeight()-inventoryPanel.RemoveConfirmation:GetHeight())/2);
inventoryPanel.RemoveConfirmation:SetBackColor(trimColor);
inventoryPanel.RemoveConfirmation:SetZOrder(2);
inventoryPanel.RemoveConfirmation.Text=Turbine.UI.Label();
inventoryPanel.RemoveConfirmation.Text:SetParent(inventoryPanel.RemoveConfirmation)
inventoryPanel.RemoveConfirmation.Text:SetSize(inventoryPanel.RemoveConfirmation:GetWidth()-2,inventoryPanel.RemoveConfirmation:GetHeight()-2);
inventoryPanel.RemoveConfirmation.Text:SetPosition(1,1);
inventoryPanel.RemoveConfirmation.Text:SetBackColor(backColor);
inventoryPanel.RemoveConfirmation.Text:SetForeColor(fontColor);
inventoryPanel.RemoveConfirmation.Text:SetFont(Turbine.UI.Lotro.Font.TrajanPro18);
inventoryPanel.RemoveConfirmation.Text:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleCenter);
inventoryPanel.RemoveConfirmation.Text:SetText(Resource[1][46]);
inventoryPanel.RemoveConfirmation.OK=Turbine.UI.Lotro.Button();
inventoryPanel.RemoveConfirmation.OK:SetParent(inventoryPanel.RemoveConfirmation.Text);
inventoryPanel.RemoveConfirmation.OK:SetSize(80,20);
inventoryPanel.RemoveConfirmation.OK:SetPosition(10,inventoryPanel.RemoveConfirmation.Text:GetHeight()-25);
inventoryPanel.RemoveConfirmation.OK:SetText(Resource[1][47]);
inventoryPanel.RemoveConfirmation.OK.Click=function()
	local name=inventoryWindow.CharList:GetText();
	local vaultName=name.." (Vault)";
	local eiName=name.." (EI)"
	for k,v in pairs(accountItemQty) do
		local delElem=v.Qty[name];
		if delElem~=nil then
			if type(delElem)=="table" then
				if delElem.Subtotal~=nil then
					accountItemQty[k].Total=accountItemQty[k].Total-delElem.Subtotal;
				end
			else
				if delElem~=nil then
					accountItemQty[k].Total=accountItemQty[k].Total-delElem;
				end
			end
			accountItemQty[k].Qty[name]=nil;
		end
		delElem=v.Qty[vaultName];
		if delElem~=nil then
			if type(delElem)=="table" then
				if delElem.Subtotal~=nil then
					accountItemQty[k].Total=accountItemQty[k].Total-delElem.Subtotal;
				end
			else
				if delElem~=nil then
					accountItemQty[k].Total=accountItemQty[k].Total-delElem;
				end
			end
			accountItemQty[k].Qty[vaultName]=nil;
		end
	end
	inventoryWindow:SetEnabled(true);
	inventoryPanel.RemoveConfirmation:SetVisible(false);
	local index=inventoryWindow.CharList:GetSelectedIndex();
	inventoryWindow.CharList:RemoveItemAt(index);
	minimalWindow.CharMenu:GetItems():RemoveAt(index);
	CharList[name]=nil;
	local delIndex;
	for delIndex=3,inventoryWindow.CharList.ListData:GetItemCount() do
		if inventoryWindow.CharList.ListData:GetItem(delIndex):GetText()==vaultName then
			inventoryWindow.CharList:RemoveItemAt(delIndex);
			minimalWindow.CharMenu:GetItems():RemoveAt(delIndex);
			if delIndex>index then index=delIndex end -- shouldn't be possible
			break;
		end
	end
	CharList[vaultName]=nil
	for delIndex=3,inventoryWindow.CharList.ListData:GetItemCount() do
		if inventoryWindow.CharList.ListData:GetItem(delIndex):GetText()==eiName then
			inventoryWindow.CharList:RemoveItemAt(delIndex);
			minimalWindow.CharMenu:GetItems():RemoveAt(delIndex);
			if delIndex>index then index=delIndex end -- shouldn't be possible
			break;
		end
	end
	CharList[eiName]=nil
	--update the currentCharBagsIndex
	for k,v in ipairs(CharList) do
		if k==name then
			inventoryWindow.currentCharBagsIndex=k
			break
		end
	end

	if index>inventoryWindow.CharList.ListData:GetItemCount() then index=inventoryWindow.CharList.ListData:GetItemCount() end
	inventoryWindow.CharList:SetSelectedIndex(index)
	updateDisplayTabXref(true)
	inventoryWindow.CharList:SelectedIndexChanged()
end
inventoryPanel.RemoveConfirmation.Cancel=Turbine.UI.Lotro.Button();
inventoryPanel.RemoveConfirmation.Cancel:SetParent(inventoryPanel.RemoveConfirmation.Text);
inventoryPanel.RemoveConfirmation.Cancel:SetSize(80,20);
inventoryPanel.RemoveConfirmation.Cancel:SetPosition(inventoryPanel.RemoveConfirmation.Text:GetWidth()-inventoryPanel.RemoveConfirmation.Cancel:GetWidth()-10,inventoryPanel.RemoveConfirmation.Text:GetHeight()-25);
inventoryPanel.RemoveConfirmation.Cancel:SetText(Resource[1][37]);
inventoryPanel.RemoveConfirmation.Cancel.Click=function()
	inventoryWindow:SetEnabled(true);
	inventoryPanel.RemoveConfirmation:SetVisible(false);
	end
inventoryPanel.RemoveConfirmation:SetVisible(false);

inventoryPanel:Layout()

getCurrentItemPanel=function(name,category)
	local container=getCurrentContainer()
	if displayTabXref[name]==nil then
		local newXref=recalcItemDisplayTabXref(name,category)
		if newXref==nil then
			-- we have a problem, no xref for this item name found. put it in the default panel
			displayTabXref[name]=defaultDisplayTabIndex
		end
	end
	if debug then
		Turbine.Shell.WriteLine("container:"..tostring(container).." name:"..tostring(name))
	end
	return displayTabs[container][displayTabXref[name]].panel
end
ItemEntry=class(Turbine.UI.Control)
function ItemEntry:Constructor(name,char,qty,itemInfo,category,infoName)
	Turbine.UI.Control.Constructor(self)
	if char~=charName and itemInfo==nil then
		-- try to resolve itemInfo from item name - this will fail on items with custom names like crit crafted items and LIs
		if infoName==nil then infoName=name end
		local tmpItemID=getItemID(infoName,category) -- need category for quicker lookup
		if tmpItemID~=nil then
			local tmpSC=Turbine.UI.Lotro.Shortcut(Turbine.UI.Lotro.ShortcutType.Item,string.format("0,0x%x",tmpItemID))
			if tmpSC~=nil then
				local tmpItem=tmpSC:GetItem()
				if tmpItem~=nil then
					itemInfo=tmpItem:GetItemInfo()
				end
			end
		end
	end
	if itemInfo~=nil then
		self.panel=getCurrentItemPanel(name,itemInfo:GetCategory())
	else
		self.panel=getCurrentItemPanel(name,nil)
	end
	local dt
	local settingsSource=inventoryWindow
	if self.panel~=nil then
		dt=self.panel.tabHandle
		if dt~=nil then
			if not dt.docked and dt.displayMode~=nil and dt.displayMode>0 then
				settingsSource=self.panel
			end
		end
	end

	self:SetParent(self.panel)
	self.FilteredHeight=inventoryPanel.EntryNormalHeight;
	self:SetBackColor(Turbine.UI.Color(.1,.15,.25));
	self.Back=Turbine.UI.Control();
	self.Back:SetParent(self);
	self.Back:SetPosition(2,2);
	self.Back:SetBackColor(Settings.backColor);
	self.Back:SetMouseVisible(false);
	self.Text=Turbine.UI.Label();
	self.Text:SetParent(self.Back);
	self.Text:SetForeColor(GetQualityColor(accountItemQty[name].Quality));
	self.Text:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft);
	self.Text:SetFont(Turbine.UI.Lotro.Font.Verdana16);
	self.Text:SetText(name);
	self.Text:SetMouseVisible(false);
	self.IconBack=Turbine.UI.Control();
	self.IconBack:SetParent(self.Back);
	self.IconBack:SetSize(32,32);
	self.IconBack:SetMouseVisible(false);

	self.ItemInfo=Turbine.UI.Lotro.ItemInfoControl();
	self.ItemInfo:SetParent(self.IconBack);
	self.ItemInfo:SetSize(32,32);
	self.ItemInfo:SetPosition(-3,-3);
	self.ItemInfo:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	self.ItemInfo:SetMouseVisible(true);
	if itemInfo==nil then
		self.ItemInfo:SetMouseVisible(false);
	else
		self.ItemInfo:SetMouseVisible(true);
		self.ItemInfo:SetItemInfo(itemInfo);
	end
	self.Icon=Turbine.UI.Control();
	self.Icon:SetParent(self.IconBack);
	self.Icon:SetSize(32,32);
	self.Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	self.Icon:SetBackground(accountItemQty[name].IconImageID);
	self.Icon:SetMouseVisible(false);
	self.Icon:SetZOrder(1);
	self.IconBorder=Turbine.UI.Control();
	self.IconBorder:SetParent(self.Icon);
	self.IconBorder:SetSize(32,32);
	self.IconBorder:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	self.IconBorder:SetBackground("GaranStuff/AltInventory/Resources/IconBorder.tga");
	self.IconBorder:SetMouseVisible(false);
	self.IconBorder:SetVisible((itemInfo==nil));
	self.Qty=Turbine.UI.Label();
	self.Qty:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleRight);
	self.Qty:SetForeColor(Turbine.UI.Color.White)
	self.Qty:SetOutlineColor(Turbine.UI.Color(0,0,0));
	self.Qty:SetFontStyle(Turbine.UI.FontStyle.Outline);
	self.Qty:SetMouseVisible(false);
	self.Qty:SetText(qty)
	self.Name=name;
	self.Category=accountItemQty[name].Category;
	self.SortCategory=accountItemQty[name].SortCategory;
	if self.SortCategory==nil then self.SortCategory=self.Category end;
	self.Quality=accountItemQty[name].Quality;
	local filterHeight=inventoryPanel.EntryNormalHeight
	local cmpText=string.lower(string.stripaccent(inventoryPanel.SearchText.Text));
	self.ForceSort=1;
	local category=inventoryPanel.FilterList:GetValue()
	if category~=-1 and self.Category~=category then
		filterHeight=0;
		self.ForceSort=2;
	elseif inventoryPanel.SearchText.Text~="" then
		if string.find(string.lower(string.stripaccent(self.Name)),cmpText)==nil then
			filterHeight=0;
			self.ForceSort=2;
		end
	end
--	if char==charName or char=="Shared Storage" or char==charName.." (Vault)" and not Settings.totalsOnly then
--Turbine.Shell.WriteLine("char:"..tostring(char).." char==charName:"..tostring(char==charName).." Settings.totalsOnly:"..tostring(Settings.totalsOnly))
	if char==charName and not Settings.totalsOnly then
		self.Item=Turbine.UI.Lotro.ItemControl()
		self.Item:SetParent(self.IconBack);
		self.Item:SetSize(36,36);
		self.Item:SetPosition(-3,-3);
		self.ItemInfo:SetVisible(false);
		self.Icon:SetVisible(false);
		self.Item:SetAllowDrop(true);
		self.Item.DragDrop=function(sender,args)
			if Settings.debug then
				Table.Dump(args)
			end
			backPack:PerformShortcutDrop(args.DragDropInfo:GetShortcut(),self.Item.Index);
		end
	else
		self.IconBack:SetBackground(accountItemQty[name].BackgroundImageID);
	end

	self.Back:SetSize(settingsSource.ItemEntryBackWidth,settingsSource.ItemEntryBackHeight)
	self:SetSize(settingsSource.ItemEntryBackWidth+4,settingsSource.ItemEntryBackHeight+4) -- items have a 2 pixel border
	if settingsSource.ItemEntryTextVisible then
		local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
		self.Text:SetFont(Settings.fontFace)
		self.Text:SetText(self.Text:GetText())
		self.Text:SetSize(settingsSource.ItemEntryTextWidth,fontSize*2)
		self.Text:SetPosition(settingsSource.ItemEntryTextLeft,settingsSource.ItemEntryTextTop)
		self.Text:SetVisible(true)
	else
		self.Text:SetVisible(false)
	end

	self.IconBack:SetTop(settingsSource.ItemEntryIconTop)
	self.IconBack:SetStretchMode(0)
	self.IconBack:SetSize(32,32)
	if Settings.zoom>1 then
		self.IconBack:SetStretchMode(1)
		self.IconBack:SetSize(settingsSource.ItemEntryIconWidth,settingsSource.ItemEntryIconWidth)
	end
	self.IconBack:SetVisible(settingsSource.ItemEntryIconVisible)

	if char~=charName or Settings.totalsOnly then
		self.MouseHover=function()
			-- remove prior items
			local detailList=qtyDetail:GetItems();
			detailList:Clear();
			local tmpList={};
--*** this is where we create the popup list of quantities
			if inventoryWindow.CharList:GetSelectedIndex()==1 then
				for k,v in pairs(accountItemQty[self.Name].Qty) do
					if k~="Shared Storage" then
						table.insert(tmpList,tostring(k)..": "..tostring(v.Subtotal));
					end
				end
				table.sort(tmpList)
				if accountItemQty[self.Name].Qty["Shared Storage"]~=nil then
					table.insert(tmpList,1,tostring("Shared Storage")..": "..tostring(accountItemQty[self.Name].Qty["Shared Storage"].Subtotal));
				end
			elseif inventoryWindow.CharList:GetSelectedIndex()==2 then
				for k,v in pairs(accountItemQty[self.Name].Qty["Shared Storage"]) do
					if k~="Subtotal" then
						table.insert(tmpList,tostring(sharedStorage:GetChestName(k))..": "..tostring(v))
					end
				end
				table.sort(tmpList)
			elseif inventoryWindow.CharList:GetText()==charName then
				-- calculate and display bag totals
				local bagQty={}
				for k,v in pairs(accountItemQty[self.Name].Qty[charName]) do
					if k~="Subtotal" then
--*** this calculation is flawed now that bags are variable size
--*** there is no bag property for backpack items like the chest property for vault items :(
						local bag=math.floor((k+14)/15)
						if bagQty[bag]==nil then
							bagQty[bag]=v;
						else
							bagQty[bag]=bagQty[bag]+v;
						end
					end
				end
				for k,v in pairs(bagQty) do
					table.insert(tmpList,Resource[language][49]..tostring(k)..": "..tostring(v));
				end
			else
				local char=string.match(inventoryWindow.CharList:GetText(),"(.*) %(Vault%)")
				local isVault=true;
				if char==nil then
					char=inventoryWindow.CharList:GetText();
					isVault=false;
				end
				
				for k,v in pairs(accountItemQty[self.Name].Qty[inventoryWindow.CharList:GetText()]) do
					if k~="Subtotal" then
						if isVault then
							if char==charName then
								table.insert(tmpList,tostring(vault:GetChestName(k))..": "..tostring(v));
							else
								table.insert(tmpList,tostring(CharList[char].VaultChestNames[k+1])..": "..tostring(v));
							end
						else
							table.insert(tmpList,Resource[language][49]..tostring(k)..": "..tostring(v));
						end
					end
				end
				table.sort(tmpList)
			end
			for k,v in ipairs(tmpList) do
				detailList:Add(Turbine.UI.MenuItem(v));
			end
			qtyDetail:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20);
		end
		self.MouseLeave=function()
			qtyDetail:Close();
		end
	end
	self.PositionChanged=function(sender,args)
		if Settings.zoom>1 and not inApplySearch and not inSort then
			-- this will fire whenever the panel is in inventoryPanel.ItemList and the list is scrolled
			local itemList=sender:GetParent()
			local panel=itemList:GetParent()
			local dt=panel.tabHandle
			if not dt.docked and dt.window~=nil then
				if panel:GetParent()~=nil then
					-- use position changed to fire crop when docked
					panel:SetWantsUpdates(true)
				end
			end
		end
	end
	
end
function ItemEntry:Layout(settingsSource)
	if settingsSource==nil then settingsSource=inventoryWindow end
	-- apply the layout to an instance of ItemEntry
	-- apply the settings for ItemEntryIconVisible, ItemEntryTextVisible, ItemEntryIconWidth, ItemEntryIconTop, ItemEntryTextWidth, ItemEntryTextLEft, ItemEntryTextTop, ItemEntryBackWidth, ItemEntryBackHeight
	self.Back:SetSize(settingsSource.ItemEntryBackWidth,settingsSource.ItemEntryBackHeight)
	self:SetSize(settingsSource.ItemEntryBackWidth+4,settingsSource.ItemEntryBackHeight+4) -- items have a 2 pixel border
	if settingsSource.ItemEntryTextVisible then
		local fontSize=Turbine.UI.Lotro.FontInfo[Settings.fontFace].size
		self.Text:SetFont(Settings.fontFace)
		self.Text:SetText(self.Text:GetText())
		self.Text:SetSize(settingsSource.ItemEntryTextWidth,fontSize*2)
		self.Text:SetPosition(settingsSource.ItemEntryTextLeft,settingsSource.ItemEntryTextTop)
		self.Text:SetVisible(true)
	else
		self.Text:SetVisible(false)
	end

	self.IconBack:SetTop(settingsSource.ItemEntryIconTop)
	self.IconBack:SetSize(settingsSource.ItemEntryIconWidth,settingsSource.ItemEntryIconWidth)
	self.IconBack:SetStretchMode(0)
	self.IconBack:SetSize(32,32)
	if Settings.zoom>1 then
		self.IconBack:SetStretchMode(1)
		self.IconBack:SetSize(32*Settings.zoom,32*Settings.zoom)
	end
	if settingsSource.ItemEntryIconVisible then
		self.IconBack:SetVisible(true)
		self.Qty:SetParent(self.Icon);
		self.Qty:SetSize(28,12);
		self.Qty:SetTop(16);
		local qty=tonumber(self.Qty:GetText());
		if qty==nil then qty=0 end
		if qty<10000 then
			self.Qty:SetFont(Turbine.UI.Lotro.Font.Verdana12);
		else
			self.Qty:SetFont(Turbine.UI.Lotro.Font.Verdana10);
		end
		self.Qty:SetText(self.Qty:GetText())
	else
		self.IconBack:SetVisible(false)
		self.Qty:SetParent(self.Back)
		self.Qty:SetSize(48,self.Text:GetHeight())
		self.Qty:SetTop(1)
		self.Qty:SetFont(Settings.fontFace)
		self.Qty:SetText(self.Qty:GetText())
	end
end
inventoryPanel.Update=function()
	-- only crop if an inventory type display is currently shown (no EI)
	if inventoryWindow.CharList:GetSelectedIndex()==3 then
		-- nothing to crop in EI ALL display
		inventoryPanel:SetWantsUpdates(false)
	elseif string.find(inventoryWindow.CharList:GetText(),"(EI)") ~=nil then
		-- need to crop mannequin?
		inventoryPanel:SetWantsUpdates(false)
	else	
		if inventoryPanel.cropDelay~=nil then
			inventoryPanel.cropTimeout=Turbine.Engine:GetGameTime()+inventoryPanel.cropDelay
			inventoryPanel.cropDelay=nil
		end
		if inventoryPanel.cropTimeout~=nil then
			if Turbine.Engine:GetGameTime()>inventoryPanel.cropTimeout then
				inventoryPanel.cropTimeout=nil
			end
			-- crop will occur on NEXT frame update, guarantees at least one frame of update if .cropDelay was not nil
		else
			cropAllIcons()
			inventoryPanel:SetWantsUpdates(false)
		end
	end
end
inventoryPanel.Refresh=function()
	-- will clear the main list and repopulate it with docked panels
	-- then will re-populate all of the panels

	local selCharName;
	if inventoryWindow.CharList:GetSelectedIndex()==1 then
		selCharName="ALL";
	elseif inventoryWindow.CharList:GetSelectedIndex()==2 then
		selCharName="Shared Storage";
	else
		selCharName=inventoryWindow.CharList:GetText();
	end
	local container=getCurrentContainer()

	inventoryPanel.ItemList:ClearItems()
	local tmpHdr=Turbine.UI.Control()
	tmpHdr.Name="Item List Header"
	tmpHdr:SetSize(inventoryPanel.ItemList:GetWidth(),20)
	tmpHdr:SetParent(inventoryPanel.ItemList)
	tmpHdr.MouseClick=function(sender,args)
		if args.Button==2 then
			-- show popup menu for New Group
			local menuItems=groupMenu:GetItems()
			menuItems:Clear()
			menuItems:Add(Turbine.UI.MenuItem(Resource[language][65]))
			menuItems:Get(1).Click=function(sender,args)
				-- show Group Maintenance dialog for new group
				groupMaint:Show()
			end
			groupMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
		end
	end
	tmpHdr.text=Turbine.UI.Label()
	tmpHdr.text:SetParent(tmpHdr)
	tmpHdr.text:SetSize(tmpHdr:GetWidth()-40,20)
	tmpHdr.text:SetText(Resource[language][64])
	tmpHdr.text:SetMouseVisible(false)

	tmpHdr.expandAll=Turbine.UI.Control()
	tmpHdr.expandAll:SetParent(tmpHdr)
	tmpHdr.expandAll:SetSize(16,16)
	tmpHdr.expandAll:SetPosition(tmpHdr:GetWidth()-38,2)
	tmpHdr.expandAll.normalImage=0x4100027b
	tmpHdr.expandAll.disabledImage=0x4100027a
	tmpHdr.expandAll.highlightImage=0x4100027c
	tmpHdr.expandAll:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	tmpHdr.expandAll:SetBackground(tmpHdr.expandAll.normalImage)
	tmpHdr.expandAll.MouseEnter=function(sender,args)
		sender:SetBackground(sender.highlightImage)
	end
	tmpHdr.expandAll.MouseLeave=function(sender,args)
		sender:SetBackground(sender.normalImage)
	end
	tmpHdr.expandAll.MouseClick=function(sender,args)
		local container=getCurrentContainer()
		for k,v in ipairs(displayTabs[container]) do
			if not v.expanded then
				v.panel.ExpandButton:MouseClick()
			end
		end
	end

	tmpHdr.collapseAll=Turbine.UI.Control()
	tmpHdr.collapseAll:SetParent(tmpHdr)
	tmpHdr.collapseAll:SetSize(16,16)
	tmpHdr.collapseAll:SetPosition(tmpHdr:GetWidth()-18,2)
	tmpHdr.collapseAll.normalImage=0x4100027e
	tmpHdr.collapseAll.disabledImage=0x4100027d
	tmpHdr.collapseAll.highlightImage=0x4100027f
	tmpHdr.collapseAll:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	tmpHdr.collapseAll:SetBackground(tmpHdr.collapseAll.normalImage)
	tmpHdr.collapseAll.MouseEnter=function(sender,args)
		sender:SetBackground(sender.highlightImage)
	end
	tmpHdr.collapseAll.MouseLeave=function(sender,args)
		sender:SetBackground(sender.normalImage)
	end
	tmpHdr.collapseAll.MouseClick=function(sender,args)
		local container=getCurrentContainer()
		for k,v in ipairs(displayTabs[container]) do
			if v.expanded then
				v.panel.ExpandButton:MouseClick()
			end
		end
	end

	inventoryPanel.ItemList:AddItem(tmpHdr)

	-- setup panels and windows and clear panel item lists
	for tmpKey,tmpContainer in pairs(displayTabs) do
		for k,v in ipairs(tmpContainer) do
			if type(v)=="table" then
				if tmpKey==container then
					if v.panel==nil then v.panel=createDisplayTabPanel(v) end
					v.panel.TitleBar.MouseClick=function(sender,args)
						local panel=sender:GetParent()
						local dt=panel.tabHandle
						if args.Button==2 then
							-- show popup menu for New Group
							groupMenu.displayTab=dt
							local menuItems=groupMenu:GetItems()
							menuItems:Clear()
							menuItems:Add(Turbine.UI.MenuItem(Resource[language][67]))
							menuItems:Get(1).Click=function(sender,args)
								groupMaint:Show(getCurrentContainer(),dt)
							end
							menuItems:Add(Turbine.UI.MenuItem(Resource[language][69]))
							if dt.isMain then
								menuItems:Get(2):SetEnabled(false)
							else
								menuItems:Get(2).Click=function(sender,args)
									deleteGroup(nil,groupMenu.displayTab)
								end
							end
							-- only add option if group is not docked (max height only applies to undocked groups)
							if not dt.docked then
								local menuItem=Turbine.UI.MenuItem(Resource[language][135])
								menuItem:SetChecked(dt.lockHeight)
								menuItem.dt=dt
								menuItem.Click=function(sender,args)
									sender.dt.lockHeight=not sender.dt.lockHeight
									sender.dt.panel.ItemList:Layout()
									local index=getDisplayTabIndexFromName(nil,sender.dt.name)
									if index~=nil then
										updateGroupMaint(index)
									end
								end
								menuItems:Add(menuItem)
							end

							groupMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20)
						else
							-- pass click to the expand button
							panel.ExpandButton:MouseClick(args)
						end
					end
					if v.docked then
						if v.window~=nil then
							v.window:SetVisible(false)
						end
						v.panel:SetParent(inventoryPanel.ItemList)
						inventoryPanel.ItemList:AddItem(v.panel)
					else
						if v.window==nil then
							v.window=createDTWindow(dt)
							if v.left==nil then v.left=displayWidth/2 end
							if v.left>displayWidth-40 then v.left=displayWidth-40 end
							if v.top==nil then v.top=displayHeight/2 end
							if v.top>displayHeight-20 then v.top=displayHeight-20 end
							v.window:SetPosition(v.left,v.top)
							if v.width==nil then v.width=inventoryPanel.ItemList:GetWidth() end
							if v.expanded then
								v.window:SetSize(v.width,v.panel:GetHeight()+4)
							else
								v.window:SetSize(v.width,20)
							end
						end
						if v.panel~=nil then
							v.panel:SetParent(v.window)
						end
						v.window:SetVisible(true)
					end
				else
					if v.window~=nil then
						if v.panel~=nil then
							v.panel:SetParent(v.window)
						end
						v.window:SetVisible(false)
					end
				end
				if v.panel~=nil then
					v.panel.ItemList:ClearItems()
				end
			end
		end
	end

	-- Now display the item entries
	local filterIndex;

	for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
		inventoryPanel.FilterList:HideEntry(filterIndex);
	end

	local showUndefined=false
	getItemEntryLayout()

	-- populate panel.ItemList lists based on displayTabXref
	-- note, filter list is a category list (poor choice of name when it was first implemented)
	for k,v in pairs(accountItemQty) do
		for filterIndex=2,inventoryPanel.FilterList.ListData:GetItemCount() do
			if v.Category==inventoryPanel.FilterList.ListData:GetItem(filterIndex).DataValue and (inventoryWindow.CharList:GetSelectedIndex()==1 or v.Qty[inventoryWindow.CharList:GetText()]~=nil) then
				inventoryPanel.FilterList:ShowEntry(filterIndex);
				break;
			end
		end
		-- there shouldn't BE any undefined categories after U37 until SSG adds new categories
		if v.SortCategory==Turbine.Gameplay.ItemCategory.Undefined then
			showUndefined=true;
		end
		if v.ListItem==nil then v.ListItem={} end
		local itemDisplayTab=displayTabs[container][displayTabXref[k]]		
		if selCharName=="ALL" then
			local tmpEntry=ItemEntry(k,"ALL",v.Total,v.ItemInfo,v.Category, v.InfoName);
			v.ListItem["ALL"]=tmpEntry;
			itemDisplayTab.panel.ItemList:AddItem(tmpEntry)
			if v.Qty[charName]~=nil then
				for char,qty in pairs(v.Qty) do
					if char==selCharName then
						local tmpEntry=ItemEntry(k,selCharName,qty.Subtotal,v.ItemInfo,v.Category, v.InfoName);
						v.ListItem[selCharName]=tmpEntry;
					else
						v.ListItem[char]=nil;
					end
				end
			end
		elseif selCharName==charName and not Settings.totalsOnly then
			v.ListItem["ALL"]=nil;
			if selCharName==charName and v.ListItem[charName]==nil then v.ListItem[charName]={} end
			if v.Qty[selCharName]~=nil then
				for char,qty in pairs(v.Qty) do
					if char==selCharName then
						v.ListItem[selCharName]={};
						for index,qty2 in pairs(qty) do
							if index~="Subtotal" then
--Turbine.Shell.WriteLine("k:"..tostring(k).." selCharName:"..tostring(selCharName).." qty2:"..tostring(qty2).." v.ItemInfo:"..tostring(v.ItemInfo))
								local tmpEntry=ItemEntry(k,selCharName,qty2,v.ItemInfo,v.Category, v.InfoName);
								tmpEntry.Item:SetItem(backPack:GetItem(index))
								v.ListItem[selCharName][index]=tmpEntry;
								tmpEntry.Item.Index=index;
								itemDisplayTab.panel.ItemList:AddItem(tmpEntry);
							end
						end
					else
						v.ListItem[char]=nil;
					end
				end
			end
		else
			if v.ListItem[selCharName]~=nil and v.ListItem[selCharName].IconBack==nil then v.ListItem[selCharName]=nil end; -- Settings.totalsOnly was previously on
			v.ListItem["ALL"]=nil;
			if v.Qty[selCharName]~=nil then
				for char,qty in pairs(v.Qty) do
					if char==selCharName then
						local tmpEntry
						if type(qty)=="table" then
							tmpEntry=ItemEntry(k,selCharName,qty.Subtotal,v.ItemInfo,v.Category, v.InfoName);
						else
							tmpEntry=ItemEntry(k,selCharName,qty,v.ItemInfo,v.Category, v.InfoName);
						end
						v.ListItem[selCharName]=tmpEntry;
						itemDisplayTab.panel.ItemList:AddItem(tmpEntry);
					else
						v.ListItem[char]=nil;
					end
				end
			end
		end
	end
end

--***** Equipped Items panel ***************************************************************************
equipmentWearColor={}
equipmentWearColor[Turbine.Gameplay.ItemWearState.Undefined]=Turbine.UI.Color.Grey
equipmentWearColor[Turbine.Gameplay.ItemWearState.Damaged]=Turbine.UI.Color.Yellow
equipmentWearColor[Turbine.Gameplay.ItemWearState.Pristine]=Turbine.UI.Color.Green
equipmentWearColor[Turbine.Gameplay.ItemWearState.Broken]=Turbine.UI.Color.Red
equipmentWearColor[Turbine.Gameplay.ItemWearState.Worn]=Turbine.UI.Color.LightGreen

equipmentLayout={}
equipmentLayout[0]={0,0} -- mannequin centered
equipmentLayout[0].background=0x41007e95;
equipmentLayout[0].Text="Mannequin";
equipmentLayout[0].Class=Turbine.Gameplay.Equipment.Undefined; -- should be same as index...
equipmentLayout[1]={119,-30} -- orig 149
equipmentLayout[1].background=0x41007eed;
equipmentLayout[1].Text="Head Slot";
equipmentLayout[1].Class=Turbine.Gameplay.Equipment.Head;
equipmentLayout[2]={119,16};
equipmentLayout[2].background=0x41007ef0;
equipmentLayout[2].Text="Chest Slot";
equipmentLayout[2].Class=Turbine.Gameplay.Equipment.Chest;
equipmentLayout[3]={119,108};
equipmentLayout[3].background=0x41007ef1;
equipmentLayout[3].Text="Legs Slot";
equipmentLayout[3].Class=Turbine.Gameplay.Equipment.Legs;
equipmentLayout[4]={119,62};
equipmentLayout[4].background=0x41007ef2;
equipmentLayout[4].Text="Gloves Slot";
equipmentLayout[4].Class=Turbine.Gameplay.Equipment.Gloves;
equipmentLayout[5]={163,108};
equipmentLayout[5].background=0x41007ef5;
equipmentLayout[5].Text="Boots Slot";
equipmentLayout[5].Class=Turbine.Gameplay.Equipment.Boots;
equipmentLayout[6]={163,-30} -- orig 203
equipmentLayout[6].background=0x41007eee;
equipmentLayout[6].Text="Shoulder Slot";
equipmentLayout[6].Class=Turbine.Gameplay.Equipment.Shoulder;
equipmentLayout[7]={163,16}
equipmentLayout[7].background=0x41007ee9;
equipmentLayout[7].Text="Back Slot";
equipmentLayout[7].Class=Turbine.Gameplay.Equipment.Back;
equipmentLayout[8]={-98,62};
equipmentLayout[8].background=0x41007ef8;
equipmentLayout[8].Text="Left Bracelet Slot";
equipmentLayout[8].Class=Turbine.Gameplay.Equipment.Bracelet1;
equipmentLayout[9]={-54,62};
equipmentLayout[9].background=0x41007ef9;
equipmentLayout[9].Text="Right Bracelet Slot";
equipmentLayout[9].Class=Turbine.Gameplay.Equipment.Bracelet2;
equipmentLayout[10]={-98,16};
equipmentLayout[10].background=0x41007eef;
equipmentLayout[10].Text="Necklace Slot";
equipmentLayout[10].Class=Turbine.Gameplay.Equipment.Necklace;
equipmentLayout[11]={-98,108};
equipmentLayout[11].background=0x41007ef3;
equipmentLayout[11].Text="Left Ring Slot";
equipmentLayout[11].Class=Turbine.Gameplay.Equipment.Ring1;
equipmentLayout[12]={-54,108};
equipmentLayout[12].background=0x41007ef4;
equipmentLayout[12].Text="Right Ring Slot";
equipmentLayout[12].Class=Turbine.Gameplay.Equipment.Ring2;
equipmentLayout[13]={-98,-30} -- orig -138
equipmentLayout[13].background=0x41007ef6;
equipmentLayout[13].Text="Left Ear Slot";
equipmentLayout[13].Class=Turbine.Gameplay.Equipment.Earring1;
equipmentLayout[14]={-54,-30} --orig -84 
equipmentLayout[14].background=0x41007ef7;
equipmentLayout[14].Text="Right Ear Slot";
equipmentLayout[14].Class=Turbine.Gameplay.Equipment.Earring2;
equipmentLayout[15]={-54,16};
equipmentLayout[15].background=0x41007efa;
equipmentLayout[15].Text="Pocket Slot";
equipmentLayout[15].Class=Turbine.Gameplay.Equipment.Pocket;
equipmentLayout[16]={-74,157};
equipmentLayout[16].background=0x41007eea;
equipmentLayout[16].Text="Primary Weapon Slot";
equipmentLayout[16].Class=Turbine.Gameplay.Equipment.PrimaryWeapon;
equipmentLayout[17]={-20,157};
equipmentLayout[17].background=0x41007eeb;
equipmentLayout[17].Text="Secondary Weapon/Shield Slot";
equipmentLayout[17].Class=Turbine.Gameplay.Equipment.SecondaryWeapon;
equipmentLayout[18]={34,157};
equipmentLayout[18].background=0x41007eec;
equipmentLayout[18].Text="Ranged Weapon/Instrument Slot";
equipmentLayout[18].Class=Turbine.Gameplay.Equipment.RangedWeapon;
equipmentLayout[19]={88,157};
equipmentLayout[19].background=0x41007efb;
equipmentLayout[19].Text="Craft Slot";
equipmentLayout[19].Class=Turbine.Gameplay.Equipment.CraftTool;
equipmentLayout[20]={142,157};
equipmentLayout[20].background=0x410e8680;
equipmentLayout[20].Text="Class Slot";
equipmentLayout[20].Class=Turbine.Gameplay.Equipment.Class;

equipmentPanel=Turbine.UI.Control()
equipmentPanel.Port=Turbine.UI.Control()
equipmentPanel.Port:SetParent(equipmentPanel)
equipmentPanel.Port:SetSize(301,252)
equipmentPanel.VScroll=Turbine.UI.Lotro.ScrollBar()
equipmentPanel.VScroll:SetParent(equipmentPanel)
equipmentPanel.VScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
equipmentPanel.VScroll:SetMinimum(0)
equipmentPanel.HScroll=Turbine.UI.Lotro.ScrollBar()
equipmentPanel.HScroll:SetParent(equipmentPanel)
equipmentPanel.HScroll:SetOrientation(Turbine.UI.Orientation.Horizontal)
equipmentPanel.HScroll:SetMinimum(0)
function CreateMannequin()
	local left=(equipmentPanel.Port:GetWidth()-110)/2
	local top=(equipmentPanel.Port:GetHeight()-148)/2
	if equipmentPanel.Mannequin~=nil then
		-- we are recreating, hide the old first
		equipmentPanel.Mannequin:SetVisible(false)
		equipmentPanel.Mannequin.Head=nil
		equipmentPanel.Mannequin.Chest=nil
		equipmentPanel.Mannequin.Legs=nil
		equipmentPanel.Mannequin.Wrist=nil
		equipmentPanel.Mannequin.Boot=nil
		equipmentPanel.Mannequin.Cape=nil
		equipmentPanel.Mannequin.Shoulder=nil
		equipmentPanel.Mannequin.Main=nil
		equipmentPanel.Mannequin.Secondary=nil
		equipmentPanel.Mannequin.Ranged=nil
		equipmentPanel.Mannequin.Craft=nil
		equipmentPanel.Mannequin=nil		
	end
	equipmentPanel.Mannequin=Turbine.UI.Control()
	equipmentPanel.Mannequin:SetParent(equipmentPanel.Port)
	equipmentPanel.Mannequin:SetPosition(left,top) -- centered
	equipmentPanel.Mannequin:SetSize(110,148)
	equipmentPanel.Mannequin:SetBackColor(Turbine.UI.Color.Grey) -- will change as item gets defined
	equipmentPanel.Mannequin:SetBlendMode(4)
	equipmentPanel.Mannequin:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin:SetBackground(equipmentLayout[0].background)

	equipmentPanel.Mannequin.Head=Turbine.UI.Control();
	equipmentPanel.Mannequin.Head:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Head:SetPosition(0,0);
	equipmentPanel.Mannequin.Head:SetBackground(0x41007e96);
	equipmentPanel.Mannequin.Head:SetSize(110,148);
	equipmentPanel.Mannequin.Head:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Head:SetBlendMode(4);
	equipmentPanel.Mannequin.Head:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Head:SetStretchMode(1); -- assigning mode 1 or 3 allows the image to blend with it's backcolor but overlay the image below (it's color doesn't affect the compound image)
	equipmentPanel.Mannequin.Head:SetMouseVisible(false);

	equipmentPanel.Mannequin.Chest=Turbine.UI.Control();
	equipmentPanel.Mannequin.Chest:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Chest:SetPosition(0,0);
	equipmentPanel.Mannequin.Chest:SetBackground(0x41007e97);
	equipmentPanel.Mannequin.Chest:SetSize(110,148);
	equipmentPanel.Mannequin.Chest:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Chest:SetBlendMode(4);
	equipmentPanel.Mannequin.Chest:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Chest:SetStretchMode(1);
	equipmentPanel.Mannequin.Chest:SetMouseVisible(false);

	equipmentPanel.Mannequin.Legs=Turbine.UI.Control();
	equipmentPanel.Mannequin.Legs:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Legs:SetPosition(0,0);
	equipmentPanel.Mannequin.Legs:SetBackground(0x41007e98);
	equipmentPanel.Mannequin.Legs:SetSize(110,148);
	equipmentPanel.Mannequin.Legs:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Legs:SetBlendMode(4);
	equipmentPanel.Mannequin.Legs:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Legs:SetStretchMode(1);
	equipmentPanel.Mannequin.Legs:SetMouseVisible(false);

	equipmentPanel.Mannequin.Wrist=Turbine.UI.Control();
	equipmentPanel.Mannequin.Wrist:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Wrist:SetPosition(0,0);
	equipmentPanel.Mannequin.Wrist:SetBackground(0x41007e99);
	equipmentPanel.Mannequin.Wrist:SetSize(110,148);
	equipmentPanel.Mannequin.Wrist:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Wrist:SetBlendMode(4);
	equipmentPanel.Mannequin.Wrist:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Wrist:SetStretchMode(1);
	equipmentPanel.Mannequin.Wrist:SetMouseVisible(false);

	equipmentPanel.Mannequin.Boot=Turbine.UI.Control();
	equipmentPanel.Mannequin.Boot:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Boot:SetPosition(0,0);
	equipmentPanel.Mannequin.Boot:SetBackground(0x41007e9a);
	equipmentPanel.Mannequin.Boot:SetSize(110,148);
	equipmentPanel.Mannequin.Boot:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Boot:SetBlendMode(4);
	equipmentPanel.Mannequin.Boot:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Boot:SetStretchMode(1);
	equipmentPanel.Mannequin.Boot:SetMouseVisible(false);

	equipmentPanel.Mannequin.Cape=Turbine.UI.Control();
	equipmentPanel.Mannequin.Cape:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Cape:SetPosition(0,0);
	equipmentPanel.Mannequin.Cape:SetBackground(0x41007e9b);
	equipmentPanel.Mannequin.Cape:SetSize(110,148);
	equipmentPanel.Mannequin.Cape:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Cape:SetBlendMode(4);
	equipmentPanel.Mannequin.Cape:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Cape:SetStretchMode(1);
	equipmentPanel.Mannequin.Cape:SetMouseVisible(false);

	equipmentPanel.Mannequin.Shoulder=Turbine.UI.Control();
	equipmentPanel.Mannequin.Shoulder:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Shoulder:SetPosition(0,0);
	equipmentPanel.Mannequin.Shoulder:SetBackground(0x41007e9c);
	equipmentPanel.Mannequin.Shoulder:SetSize(110,148);
	equipmentPanel.Mannequin.Shoulder:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Shoulder:SetBlendMode(4);
	equipmentPanel.Mannequin.Shoulder:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Shoulder:SetStretchMode(1);
	equipmentPanel.Mannequin.Shoulder:SetMouseVisible(false);

	equipmentPanel.Mannequin.Main=Turbine.UI.Control();
	equipmentPanel.Mannequin.Main:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Main:SetPosition(0,0);
	equipmentPanel.Mannequin.Main:SetBackground(0x41007e9d);
	equipmentPanel.Mannequin.Main:SetSize(110,148);
	equipmentPanel.Mannequin.Main:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Main:SetBlendMode(4);
	equipmentPanel.Mannequin.Main:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Main:SetStretchMode(1);
	equipmentPanel.Mannequin.Main:SetMouseVisible(false);

	equipmentPanel.Mannequin.Secondary=Turbine.UI.Control();
	equipmentPanel.Mannequin.Secondary:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Secondary:SetPosition(0,0);
	equipmentPanel.Mannequin.Secondary:SetBackground(0x41007e9e);
	equipmentPanel.Mannequin.Secondary:SetSize(110,148);
	equipmentPanel.Mannequin.Secondary:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Secondary:SetBlendMode(4);
	equipmentPanel.Mannequin.Secondary:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Secondary:SetStretchMode(1);
	equipmentPanel.Mannequin.Secondary:SetMouseVisible(false);

	equipmentPanel.Mannequin.Ranged=Turbine.UI.Control();
	equipmentPanel.Mannequin.Ranged:SetParent(equipmentPanel.Mannequin);
	equipmentPanel.Mannequin.Ranged:SetPosition(0,0);
	equipmentPanel.Mannequin.Ranged:SetBackground(0x41007e9f);
	equipmentPanel.Mannequin.Ranged:SetSize(110,148);
	equipmentPanel.Mannequin.Ranged:SetBackColor(equipmentWearColor[0]);
	equipmentPanel.Mannequin.Ranged:SetBlendMode(4);
	equipmentPanel.Mannequin.Ranged:SetBackColorBlendMode(1); -- 6
	equipmentPanel.Mannequin.Ranged:SetStretchMode(1);
	equipmentPanel.Mannequin.Ranged:SetMouseVisible(false);

	equipmentPanel.Mannequin.Craft=Turbine.UI.Control()
	equipmentPanel.Mannequin.Craft:SetParent(equipmentPanel.Mannequin)
	equipmentPanel.Mannequin.Craft:SetPosition(0,0)
	equipmentPanel.Mannequin.Craft:SetBackground(0x41007ea0)
	equipmentPanel.Mannequin.Craft:SetSize(110,148)
	equipmentPanel.Mannequin.Craft:SetBackColor(equipmentWearColor[0])
	equipmentPanel.Mannequin.Craft:SetBlendMode(4)
	equipmentPanel.Mannequin.Craft:SetBackColorBlendMode(1) -- 6
	equipmentPanel.Mannequin.Craft:SetStretchMode(1)
	equipmentPanel.Mannequin.Craft:SetMouseVisible(false)

	equipmentPanel.Mannequin.ControlXref={}
	equipmentPanel.Mannequin.ControlXref[1]=equipmentPanel.Mannequin.Head
	equipmentPanel.Mannequin.ControlXref[2]=equipmentPanel.Mannequin.Chest
	equipmentPanel.Mannequin.ControlXref[3]=equipmentPanel.Mannequin.Legs
	equipmentPanel.Mannequin.ControlXref[4]=equipmentPanel.Mannequin.Wrist
	equipmentPanel.Mannequin.ControlXref[5]=equipmentPanel.Mannequin.Boot
	equipmentPanel.Mannequin.ControlXref[6]=equipmentPanel.Mannequin.Shoulder
	equipmentPanel.Mannequin.ControlXref[7]=equipmentPanel.Mannequin.Cape
--	equipmentPanel.Mannequin.ControlXref[8]=equipmentPanel.Mannequin. bracelet 1
--	equipmentPanel.Mannequin.ControlXref[9]=equipmentPanel.Mannequin. bracelet 2
--	equipmentPanel.Mannequin.ControlXref[10]=equipmentPanel.Mannequin. necklace
--	equipmentPanel.Mannequin.ControlXref[11]=equipmentPanel.Mannequin. ring 1
--	equipmentPanel.Mannequin.ControlXref[12]=equipmentPanel.Mannequin. ring 2
--	equipmentPanel.Mannequin.ControlXref[13]=equipmentPanel.Mannequin. earring 1
--	equipmentPanel.Mannequin.ControlXref[14]=equipmentPanel.Mannequin. earring 2
--	equipmentPanel.Mannequin.ControlXref[15]=equipmentPanel.Mannequin. pocket
	equipmentPanel.Mannequin.ControlXref[16]=equipmentPanel.Mannequin.Main
	equipmentPanel.Mannequin.ControlXref[17]=equipmentPanel.Mannequin.Secondary -- also shield
	equipmentPanel.Mannequin.ControlXref[18]=equipmentPanel.Mannequin.Ranged -- also instrument
	equipmentPanel.Mannequin.ControlXref[19]=equipmentPanel.Mannequin.Craft
--	equipmentPanel.Mannequin.ControlXref[20]=equipmentPanel.Mannequin. class item
end
do
	equipmentPanel:SetSize(Settings.panelWidth*displayWidth,Settings.panelHeight*displayHeight)
	equipmentPanel.Port:SetLeft(equipmentPanel:GetWidth()-equipmentPanel.Port:GetWidth())
	equipmentPanel.Port:SetTop(equipmentPanel:GetHeight()-equipmentPanel.Port:GetHeight())
	if equipmentPanel.Port:GetLeft()<0 then equipmentPanel.Port:SetLeft(0) end
	if equipmentPanel.Port:GetTop()<0 then equipmentPanel.Port:SetTop(0) end
	local left=(equipmentPanel.Port:GetWidth()-110)/2
	local top=(equipmentPanel.Port:GetHeight()-148)/2
	CreateMannequin()
	equipmentPanel.Popup=Turbine.UI.ContextMenu()
	equipmentPanel.Popup:GetItems():Add(Turbine.UI.MenuItem(""))
	equipmentPanel.Popup:GetItems():Get(1):SetEnabled(false)
	equipmentPanel.Popup:GetItems():Add(Turbine.UI.MenuItem(""))
	equipmentPanel.Popup:GetItems():Get(2):SetEnabled(false)
	equipmentPanel.ItemSlot={}
	equipmentPanel.EquipmentSlot={}
	for k=1,20 do
		equipmentPanel.ItemSlot[k]=Turbine.UI.Control()
		equipmentPanel.ItemSlot[k]:SetParent(equipmentPanel.Port)
		equipmentPanel.ItemSlot[k]:SetSize(44,44)
		equipmentPanel.ItemSlot[k]:SetPosition(left+equipmentLayout[k][1],top+equipmentLayout[k][2])
		equipmentPanel.ItemSlot[k]:SetBackColor(Turbine.UI.Color.Grey)
		equipmentPanel.ItemSlot[k]:SetBlendMode(4)
		equipmentPanel.ItemSlot[k]:SetBackColorBlendMode(1)
		equipmentPanel.ItemSlot[k]:SetBackground(equipmentLayout[k].background)

		equipmentPanel.ItemSlot[k].ItemInfo=Turbine.UI.Lotro.ItemInfoControl()
		equipmentPanel.ItemSlot[k].ItemInfo:SetParent(equipmentPanel.ItemSlot[k])
		equipmentPanel.ItemSlot[k].ItemInfo:SetSize(34,34)
		equipmentPanel.ItemSlot[k].ItemInfo:SetPosition(2,2)
		equipmentPanel.ItemSlot[k].ItemInfo:SetMouseVisible(true)
		equipmentPanel.ItemSlot[k].ItemInfo:SetBlendMode(Turbine.UI.BlendMode.Overlay)
		equipmentPanel.ItemSlot[k].ItemInfo:SetVisible(false)

		equipmentPanel.ItemSlot[k].Icon=Turbine.UI.Control()
		equipmentPanel.ItemSlot[k].Icon:SetParent(equipmentPanel.ItemSlot[k])
		equipmentPanel.ItemSlot[k].Icon:SetSize(32,32)
		equipmentPanel.ItemSlot[k].Icon:SetPosition(6,6)
		equipmentPanel.ItemSlot[k].Icon:SetMouseVisible(true)
		equipmentPanel.ItemSlot[k].Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay)
		equipmentPanel.ItemSlot[k].Icon.Name=""
		equipmentPanel.ItemSlot[k].Icon:SetVisible(false)
		equipmentPanel.ItemSlot[k].Icon.MouseHover=function(sender,args)
			equipmentPanel.Popup:GetItems():Get(1):SetText(sender.Name)
			if sender.Name~=sender.InfoName then
				equipmentPanel.Popup:GetItems():Get(2):SetText(sender.InfoName)
			else
				equipmentPanel.Popup:GetItems():Get(2):SetText("")
			end
			equipmentPanel.Popup:ShowMenu()
			-- show name as popup
		end
		equipmentPanel.ItemSlot[k].Icon.Image=Turbine.UI.Control()
		equipmentPanel.ItemSlot[k].Icon.Image:SetParent(equipmentPanel.ItemSlot[k].Icon)
		equipmentPanel.ItemSlot[k].Icon.Image:SetSize(32,32)
		equipmentPanel.ItemSlot[k].Icon.Image:SetPosition(0,0)
		equipmentPanel.ItemSlot[k].Icon.Image:SetMouseVisible(false)
		equipmentPanel.ItemSlot[k].Icon.Image:SetBlendMode(Turbine.UI.BlendMode.Overlay)
		equipmentPanel.ItemSlot[k].Icon.Image:SetVisible(true)

		equipmentPanel.EquipmentSlot[k]=Turbine.UI.Lotro.EquipmentSlot()
		equipmentPanel.EquipmentSlot[k]:SetParent(equipmentPanel.ItemSlot[k])
		equipmentPanel.EquipmentSlot[k]:SetSize(38,38)
		equipmentPanel.EquipmentSlot[k]:SetPosition(3,3)
		equipmentPanel.EquipmentSlot[k]:SetAllowDrop(true)
		equipmentPanel.EquipmentSlot[k]:SetMouseVisible(true)
		equipmentPanel.EquipmentSlot[k]:SetEquipmentSlot(k)
	end
end
equipmentPanel.VScroll.ValueChanged=function()
	equipmentPanel.Port:SetTop(0-equipmentPanel.VScroll:GetValue())
end
equipmentPanel.HScroll.ValueChanged=function()
	equipmentPanel.Port:SetLeft(0-equipmentPanel.HScroll:GetValue())
end

if Settings.useMinimalHeader then
	equipmentPanel:SetParent(minimalWindow)
	equipmentPanel:SetPosition(minimalWindow.LeftMargin,minimalWindow.TopMargin)
else
	equipmentPanel:SetParent(inventoryWindow)
	equipmentPanel:SetPosition(inventoryWindow.LeftMargin,inventoryWindow.TopMargin)
end
equipmentPanel.Refresh=function()
if true then
	equipmentPanel:SetBackColor(Settings.backColor)
	local selCharName;
	if inventoryWindow.CharList:GetSelectedIndex()==1 then
		selCharName="ALL";
	elseif inventoryWindow.CharList:GetSelectedIndex()==2 then
		selCharName="Shared Storage";
	else
		selCharName=inventoryWindow.CharList:GetText();
	end
	-- don't try to refresh if we aren't showing and EI panel
	if string.find(selCharName,"(EI)")~=nil then
		local tmpCharName=string.gsub(selCharName," .(EI.)","")
		local ei
		if selCharName==charEIName then
			ei=localPlayer:GetEquipment()
		end
		-- redisplay icons
		for k=1,20 do
			equipmentPanel.EquipmentSlot[k]:SetVisible(selCharName==charEIName)
			equipmentPanel.ItemSlot[k].ItemInfo:SetVisible(false)
			equipmentPanel.ItemSlot[k].Icon:SetVisible(false)
			if selCharName~=charEIName then
				-- update icon and text in equipmentPanel.ItemSlot[k] from equippedItems[tmpCharName][k]
				local tmpItem
				if equippedItems[tmpCharName]~=nil then
					tmpItem=equippedItems[tmpCharName][k]
				end
				if tmpItem~=nil then
					equipmentPanel.ItemSlot[k].Icon.Name=tmpItem.Name
					equipmentPanel.ItemSlot[k].Icon.InfoName=tmpItem.InfoName
					--*** add item controls based on ItemData[tmpItem.Name,tmpItem.Category] - note, if named version doesn't exist, try showing based on ItemData[tmpItem.InfoName,tmpItem.Category]
					local itemIDList=findItemIDs(tmpItem.Name,tmpItem.Category,tmpItem.Durability,tmpItem.Quality,nil,nil,nil,1) --only want first response since we have no way of guessing between multiple items
					local itemID
					if itemIDList~=nil and itemIDList[1]~=nil and itemIDList[1][1]~=nil then itemID=itemIDList[1][1] end
					-- try again without durability and quality
					if itemID==nil then
						itemIDList=findItemIDs(tmpItem.Name,tmpItem.Category,nil,nil,nil,nil,nil,1) --only want first response since we have no way of guessing between multiple items
						if itemIDList~=nil and itemIDList[1]~=nil and itemIDList[1][1]~=nil then itemID=itemIDList[1][1] end
					end
					if itemID==nil then
						itemIDList=findItemIDs(tmpItem.InfoName,tmpItem.Category,tmpItem.Durability,tmpItem.Quality,nil,nil,nil,1) --only want first response since we have no way of guessing between multiple items
						if itemIDList~=nil and itemIDList[1]~=nil and itemIDList[1][1]~=nil then itemID=itemIDList[1][1] end
					end
					if itemID==nil then
						itemIDList=findItemIDs(tmpItem.InfoName,tmpItem.Category,nil,nil,nil,nil,nil,1) --only want first response since we have no way of guessing between multiple items
						if itemIDList~=nil and itemIDList[1]~=nil and itemIDList[1][1]~=nil then itemID=itemIDList[1][1] end
					end
					-- if itemID is nil at this point, do NOT show control
					local tmpItemInfo
					if itemID~=nil then
						tmpItemInfo=getItemInfo(itemID) -- even under best conditions, we still only return the generic item
						equipmentPanel.ItemSlot[k].ItemInfo:SetItemInfo(tmpItemInfo)
						equipmentPanel.ItemSlot[k].ItemInfo:SetVisible(true)
						equipmentPanel.ItemSlot[k].Icon:SetVisible(false)
					else
						equipmentPanel.ItemSlot[k].Icon:SetBackground(tmpItem.BackgroundImageID)
						equipmentPanel.ItemSlot[k].Icon.Image:SetBackground(tmpItem.IconImageID)
						equipmentPanel.ItemSlot[k].ItemInfo:SetVisible(false)
						equipmentPanel.ItemSlot[k].Icon:SetVisible(true)
					end
					if equipmentPanel.Mannequin.ControlXref[k]~=nil then
						equipmentPanel.Mannequin.ControlXref[k]:SetBackColor(equipmentWearColor[tmpItem.WearState])
					end
				else
					if equipmentPanel.Mannequin.ControlXref[k]~=nil then
						equipmentPanel.Mannequin.ControlXref[k]:SetBackColor(equipmentWearColor[0])
					end
				end
			else
				-- just refresh mannequin
				if equipmentPanel.Mannequin.ControlXref[k]~=nil then
					tmpItem=ei:GetItem(k)
					if tmpItem~=nil then
						equipmentPanel.Mannequin.ControlXref[k]:SetBackColor(equipmentWearColor[tmpItem:GetWearState()])
					else
						equipmentPanel.Mannequin.ControlXref[k]:SetBackColor(equipmentWearColor[0])
					end
				end
			end
		end
	end
end
end

equipmentPanel.Layout=function()
	local parent=equipmentPanel:GetParent()
	if true then
		local width,height
		if parent.LeftMargin~=nil then
			width=parent:GetWidth()-parent.LeftMargin*2
			height=parent:GetHeight()-parent.TopMargin-parent.BottomMargin
		else
			width=parent:GetWidth()
			height=parent:GetHeight()
		end
		-- re-layout controls
		local left=(width-110)/2
		local top=(height-148)/2
		equipmentPanel:SetSize(width,height)
		equipmentPanel.Port:SetPosition((width-equipmentPanel.Port:GetWidth())/2,(height-equipmentPanel.Port:GetHeight())/2)
		if equipmentPanel:GetWidth()<equipmentPanel.Port:GetWidth() then
			equipmentPanel.Port:SetLeft(0)
			equipmentPanel.HScroll:SetTop(equipmentPanel:GetHeight()-10)
			equipmentPanel.HScroll:SetWidth(equipmentPanel:GetWidth())
			equipmentPanel.HScroll:SetMaximum(equipmentPanel.Port:GetWidth()-equipmentPanel:GetWidth()+10)
			equipmentPanel.HScroll:SetValue(0)
			equipmentPanel.HScroll:SetVisible(true)
		else
			equipmentPanel.Port:SetLeft((width-equipmentPanel.Port:GetWidth())/2)
			equipmentPanel.HScroll:SetVisible(false)
		end
		if equipmentPanel:GetHeight()<equipmentPanel.Port:GetHeight() then
			equipmentPanel.Port:SetTop(0)
			equipmentPanel.VScroll:SetLeft(equipmentPanel:GetWidth()-10)
			equipmentPanel.VScroll:SetHeight(equipmentPanel:GetHeight())
			equipmentPanel.VScroll:SetMaximum(equipmentPanel.Port:GetHeight()-equipmentPanel:GetHeight()+10)
			equipmentPanel.VScroll:SetValue(0)
			equipmentPanel.VScroll:SetVisible(true)
		else
			equipmentPanel.Port:SetTop((height-equipmentPanel.Port:GetHeight())/2)
			equipmentPanel.VScroll:SetVisible(false)
		end
		-- recreate mannequin parts to fix mannequin zoom issues (when going from zoom2->zoom1 the stretchmode seems to mess up)
		CreateMannequin()
		-- now refresh so the new mannequin display will get the right colors...
		equipmentPanel.Refresh()
	end
end

equipmentPanel:Layout()

--***** allEI panel *************************************************************************
-- ALL (EI) panel
allEIPanel=Turbine.UI.Control()
allEIPanel.SortCaption=Turbine.UI.Label()
allEIPanel.SortCaption:SetParent(allEIPanel)
allEIPanel.SortCaption:SetPosition(9,5)
allEIPanel.SortCaption:SetSize(70,fontSize+2)
allEIPanel.SortCaption:SetForeColor(Settings.fontColor)
allEIPanel.SortCaption:SetOutlineColor(Turbine.UI.Color(0,0,0))
allEIPanel.SortCaption:SetFontStyle(Turbine.UI.FontStyle.Outline)
allEIPanel.SortCaption:SetFont(Settings.fontFace)
allEIPanel.SortCaption:SetText(Resource[language][15]..":")

allEIPanel.SortList=DropDownList()
allEIPanel.SortList:SetParent(allEIPanel)
allEIPanel.SortList:SetPosition(allEIPanel.SortCaption:GetLeft()+allEIPanel.SortCaption:GetWidth()+3,allEIPanel.SortCaption:GetTop()-3)
allEIPanel.SortList:SetSize(allEIPanel:GetWidth()/2-(allEIPanel.SortCaption:GetWidth()+allEIPanel.SortCaption:GetLeft())-1,fontSize+2)
allEIPanel.SortList:SetBorderColor(Settings.trimColor)
allEIPanel.SortList:SetCurrentBackColor(Settings.backColor)
allEIPanel.SortList:SetBackColor(Settings.backColor)
allEIPanel.SortList:SetTextColor(Settings.listTextColor)
allEIPanel.SortList:SetCurrentBackColor(Settings.backColor)
allEIPanel.SortList:SetDropRows(5)  -- increase as we implement the last 2 options
allEIPanel.SortList:SetZOrder(1)
allEIPanel.SortList:SetFont(Settings.fontFace)
allEIPanel.SortList:AddItem(Resource[language][99],1)
allEIPanel.SortList:AddItem(Resource[language][100],2)
allEIPanel.SortList:AddItem(Resource[language][121],3)

allEIPanel.SortList.SelectedIndexChanged=function()
	allEIPanel:Refresh()
end
loadAllEIPanelSlotType=function()
	allEIPanel.SlotType={} -- resource index, equipment type1, equipment type2 (for rings, bracelets, and earrings)
	allEIPanel.SlotType[1]={Resource[language][101],Turbine.Gameplay.Equipment.Head}
	allEIPanel.SlotType[2]={Resource[language][102],Turbine.Gameplay.Equipment.Chest}
	allEIPanel.SlotType[3]={Resource[language][103],Turbine.Gameplay.Equipment.Shoulder}
	allEIPanel.SlotType[4]={Resource[language][104],Turbine.Gameplay.Equipment.Gloves}
	allEIPanel.SlotType[5]={Resource[language][105],Turbine.Gameplay.Equipment.Boots}
	allEIPanel.SlotType[6]={Resource[language][106],Turbine.Gameplay.Equipment.Back}
	allEIPanel.SlotType[7]={Resource[language][107],Turbine.Gameplay.Equipment.Legs}
	allEIPanel.SlotType[8]={Resource[language][108],Turbine.Gameplay.Equipment.Ring1,Turbine.Gameplay.Equipment.Ring2}
	allEIPanel.SlotType[9]={Resource[language][109],Turbine.Gameplay.Equipment.Earring1,Turbine.Gameplay.Equipment.Earring2}
	allEIPanel.SlotType[10]={Resource[language][110],Turbine.Gameplay.Equipment.Necklace}
	allEIPanel.SlotType[11]={Resource[language][111],Turbine.Gameplay.Equipment.Pocket}
	allEIPanel.SlotType[12]={Resource[language][112],Turbine.Gameplay.Equipment.Bracelet1,Turbine.Gameplay.Equipment.Bracelet2}
	allEIPanel.SlotType[13]={Resource[language][113],Turbine.Gameplay.Equipment.PrimaryWeapon}
	allEIPanel.SlotType[14]={Resource[language][114],Turbine.Gameplay.Equipment.SecondaryWeapon} -- also Turbine.Gameplay.Equipment.Shield
	allEIPanel.SlotType[15]={Resource[language][115],Turbine.Gameplay.Equipment.RangedWeapon} -- also Turbine.Gameplay.Equipment.Instrument
	allEIPanel.SlotType[16]={Resource[language][116],Turbine.Gameplay.Equipment.Class}
	allEIPanel.SlotType[17]={Resource[language][117],Turbine.Gameplay.Equipment.CraftTool}

	table.sort(allEIPanel.SlotType,function(a,b) if a[1]<b[1] then return true end end)
	-- allEIPanel.SlotType should now be in alpha order by selected language - note, if we change languages, we need to call loadAllEIPanelSlotType()
end
loadAllEIPanelSlotType()

allEIPanel.FilterCaption=Turbine.UI.Label()
allEIPanel.FilterCaption:SetParent(allEIPanel)
allEIPanel.FilterCaption:SetPosition(allEIPanel.SortCaption:GetLeft(),allEIPanel.SortCaption:GetTop()+allEIPanel.SortCaption:GetHeight()+5)
allEIPanel.FilterCaption:SetSize(70,fontSize+2)
allEIPanel.FilterCaption:SetForeColor(Settings.fontColor)
allEIPanel.FilterCaption:SetOutlineColor(Turbine.UI.Color(0,0,0))
allEIPanel.FilterCaption:SetFontStyle(Turbine.UI.FontStyle.Outline)
allEIPanel.FilterCaption:SetFont(Settings.fontFace)
allEIPanel.FilterCaption:SetText(Resource[language][11]..":")

allEIPanel.FilterBack=Turbine.UI.Control()
allEIPanel.FilterBack:SetParent(allEIPanel)
allEIPanel.FilterBack:SetPosition(allEIPanel.SortList:GetLeft(),allEIPanel.FilterCaption:GetTop())
allEIPanel.FilterBack:SetSize(allEIPanel:GetWidth()-allEIPanel.FilterBack:GetLeft()-5,fontSize+2)
allEIPanel.FilterBack:SetBackColor(Settings.trimColor)
allEIPanel.Filter=Turbine.UI.TextBox()
allEIPanel.Filter:SetParent(allEIPanel.FilterBack)
allEIPanel.Filter:SetPosition(1,1)
allEIPanel.Filter:SetSize(allEIPanel.FilterBack:GetWidth()-2,allEIPanel.FilterBack:GetHeight()-2)
allEIPanel.Filter:SetFont(Settings.fontFace)
allEIPanel.Filter:SetBackColor(Settings.backColor)
allEIPanel.Filter.TextChanged=function()
	allEIPanel.Refresh()
end

allEIPanel.ItemList=Turbine.UI.ListBox()
allEIPanel.ItemList:SetParent(allEIPanel)
allEIPanel.ItemList:SetPosition(0,allEIPanel.FilterCaption:GetTop()+allEIPanel.FilterCaption:GetHeight()+5)

allEIPanel.VScroll=Turbine.UI.Lotro.ScrollBar()
allEIPanel.VScroll:SetParent(allEIPanel)
allEIPanel.VScroll:SetOrientation(Turbine.UI.Orientation.Vertical)
allEIPanel.VScroll:SetMinimum(0)
allEIPanel.VScroll:SetTop(allEIPanel.ItemList:GetTop())
allEIPanel.VScroll:SetWidth(10)
allEIPanel.VScroll:SetTop(allEIPanel.ItemList:GetTop())
allEIPanel.ItemList:SetVerticalScrollBar(allEIPanel.VScroll)

allEIPanel:SetSize(Settings.panelWidth*displayWidth,Settings.panelHeight*displayHeight)
if Settings.useMinimalHeader then
	allEIPanel:SetParent(minimalWindow)
	allEIPanel:SetPosition(minimalWindow.LeftMargin,minimalWindow.TopMargin)
else
	allEIPanel:SetParent(inventoryWindow)
	allEIPanel:SetPosition(inventoryWindow.LeftMargin,inventoryWindow.TopMargin)
end
allEIPanel.Refresh=function()
if true then
	-- reapply colors in case settings changed
	allEIPanel:SetBackColor(Settings.backColor)
	allEIPanel.SortList:SetBorderColor(Settings.trimColor)
	allEIPanel.SortList:SetCurrentBackColor(Settings.backColor)
	allEIPanel.SortList:SetBackColor(Settings.backColor)
	allEIPanel.SortList:SetTextColor(Settings.listTextColor)
	allEIPanel.Filter:SetBackColor(Settings.backColor)
	allEIPanel.FilterCaption:SetForeColor(Settings.fontColor)
	allEIPanel.SortCaption:SetForeColor(Settings.fontColor)
	
	-- only actually update if the "All (EI)" entry is selected
	if inventoryWindow.CharList:GetSelectedIndex()==3 then
		-- clear the EI list
		allEIPanel.ItemList:ClearItems()
		-- refresh the list of items
		local tmpEIList={}
		for charName,charData in pairs(CharList) do
			if charName~="All (EI)" and string.find(charName,"(EI)")~=nil then
				local tmpCharName=string.gsub(charName," .(EI.)","")
				if equippedItems[tmpCharName]~=nil then
					for slotIndex,slotType in ipairs(allEIPanel.SlotType) do
						if equippedItems[tmpCharName][slotType[2]]~=nil then
							local tmpName=equippedItems[tmpCharName][slotType[2]].Name
							if tmpName~=equippedItems[tmpCharName][slotType[2]].InfoName then
								tmpName=tmpName.." ("..equippedItems[tmpCharName][slotType[2]].InfoName..")"
							end
							table.insert(tmpEIList,{["char"]=tmpCharName,["slot"]=slotIndex,["name"]=tmpName})
						end
						if slotType[3]~=nil and equippedItems[tmpCharName][slotType[3]]~=nil then
							local tmpName=equippedItems[tmpCharName][slotType[3]].Name
							if tmpName~=equippedItems[tmpCharName][slotType[3]].InfoName then
								tmpName=tmpName.." ("..equippedItems[tmpCharName][slotType[3]].InfoName..")"
							end
							table.insert(tmpEIList,{["char"]=tmpCharName,["slot"]=slotIndex,["name"]=tmpName})
						end
					end
				end
			end
		end
		-- apply sort order
		local sortType=allEIPanel.SortList:GetValue()
		if sortType==3 then
			-- sort item then char - ignore slot
			table.sort(tmpEIList,function(a,b) if a.name<b.name then return true else if a.name==b.name and a.char<b.char then return true end end end)
		elseif sortType==2 then
			-- sort char then slot then item name
			table.sort(tmpEIList,function(a,b) if a.char<b.char then return true else if a.char==b.char and a.slot<b.slot then return true else if a.char==b.char and a.slot==b.slot and a.name<b.name then return true end end end end)
		else
			-- sort slot then item then char (default)
			table.sort(tmpEIList,function(a,b) if a.slot<b.slot then return true else if a.slot==b.slot and a.name<b.name then return true else if a.slot==b.slot and a.name==b.name and a.char<b.char then return true end end end end)
		end
		local tmpChar,tmpItem,tmpSlot,lastRow
		local filter=string.stripaccent(string.lower(allEIPanel.Filter:GetText()))
		-- now create table entries
		local tmpCount
		local tmpText=""
		for k,v in ipairs(tmpEIList) do
			if sortType==3 then
				if filter=="" or string.find(string.stripaccent(string.lower(v.name)),filter,1,true)~=nil then
					if v.name==tmpItem and v.char==tmpChar and lastRow~=nil then
						--then increment count to the prior row and just ignore this one (character has two or more of same item)
						tmpCount=tmpCount+1
						lastRow.text:SetText(tmpText.." ("..tostring(tmpCount)..")")
					else
						tmpCount=1
						if v.name~=tmpItem then
							-- create item header row
							local tmpRow=Turbine.UI.Control()
							tmpRow:SetParent(allEIPanel.ItemList)
							tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
							tmpRow.text=Turbine.UI.Label()
							tmpRow.text:SetParent(tmpRow)
							tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
--*** WHY IS COLOR BEING IGNORED?!?!
							tmpRow.text:SetForeColor(Settings.headingsColor)
							tmpRow.text:SetFont(Settings.fontFace)
							tmpRow.text:SetForeColor(Settings.headingsColor)
							tmpRow.text:SetText(v.name)
							allEIPanel.ItemList:AddItem(tmpRow)
							tmpItem=v.name
							tmpChar=nil
						end
						-- create char row
						local tmpRow=Turbine.UI.Control()
						tmpRow:SetParent(allEIPanel.ItemList)
						tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
						tmpRow.text=Turbine.UI.Label()
						tmpRow.text:SetParent(tmpRow)
						tmpRow.text:SetLeft(100)
						tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
						tmpRow.text:SetForeColor(Settings.listTextColor)
						tmpRow.text:SetFont(Settings.fontFace)
						tmpRow.text:SetText(v.char)
						tmpText=v.char
						allEIPanel.ItemList:AddItem(tmpRow)
						tmpChar=v.char
						lastRow=tmpRow
					end
				end

			elseif sortType==2 then
				if filter=="" or string.find(string.stripaccent(string.lower(v.name)),filter,1,true)~=nil then
					if v.slot==tmpSlot and v.name==tmpItem and v.char==tmpChar and lastRow~=nil then
						--then increment count to the prior row and just ignore this one (character has two or more of same item)
						tmpCount=tmpCount+1
						lastRow.text:SetText(tmpText.." ("..tostring(tmpCount)..")")
					else
						tmpCount=1
						if v.char~=tmpChar then
							-- create char header row
							local tmpRow=Turbine.UI.Control()
							tmpRow:SetParent(allEIPanel.ItemList)
							tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
							tmpRow.text=Turbine.UI.Label()
							tmpRow.text:SetParent(tmpRow)
							tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
							tmpRow.text:SetForeColor(Settings.headingsColor)
							tmpRow.text:SetFont(Settings.fontFace)
							tmpRow.text:SetText(v.char)
							allEIPanel.ItemList:AddItem(tmpRow)
							tmpChar=v.char
							tmpSlot=nil
							tmpItem=nil
						end
						if v.slot~=tmpSlot then
							-- create slot header row
							local tmpRow=Turbine.UI.Control()
							tmpRow:SetParent(allEIPanel.ItemList)
							tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
							tmpRow.text=Turbine.UI.Label()
							tmpRow.text:SetParent(tmpRow)
							tmpRow.text:SetLeft(100)
							tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
							tmpRow.text:SetForeColor(Settings.headingsColor)
							tmpRow.text:SetFont(Settings.fontFace)
							tmpRow.text:SetText(allEIPanel.SlotType[v.slot][1])
							allEIPanel.ItemList:AddItem(tmpRow)
							tmpSlot=v.slot
							tmpItem=nil
						end
						-- create item row
						local tmpRow=Turbine.UI.Control()
						tmpRow:SetParent(allEIPanel.ItemList)
						tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
						tmpRow.text=Turbine.UI.Label()
						tmpRow.text:SetParent(tmpRow)
						tmpRow.text:SetLeft(200)
						tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
						tmpRow.text:SetForeColor(Settings.listTextColor)
						tmpRow.text:SetFont(Settings.fontFace)
						tmpRow.text:SetText(v.name)
						tmpText=v.name
						allEIPanel.ItemList:AddItem(tmpRow)
						tmpItem=v.name
						lastRow=tmpRow
					end
				end
			else
				if filter=="" or string.find(string.stripaccent(string.lower(v.name)),filter,1,true)~=nil then
					if v.slot==tmpSlot and v.name==tmpItem and v.char==tmpChar and lastRow~=nil then
						--then increment count to the prior row and just ignore this one (character has two or more of same item)
						tmpCount=tmpCount+1
						lastRow.text:SetText(tmpText.." ("..tostring(tmpCount)..")")
					else
						tmpCount=1
						if v.slot~=tmpSlot then
							-- create slot header row
							local tmpRow=Turbine.UI.Control()
							tmpRow:SetParent(allEIPanel.ItemList)
							tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
							tmpRow.text=Turbine.UI.Label()
							tmpRow.text:SetParent(tmpRow)
							tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
							tmpRow.text:SetFont(Settings.fontFace)
							tmpRow.text:SetText(allEIPanel.SlotType[v.slot][1])
							allEIPanel.ItemList:AddItem(tmpRow)
							tmpSlot=v.slot
							tmpItem=nil
							tmpChar=nil
						end
						if v.name~=tmpItem then
							-- create item header row
							local tmpRow=Turbine.UI.Control()
							tmpRow:SetParent(allEIPanel.ItemList)
							tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
							tmpRow.text=Turbine.UI.Label()
							tmpRow.text:SetParent(tmpRow)
							tmpRow.text:SetLeft(100)
							tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
							tmpRow.text:SetFont(Settings.fontFace)
							tmpRow.text:SetText(v.name)
							allEIPanel.ItemList:AddItem(tmpRow)
							tmpItem=v.name
							tmpChar=nil
						end
						-- create char row
						local tmpRow=Turbine.UI.Control()
						tmpRow:SetParent(allEIPanel.ItemList)
						tmpRow:SetSize(allEIPanel.ItemList:GetWidth(),fontSize+2)
						tmpRow.text=Turbine.UI.Label()
						tmpRow.text:SetParent(tmpRow)
						tmpRow.text:SetLeft(200)
						tmpRow.text:SetSize(tmpRow:GetWidth(),fontSize+2)
						tmpRow.text:SetFont(Settings.fontFace)
						tmpRow.text:SetText(v.char)
						tmpText=v.char
						allEIPanel.ItemList:AddItem(tmpRow)
						tmpChar=v.char
						lastRow=tmpRow
					end
				end
			end
		end
	end
end
end
allEIPanel.Layout=function(sender,args)
	local parent=allEIPanel:GetParent()
	if true then
		local width,height
		if parent.LeftMargin~=nil then
			width=parent:GetWidth()-parent.LeftMargin*2
			height=parent:GetHeight()-parent.TopMargin-parent.BottomMargin
		else
			width=parent:GetWidth()
			height=parent:GetHeight()
		end
		allEIPanel:SetSize(width,height)
		allEIPanel.SortList:SetWidth(width-allEIPanel.SortList:GetLeft()-5)
		allEIPanel.FilterBack:SetWidth(allEIPanel:GetWidth()-allEIPanel.FilterBack:GetLeft()-5)
		allEIPanel.Filter:SetWidth(allEIPanel.FilterBack:GetWidth()-2)

		allEIPanel.ItemList:SetWidth(width-10)
		for k=1,allEIPanel.ItemList:GetItemCount() do
			local tmpItem=allEIPanel.ItemList:GetItem(k)
			tmpItem:SetWidth(width-10)
		end
		allEIPanel.ItemList:SetHeight(height-allEIPanel.ItemList:GetTop()-5)
		allEIPanel.VScroll:SetLeft(width-10)
		allEIPanel.VScroll:SetHeight(allEIPanel.ItemList:GetHeight())
		allEIPanel.Refresh()
	end
end
allEIPanel:Layout()
inventoryWindow.VisibleChanged=function()
	if inventoryWindow:IsVisible() then
		if AIItemDataIndex==nil then
			rebuildAIItemDataIndex()
		end
	end
end
rebuildAIItemDataIndex=function()
	AIItemDataIndex={}
	-- build the AIItemDataIndex (only the first time user opens the window per session)
	for cat,items in pairs(AIItemData) do
		AIItemDataIndex[cat]={}
		for id,data in pairs(items) do
			table.insert(AIItemDataIndex[cat],id)
		end
		table.sort(AIItemDataIndex[cat])
	end
end