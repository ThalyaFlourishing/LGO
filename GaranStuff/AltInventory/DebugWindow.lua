-- creates an interactive debugging window
debugPath=string.gsub(getfenv(1)._.Name,"%.DebugWindow","").."."
debugResourcePath=string.gsub(debugPath,"%.","/").."Resources/"
debugBackColor=Turbine.UI.Color(.1,.1,.1);
debugFont=Turbine.UI.Lotro.Font.Verdana12;
if backColor==nil then backColor=Turbine.UI.Color(0,0,0) end
if fontColor==nil then fontColor=Turbine.UI.Color(1,1,1) end
if trimColor==nil then trimColor=Turbine.UI.Color(.7,.7,.7) end

debugWindow = Turbine.UI.Lotro.Window();
debugWindow:SetText("Debug Console");
debugWindow:SetVisible(false);
debugWindow:SetSize(640,680);
debugWindow:SetPosition((Turbine.UI.Display:GetWidth() - debugWindow:GetWidth()) / 2,(Turbine.UI.Display:GetHeight() - debugWindow:GetHeight()) / 2);
debugWindow:SetBackColor(debugBackColor);

debugWindow.CommandPanel=Turbine.UI.Control();
debugWindow.CommandPanel:SetParent(debugWindow);
debugWindow.CommandPanel:SetSize(debugWindow:GetWidth()-30,200);
debugWindow.CommandPanel:SetPosition(15,40);
debugWindow.CommandPanel:SetBackColor(debugBackColor);

debugWindow.CommandCaption=Turbine.UI.Label();
debugWindow.CommandCaption:SetParent(debugWindow.CommandPanel);
debugWindow.CommandCaption:SetSize(100,20);
debugWindow.CommandCaption:SetPosition(1,1);
debugWindow.CommandCaption:SetFont(debugFont);
debugWindow.CommandCaption:SetText("Command:");

debugWindow.CommandEnvironment=RadioButtonGroup();
debugWindow.CommandEnvironment:SetParent(debugWindow.CommandPanel);
debugWindow.CommandEnvironment:SetPosition(debugWindow.CommandCaption:GetLeft()+debugWindow.CommandCaption:GetWidth()+15,debugWindow.CommandCaption:GetTop()-2);
debugWindow.CommandEnvironment:SetSize(debugWindow.CommandPanel:GetWidth()-debugWindow.CommandEnvironment:GetLeft()-3,20);
debugWindow.CommandEnvironment.UnselectedIcon=debugResourcePath.."RB_unselected.tga";
debugWindow.CommandEnvironment.SelectedIcon=debugResourcePath.."RB_selected.tga";
debugWindow.CommandEnvironment.IconWidth=16;
debugWindow.CommandEnvironment.IconHeight=16;
debugWindow.CommandEnvironment:SetBorderColor(backColor);
debugWindow.CommandEnvironment:SetBackColor(backColor);
debugWindow.CommandEnvironment:SetTextColor(fontColor);
debugWindow.CommandEnvironment:AddChoice("Global Environment");
debugWindow.CommandEnvironment:AddChoice("Plugin Environment");
debugWindow.CommandEnvironment:SetRows(1);
debugWindow.CommandEnvironment:SetSelectedChoice(2);

debugWindow.CommandText=Turbine.UI.Lotro.TextBox();
debugWindow.CommandText:SetParent(debugWindow.CommandPanel);
debugWindow.CommandText:SetSize(debugWindow.CommandPanel:GetWidth()-12,debugWindow.CommandPanel:GetHeight()-62);
debugWindow.CommandText:SetPosition(1,31);
debugWindow.CommandText:SetFont(debugFont);
debugWindow.CommandText:SetSelectable(true);

debugWindow.CommandText.VScroll=Turbine.UI.Lotro.ScrollBar();
debugWindow.CommandText.VScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
debugWindow.CommandText.VScroll:SetParent(debugWindow.CommandPanel);
debugWindow.CommandText.VScroll:SetPosition(debugWindow.CommandText:GetLeft()+debugWindow.CommandText:GetWidth(),debugWindow.CommandText:GetTop());
debugWindow.CommandText.VScroll:SetWidth(10);
debugWindow.CommandText.VScroll:SetHeight(debugWindow.CommandText:GetHeight());
debugWindow.CommandText:SetVerticalScrollBar(debugWindow.CommandText.VScroll);

-- get around the glitch with selected text in a textbox
debugWindow.CommandText.selectedText="";
debugWindow.CommandText.selectionStart=0;
debugWindow.CommandText.selectionLength=0;
debugWindow.CommandText.Update=function()
	debugWindow.CommandText.selectionStart=debugWindow.CommandText:GetSelectionStart();
	debugWindow.CommandText.selectionLength=debugWindow.CommandText:GetSelectionLength();
	debugWindow.CommandText.selectedText=debugWindow.CommandText:GetSelectedText();
end
debugWindow.CommandText.FocusGained=function()
	debugWindow.CommandText:SetWantsUpdates(true);
end
debugWindow.CommandText.FocusLost=function()
	debugWindow.CommandText:SetWantsUpdates(false);
	debugWindow.CommandText:SetSelectionLength(debugWindow.CommandText.selectionLength);
	debugWindow.CommandText:SetSelectionStart(debugWindow.CommandText.selectionStart);
end

debugWindow.ExecuteText=function(tmpString)
	userFunc,error=loadstring(tmpString);
	if userFunc==nil then
		Turbine.Shell.WriteLine("Error:"..tostring(error));
	else
		if debugWindow.CommandEnvironment:GetSelectedChoice()==1 then
			setfenv(userFunc,getfenv(0)); -- we set the environment to the global environment
		else
			setfenv(userFunc,getfenv()); -- we set the environment to the current plugin environment
		end
		success,retval=pcall(userFunc);
		if success then
			-- if table, yada yada yada
			if type(retval)=="table" then
				Table.Dump(retval)
			else
				Turbine.Shell.WriteLine(tostring(retval));
			end
		else
			Turbine.Shell.WriteLine("Error:"..tostring(retval))
		end
	end
end

debugWindow.CommandExec=Turbine.UI.Lotro.Button();
debugWindow.CommandExec:SetParent(debugWindow.CommandPanel);
debugWindow.CommandExec:SetSize(140,20);
debugWindow.CommandExec:SetText("Execute");
debugWindow.CommandExec:SetPosition((debugWindow.CommandPanel:GetWidth() - debugWindow.CommandExec:GetWidth()*2) / 3, debugWindow.CommandPanel:GetHeight()-21);
debugWindow.CommandExec.Click=function()
	debugWindow.ExecuteText(debugWindow.CommandText:GetText());
end

debugWindow.CommandExecSelected=Turbine.UI.Lotro.Button();
debugWindow.CommandExecSelected:SetParent(debugWindow.CommandPanel);
debugWindow.CommandExecSelected:SetSize(140,20);
debugWindow.CommandExecSelected:SetText("Execute Selected");
debugWindow.CommandExecSelected:SetPosition((debugWindow.CommandPanel:GetWidth() - debugWindow.CommandExec:GetWidth()*2)*2/3+debugWindow.CommandExec:GetWidth(), debugWindow.CommandPanel:GetHeight()-21);
debugWindow.CommandExecSelected.Click=function()
	local selectedText=debugWindow.CommandText.selectedText;
	if selectedText~=nil and selectedText~="" then
		debugWindow.ExecuteText(selectedText);
	else
		Turbine.Shell.WriteLine("You must select some text before clicking 'Execute Selected'.")
	end
end

debugWindow.WatchPanel=Turbine.UI.Control();
debugWindow.WatchPanel:SetParent(debugWindow);
debugWindow.WatchPanel:SetPosition(15,debugWindow.CommandPanel:GetTop()+debugWindow.CommandPanel:GetHeight()+10);
debugWindow.WatchPanel:SetSize(debugWindow:GetWidth()-30,200);
debugWindow.WatchPanel:SetBackColor(debugBackColor);

debugWindow.WatchCaption=Turbine.UI.Label();
debugWindow.WatchCaption:SetParent(debugWindow.WatchPanel);
debugWindow.WatchCaption:SetSize(180,20);
debugWindow.WatchCaption:SetPosition(1,1);
debugWindow.WatchCaption:SetFont(debugFont);
debugWindow.WatchCaption:SetText("Var/Prop:");

debugWindow.WatchText=Turbine.UI.Lotro.TextBox();
debugWindow.WatchText:SetParent(debugWindow.WatchPanel);
debugWindow.WatchText:SetSize(180,20)
debugWindow.WatchText:SetPosition(1,25);
debugWindow.WatchText:SetFont(debugFont);

debugWindow.WatchAdd=Turbine.UI.Lotro.Button();
debugWindow.WatchAdd:SetParent(debugWindow.WatchPanel);
debugWindow.WatchAdd:SetSize(180,20);
debugWindow.WatchAdd:SetPosition(1,55);
debugWindow.WatchAdd:SetText("Add");
debugWindow.WatchAdd.Click=function()
	-- attempt to resolve the watchtext
	local success, retval, error;

	result,error=loadstring("return "..debugWindow.WatchText:GetText());
	if result==nil then
		Turbine.Shell.WriteLine("Error:"..tostring(error));
	else
		setfenv(result,getfenv());
		success,retval=pcall(result);
		if success then
			-- if watchtext resolves without error, then add a watch list entry for it
			local tmpWatch=Turbine.UI.Label()
			tmpWatch:SetParent(debugWindow.WatchList);
			tmpWatch:SetSize(debugWindow.WatchList:GetWidth(),20);
			tmpWatch.Variable=debugWindow.WatchText:GetText();
			tmpWatch:SetFont(debugFont);
			tmpWatch:SetText(tmpWatch.Variable.."="..tostring((retval)));
			debugWindow.WatchList:AddItem(tmpWatch);
		else
			Turbine.Shell.WriteLine("Error:"..tostring(retval))
		end
	end
end

debugWindow.WatchRemove=Turbine.UI.Lotro.Button();
debugWindow.WatchRemove:SetParent(debugWindow.WatchPanel);
debugWindow.WatchRemove:SetSize(180,20);
debugWindow.WatchRemove:SetPosition(1,85);
debugWindow.WatchRemove:SetText("Remove");
debugWindow.WatchRemove.Click=function()
	if debugWindow.WatchList:GetSelectedIndex()~=nil and debugWindow.WatchList:GetSelectedIndex()>0 then
		-- remove the currently selected watch list item
		debugWindow.WatchList:RemoveItemAt(debugWindow.WatchList:GetSelectedIndex());
	end
	debugWindow.WatchList:SelectedIndexChanged();
end

debugWindow.WatchList=Turbine.UI.ListBox();
debugWindow.WatchList:SetParent(debugWindow.WatchPanel);
debugWindow.WatchList:SetSize(400, debugWindow.WatchPanel:GetHeight()-2);
debugWindow.WatchList:SetPosition(190, 1);
debugWindow.WatchList.OldSelectedIndex=nil;
debugWindow.WatchList.SelectedIndexChanged=function()
	if debugWindow.WatchList.OldSelectedIndex~=nil and debugWindow.WatchList.OldSelectedIndex>0 and debugWindow.WatchList.OldSelectedIndex<=debugWindow.WatchList:GetItemCount() then
		debugWindow.WatchList:GetItem(debugWindow.WatchList.OldSelectedIndex):SetBackColor(debugWindow:GetBackColor());
	end
	if debugWindow.WatchList:GetSelectedIndex()~=nil and debugWindow.WatchList:GetSelectedIndex()>0 and debugWindow.WatchList:GetSelectedIndex()<=debugWindow.WatchList:GetItemCount() then
		debugWindow.WatchList:GetItem(debugWindow.WatchList:GetSelectedIndex()):SetBackColor(Turbine.UI.Color(.2,.2,.6));
	end
	debugWindow.WatchList.OldSelectedIndex=debugWindow.WatchList:GetSelectedIndex();
end

debugWindow.WatchVScroll=Turbine.UI.Lotro.ScrollBar();
debugWindow.WatchVScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
debugWindow.WatchVScroll:SetParent(debugWindow.WatchPanel);
debugWindow.WatchVScroll:SetPosition(debugWindow.WatchList:GetLeft()+debugWindow.WatchList:GetWidth(),debugWindow.WatchList:GetTop());
debugWindow.WatchVScroll:SetWidth(12);
debugWindow.WatchVScroll:SetHeight(debugWindow.WatchList:GetHeight());
debugWindow.WatchList:SetVerticalScrollBar(debugWindow.WatchVScroll);
debugWindow.WatchVScroll:SetVisible(false);

debugWindow.InspectionPanel=Turbine.UI.Label(); -- needs to be a "scrollable" control
debugWindow.InspectionPanel:SetParent(debugWindow);
debugWindow.InspectionPanel:SetPosition(15,debugWindow.WatchPanel:GetTop()+debugWindow.WatchPanel:GetHeight()+10);
debugWindow.InspectionPanel:SetSize(debugWindow:GetWidth()-30,debugWindow:GetHeight()-debugWindow.InspectionPanel:GetTop()-30);
debugWindow.InspectionPanel:SetBackColor(debugBackColor);

debugWindow.InspectionTree=Turbine.UI.TreeView();
debugWindow.InspectionTree:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionTree:SetSize(debugWindow.InspectionPanel:GetWidth()/2-14,debugWindow.InspectionPanel:GetHeight()-14);
debugWindow.InspectionTree:SetPosition(1,1);
debugWindow.InspectionTree:SetIndentationWidth(0); -- I manually control the indentation so that rows don't miss the mouse events in the indentation area
debugWindow.InspectionTree.SelectedNode=nil;
debugWindow.InspectionTree.LastSelectedNode=nil;
debugWindow.InspectionTree.SelectedChanged=false;

debugWindow.InspectionVScroll=Turbine.UI.Lotro.ScrollBar();
debugWindow.InspectionVScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
debugWindow.InspectionVScroll:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionVScroll:SetPosition(debugWindow.InspectionTree:GetLeft()+debugWindow.InspectionTree:GetWidth(),debugWindow.InspectionTree:GetTop());
debugWindow.InspectionVScroll:SetWidth(12);
debugWindow.InspectionVScroll:SetHeight(debugWindow.InspectionPanel:GetHeight());
debugWindow.InspectionTree:SetVerticalScrollBar(debugWindow.InspectionVScroll);
debugWindow.InspectionVScroll:SetVisible(false);

debugWindow.InspectionHScroll=Turbine.UI.Lotro.ScrollBar();
debugWindow.InspectionHScroll:SetOrientation(Turbine.UI.Orientation.Horizontal);
debugWindow.InspectionHScroll:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionHScroll:SetPosition(debugWindow.InspectionTree:GetLeft(),debugWindow.InspectionTree:GetHeight()+debugWindow.InspectionTree:GetTop());
debugWindow.InspectionHScroll:SetHeight(12);
debugWindow.InspectionHScroll:SetWidth(debugWindow.InspectionTree:GetWidth());
debugWindow.InspectionTree:SetHorizontalScrollBar(debugWindow.InspectionHScroll);
debugWindow.InspectionHScroll:SetVisible(false);

debugWindow.InspectionTree.SelectedNodeChanged=function()
	debugWindow.InspectionTree.SelectedChanged=true;
	if debugWindow.InspectionTree.LastSelectedNode~=nil then
		debugWindow.InspectionTree.LastSelectedNode.Text:SetBackColor(debugBackColor);
	end
	debugWindow.InspectionTree.LastSelectedNode=debugWindow.InspectionTree.SelectedNode;
	if debugWindow.InspectionTree.SelectedNode~=nil then
		-- how do we get the fully qualified name?
		local tmpNode=debugWindow.InspectionTree.SelectedNode
		local tmpName=debugWindow.InspectionTree.SelectedNode.Text:GetText();
		local objType=type(debugWindow.InspectionTree.SelectedNode.Object);
		local isFunction=false;
		if objType=="function" then
			tmpName=tmpName.."()";
			isFunction=true;
		else
			if tonumber(tmpName)~=nil then 
				tmpName="["..tmpName.."]";
			end
		end
		if tmpNode.Tier>0 then
			tmpNode=tmpNode:GetParentNode();
			while tmpNode.Tier>0 do
				if tonumber(tmpNode.Text:GetText())~=nil then
					if isFunction then
						tmpName="["..tmpNode.Text:GetText().."]:"..tmpName;
						isFunction=false;
					else
						tmpName="["..tmpNode.Text:GetText().."]"..tmpName;
					end
				else
					if string.sub(tmpName,1,1)=="[" then
						tmpName=tmpNode.Text:GetText()..tmpName;
					else
						if isFunction then
							tmpName=tmpNode.Text:GetText()..":"..tmpName;
							isFunction=false;
						else
							tmpName=tmpNode.Text:GetText().."."..tmpName;
						end
					end
				end
				tmpNode=tmpNode:GetParentNode();
			end
		end
		debugWindow.InspectionName:SetText(tmpName);
		debugWindow.WatchText:SetText(tmpName);

		debugWindow.InspectionTree.SelectedNode.Text:SetBackColor(Turbine.UI.Color(.2,.2,.6));
		-- need to fill in the detail pane
		debugWindow.InspectionType:SetText(objType)
		if debugWindow.InspectionTree.SelectedNode.Object==nil then
			debugWindow.InspectionVal:SetText("nil");
		elseif objType=="number" then
			debugWindow.InspectionVal:SetText(tostring(debugWindow.InspectionTree.SelectedNode.Object).." ("..string.format("0x%x",tostring(debugWindow.InspectionTree.SelectedNode.Object))..")");
		elseif objType=="string" or objType=="boolean" then
			debugWindow.InspectionVal:SetText(tostring(debugWindow.InspectionTree.SelectedNode.Object));
		else
			debugWindow.InspectionVal:SetText("n/a");
		end
	end
end

debugWindow.InspectionName=Turbine.UI.Label();
debugWindow.InspectionName:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionName:SetSize(debugWindow.InspectionPanel:GetWidth()-debugWindow.InspectionTree:GetLeft()-debugWindow.InspectionTree:GetWidth()-14,20);
debugWindow.InspectionName:SetPosition(debugWindow.InspectionTree:GetLeft()+debugWindow.InspectionTree:GetWidth()+14,1);
debugWindow.InspectionName:SetFont(debugFont);
debugWindow.InspectionName:SetText("");

debugWindow.InspectionTypeCaption=Turbine.UI.Label();
debugWindow.InspectionTypeCaption:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionTypeCaption:SetSize(80,20);
debugWindow.InspectionTypeCaption:SetPosition(debugWindow.InspectionTree:GetWidth()+14,23);
debugWindow.InspectionTypeCaption:SetFont(debugFont);
debugWindow.InspectionTypeCaption:SetText("Type:");

debugWindow.InspectionType=Turbine.UI.Label();
debugWindow.InspectionType:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionType:SetSize(debugWindow.InspectionPanel:GetWidth()-debugWindow.InspectionType:GetLeft()-debugWindow.InspectionType:GetWidth()-2,20);
debugWindow.InspectionType:SetPosition(debugWindow.InspectionTypeCaption:GetLeft()+debugWindow.InspectionTypeCaption:GetWidth()+1,debugWindow.InspectionTypeCaption:GetTop());
debugWindow.InspectionType:SetFont(debugFont);
debugWindow.InspectionType:SetText("");

debugWindow.InspectionValCaption=Turbine.UI.Label();
debugWindow.InspectionValCaption:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionValCaption:SetSize(debugWindow.InspectionTypeCaption:GetWidth(),20);
debugWindow.InspectionValCaption:SetPosition(debugWindow.InspectionTypeCaption:GetLeft(),45);
debugWindow.InspectionValCaption:SetFont(debugFont);
debugWindow.InspectionValCaption:SetText("Value:");

debugWindow.InspectionVal=Turbine.UI.Label();
debugWindow.InspectionVal:SetParent(debugWindow.InspectionPanel);
debugWindow.InspectionVal:SetSize(debugWindow.InspectionType:GetWidth(),20);
debugWindow.InspectionVal:SetPosition(debugWindow.InspectionType:GetLeft(),debugWindow.InspectionValCaption:GetTop());
debugWindow.InspectionVal:SetFont(debugFont);
debugWindow.InspectionVal:SetText("");

debugWindow.AddNode=function(object,parentNodeList)
	-- for efficiency sake, child nodes are only built when they are actually exposed.
	local node=Turbine.UI.TreeNode();
	node:SetParent(debugWindow.InspectionTree)
	node:SetSize(debugWindow.InspectionTree:GetWidth(),20)
	node.Tier=0;
	node.Object=object;
	node.Icon=Turbine.UI.Control();
	node.Icon:SetParent(node);
	node.Icon:SetSize(16,16);
	node.Icon:SetPosition(0,2);
	node.Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	node.Icon:SetMouseVisible(false);
	node.Text=Turbine.UI.Label();
	node.Text:SetParent(node);
	node.Text:SetFont(debugFont);
	node.Text:SetSize(node:GetSize());
	node.Text:SetPosition(18,0)
	node.Text:SetText("_G");
	node.Text:SetMouseVisible(false);
	node.MouseDown=function(sender)
		debugWindow.InspectionTree.SelectedNode=node;
	end

	parentNodeList:Add(node)

	if type(node.Object)=="table" then
		node.Icon:SetBackground(0x41007e27);
		node.Icon:SetVisible(true);
		-- we create one bogus node so that the Turbine tree control will work
		local tmpNode=Turbine.UI.TreeNode();
		tmpNode:SetParent(debugWindow.InspectionTree)
		tmpNode:SetSize(debugWindow.InspectionTree:GetWidth(),20)
		tmpNode:SetBackColor(trimColor)
		tmpNode.MouseDown=function(sender)
			debugWindow.InspectionTree.SelectedNode=sender;
		end
		node:GetChildNodes():Add(tmpNode);

		tmpNode=Turbine.UI.TreeNode();
		tmpNode:SetParent(debugWindow.InspectionTree)
		tmpNode:SetSize(debugWindow.InspectionTree:GetWidth(),20)
		tmpNode:SetBackColor(fontColor)
		tmpNode.MouseDown=function(sender)
			debugWindow.InspectionTree.SelectedNode=sender;
		end
		node:GetChildNodes():Add(tmpNode);
	else
		node.Icon:SetBackground(0x410001a4);
		node.Icon:SetVisible(false);
	end
end

debugWindow.AddChildNodes=function(node)
	if node~=nil and debugWindow.InspectionTree.SelectedChanged and debugWindow.InspectionTree.SelectedNode:IsExpanded() then
		debugWindow.InspectionTree.SelectedChanged=false;
		local object=node.Object;
		local tmpNode;
		local childNodes=node:GetChildNodes();
		for tmpNode=1,childNodes:GetCount() do
			childNodes:Get(tmpNode).MouseDown=nil;
		end
		childNodes:Clear();
		local tmpChildObj={}
		if type(node.Object)=="table" then
			-- refresh the child nodes
			for k,v in pairs(node.Object) do
				tmpChildObj[#tmpChildObj+1]={k,v};
			end
			-- check if there is a metatable for this
			local tmpMetaTable=getmetatable(node.Object);
			if tmpMetaTable~=nil then
				for k,v in pairs(tmpMetaTable) do
					tmpChildObj[#tmpChildObj+1]={k,v};
				end
			end
			table.sort(tmpChildObj,function(arg1,arg2) if tostring(arg1[1])<tostring(arg2[1]) then return(true) end end);

			for tmpIndex=1,#tmpChildObj do
				local newNode;
				newNode=Turbine.UI.TreeNode();
				newNode:SetParent(debugWindow.InspectionTree);
				newNode.Tier=node.Tier+1;
				newNode:SetSize(debugWindow.InspectionTree:GetWidth()+16*newNode.Tier,20);
				newNode.Object=tmpChildObj[tmpIndex][2];
				newNode.Icon=Turbine.UI.Control();
				newNode.Icon:SetParent(newNode);
				newNode.Icon:SetSize(16,16);
				newNode.Icon:SetPosition(16*newNode.Tier,2);
				newNode.Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
				newNode.Icon:SetMouseVisible(false);

				newNode.Text=Turbine.UI.Label();
				newNode.Text:SetParent(newNode);
				newNode.Text:SetFont(debugFont);
				newNode.Text:SetSize(newNode:GetSize());
				newNode.Text:SetPosition(18+16*newNode.Tier,0)
				newNode.Text:SetText(tostring(tmpChildObj[tmpIndex][1]));
				newNode.Text:SetMouseVisible(false);
				childNodes:Add(newNode);
				newNode.MouseDown=function(sender)
					debugWindow.InspectionTree.SelectedNode=sender;
				end
				if type(newNode.Object)=="table" then
					newNode.Icon:SetBackground(0x41007e27);
					newNode:GetChildNodes():Add(Turbine.UI.TreeNode());
					newNode.Icon:SetVisible(true);
				else
					newNode.Icon:SetBackground(0x410001a4);
					newNode.Icon:SetVisible(false);
				end
			end
		else
			-- check if there is a metatable for this
			local tmpMetaTable=getmetatable(node.Object);
			if tmpMetaTable~=nil then
				for k,v in pairs(tmpMetaTable) do
					tmpChildObj[#tmpChildObj+1]={k,v};
				end

			end
			if #tmpChildObj>0 then
				table.sort(tmpChildObj,function(arg1,arg2) if arg1[1]<arg2[1] then return(true) end end);
				for tmpIndex=1,#tmpChildObj do
					local newNode;
					newNode=Turbine.UI.TreeNode();
					newNode:SetParent(debugWindow.InspectionTree);
					newNode.Tier=node.Tier+1;
					newNode:SetSize(debugWindow.InspectionTree:GetWidth()+16*newNode.Tier,20);
					newNode.Object=tmpChildObj[tmpIndex][2];
					newNode.Icon=Turbine.UI.Control();
					newNode.Icon:SetParent(newNode);
					newNode.Icon:SetSize(16,16);
					newNode.Icon:SetPosition(16*newNode.Tier,2);
					newNode.Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
					newNode.Icon:SetMouseVisible(false);

					newNode.Text=Turbine.UI.Label();
					newNode.Text:SetParent(newNode);
					newNode.Text:SetFont(debugFont);
					newNode.Text:SetSize(newNode:GetSize());
					newNode.Text:SetPosition(18+16*newNode.Tier,0)
					newNode.Text:SetText(tostring(tmpChildObj[tmpIndex][1]));
					newNode.Text:SetMouseVisible(false);
					childNodes:Add(newNode);
					newNode.MouseDown=function(sender)
						debugWindow.InspectionTree.SelectedNode=sender;
					end
					if type(newNode.Object)=="table" then
						newNode.Icon:SetBackground(0x41007e27);
						newNode:GetChildNodes():Add(Turbine.UI.TreeNode());
						newNode.Icon:SetVisible(true);
					else
						newNode.Icon:SetBackground(0x410001a4);
						newNode.Icon:SetVisible(false);
					end
				end
			else
				node.Icon:SetBackground(0x410001a4);
				node.Icon:SetVisible(false);
			end
		end
		debugWindow.InspectionTree.SelectedNode:SetExpanded(true);
	end
end

-- add the root global environment node
debugWindow.AddNode(_G,debugWindow.InspectionTree:GetNodes())

debugWindow.Update=function()
	-- attempt to resolve the watchtext
	local success, retval, error, tmpIndex;
	for tmpIndex=1,debugWindow.WatchList:GetItemCount() do
		result,error=loadstring("return "..debugWindow.WatchList:GetItem(tmpIndex).Variable);
		if result~=nil then
			setfenv(result,getfenv());
			success,retval=pcall(result);
			if success then
				-- if watchtext resolves without error, then add a watch list entry for it
				debugWindow.WatchList:GetItem(tmpIndex):SetText(debugWindow.WatchList:GetItem(tmpIndex).Variable.."="..tostring((retval)));
			else
				Turbine.Shell.WriteLine("Error:"..tostring(retval))
			end
		end
	end
	-- refresh the child nodes for the current node if it is expanded
	if debugWindow.InspectionTree.SelectedNode~=nil then
		debugWindow.AddChildNodes(debugWindow.InspectionTree.SelectedNode)
		if debugWindow.InspectionTree.SelectedNode:GetChildNodes():GetCount()>0 then
			if debugWindow.InspectionTree.SelectedNode:IsExpanded() then
				debugWindow.InspectionTree.SelectedNode.Icon:SetBackground(0x41007e26);
			else
				debugWindow.InspectionTree.SelectedNode.Icon:SetBackground(0x41007e27);
			end
			debugWindow.InspectionTree.SelectedNode.Icon:SetVisible(true);
		else
			debugWindow.InspectionTree.SelectedNode.Icon:SetBackground(0x410001a4);
			debugWindow.InspectionTree.SelectedNode.Icon:SetVisible(false);
		end
	end
end
debugWindow.VisibleChanged=function()
	debugWindow:SetWantsUpdates(debugWindow:IsVisible());
end
