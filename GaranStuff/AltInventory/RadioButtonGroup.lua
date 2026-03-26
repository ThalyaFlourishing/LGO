import "Turbine.UI";
import "Turbine.UI.Lotro";
RadioButtonGroup = class( Turbine.UI.Control );

function RadioButtonGroup:Constructor()
	Turbine.UI.Control.Constructor( self );
	-- we use a Control as the default container for this control so that we can get a border

	self.Choices={}; -- each radio button is a distinct entry in the choices
	self.Rows=1;
	self.Cols=1;
	self.IconWidth=20;
	self.IconHeight=20;
	self.Font=Turbine.UI.Lotro.Font.Verdana16;
	self.CurrentSelection=0;
	self.UnselectedIcon=0x41000194; -- built in radio button "unselected" icon
	self.SelectedIcon=0x41000193; -- built in radio button "selected" icon
	self.TextColor=Turbine.UI.Color(1,1,1);
	self.BackColor=Turbine.UI.Color(0,0,0);

	self.RowHeight=20; -- the default row height

	Turbine.UI.Control.SetBackColor(self,Turbine.UI.Color(0,0,0));
	self.SelectionChanged=function()
	end
	self.UpdateSelection=function()
		-- clear the icon on all controls except the selected one
		local tmpIndex;
		for tmpIndex=1,#self.Choices do
			if tmpIndex==self.CurrentSelection then
				self.Choices[tmpIndex].Icon:SetBackground(self.SelectedIcon);
			else
				self.Choices[tmpIndex].Icon:SetBackground(self.UnselectedIcon);
			end
		end
		self:SelectionChanged(); -- call the assigned user callback
	end
end

function RadioButtonGroup:SetBorderColor( color )
	Turbine.UI.Control.SetBackColor(self,color);
end

function RadioButtonGroup:SetBackColor( color )
	self.BackColor=color;
	local tmpIndex;
	for tmpIndex=1,#self.Choices do
		self.Choices[tmpIndex]:SetBackColor(color);
		self.Choices[tmpIndex].Caption:SetBackColor(color);
	end
end

function RadioButtonGroup:SetTextColor( color )
	self.TextColor=color;
	local tmpIndex;
	for tmpIndex=1,#self.Choices do
		self.Choices[tmpIndex].Caption:SetForeColor(color);
	end
end

function RadioButtonGroup:AddChoice(caption, value, index)
	local tmpChoice = Turbine.UI.Control();
	tmpChoice:SetParent(self)
	tmpChoice:SetSize(100,self.RowHeight);
	tmpChoice.Value=value;
	tmpChoice.Icon=Turbine.UI.Control();
	tmpChoice.Icon:SetParent(tmpChoice);
	tmpChoice.Icon:SetSize(self.IconWidth,self.IconHeight);
	tmpChoice.Icon:SetPosition(1,0);
	tmpChoice.Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
	tmpChoice.Icon:SetBackground(self.SelectedIcon);
	tmpChoice.Icon:SetMouseVisible(false);
	tmpChoice.Caption=Turbine.UI.Label();
	tmpChoice.Caption:SetParent(tmpChoice);
	tmpChoice.Caption:SetSize(97-self.IconWidth,self.RowHeight);
	tmpChoice.Caption:SetPosition(self.IconWidth+3,0);
	tmpChoice.Caption:SetFont(self.Font);
	tmpChoice.Caption:SetTextAlignment(Turbine.UI.ContentAlignment.MiddleLeft);
	tmpChoice.Caption:SetForeColor(self.TextColor);
	tmpChoice.Caption:SetBackColor(self.BackColor);
	tmpChoice.Caption:SetFont(self.Font)
	tmpChoice.Caption:SetText(caption);
	tmpChoice.Caption:SetMouseVisible(false);
	tmpChoice.Index=index;
	tmpChoice.MouseClick=function(sender,args)
		if sender:GetParent().CurrentSelection~=sender.Index then
			sender:GetParent().CurrentSelection=sender.Index;
			sender:GetParent():UpdateSelection();
		end
	end
	if index==nil then index=#self.Choices+1 end
	index=tonumber(index)
	if index<1 then index=1 end
	if index>(#self.Choices+1) then	index=#self.Choices+1 end
	table.insert(self.Choices, index, tmpChoice)

	local tmpIndex;
	for tmpIndex=1,#self.Choices do
		self.Choices[tmpIndex].Index=tmpIndex
	end

	if self.CurrentSelection>=index then self.CurrentSelection=self.CurrentSelection+1 end

	-- if the added choice exceeds the number of rows and columns, add a column and re-layout
	if #self.Choices>self.Rows*self.Cols then
		self:SetCols(self.Cols+1);
		-- setCols already calls Layout, so no need for that here
	else
		self:Layout();
	end
end

function RadioButtonGroup:GetValue()
	local retVal=nil;
	if self.CurrentSelection~=nil then
		retVal=self.Choices[self.CurrentSelection].Value;
	end
	return retVal;
end

function RadioButtonGroup:RemoveChoice(index)
	if index==nil then index=#self.Choices end
	index=tonumber(index);
	if index>0 and index<=#self.Choices then
		self.Choices[index].MouseClick=nil;
		table.remove(self.Choices,index)
	end
	if self.CurrentSelection==index then self.CurrentSelection=0 end
	if self.CurrentSelection>index then self.CurrentSelection=self.CurrentSelection-1 end

	local tmpIndex;
	for tmpIndex=1,#self.Choices do
		self.Choices[tmpIndex].Index=tmpIndex
	end

	self:Layout();
end

function RadioButtonGroup:SetFont(font)
	local tmpIndex;
	self.Font=font;
	for tmpIndex=1,#self.Choices do
		self.Choices[tmpIndex].Caption:SetFont(font)
		self.Choices[tmpIndex].Caption:SetText(self.Choices[tmpIndex].Caption:GetText())
	end
	self:Layout();
end
function RadioButtonGroup:GetSelectedChoice()
	return (self.CurrentSelection);
end

function RadioButtonGroup:SetSelectedChoice(index)
	local oldChoice=self.CurrentSelection;
	self.CurrentSelection=index;
	if oldChoice~=index then self:UpdateSelection() end
end

function RadioButtonGroup:SetRows(newRows)
	newRows=tonumber(newRows);
	if newRows<1 then newRows=1 end
	self.Rows=newRows;
	self.Cols=#self.Choices/self.Rows;
	if self.Cols~=math.floor(self.Cols) then
		self.Cols=math.floor(self.Cols)+1; -- we had a fractional component so we have to round up
	end
	self:Layout();
end

function RadioButtonGroup:SetCols(newCols)
	newRows=tonumber(newCols);
	if newCols<1 then newCols=1 end
	self.Cols=newCols;
	self.Rows=#self.Choices/self.Cols;
	if self.Rows~=math.floor(self.Rows) then
		self.Rows=math.floor(self.Rows)+1; -- we had a fractional component so we have to round up
	end
	self:Layout();
end

function RadioButtonGroup:Layout()
	-- adjust the width of the controls based on the new # of columns
	local tmpIndex;
	local tmpColWidth=math.floor((self:GetWidth()-2)/self.Cols);
	local tmpRowHeight=math.floor((self:GetHeight()-2)/self.Rows);
	if tmpRowHeight<self.RowHeight then tmpRowHeight=self.RowHeight end -- we can never decrease the height of the controls :(
	local tmpRow=0;
	local tmpCol=0;

	for tmpIndex=1,#self.Choices do

		self.Choices[tmpIndex]:SetSize(tmpColWidth,tmpRowHeight)
		self.Choices[tmpIndex].Icon:SetSize(self.IconWidth,self.IconHeight)
		self.Choices[tmpIndex].Caption:SetLeft(self.IconWidth+2)
		self.Choices[tmpIndex].Caption:SetWidth(tmpColWidth-self.IconWidth-2)
		self.Choices[tmpIndex].Icon:SetTop((tmpRowHeight-self.IconHeight)/2)
		self.Choices[tmpIndex].Caption:SetHeight(tmpRowHeight)
		self.Choices[tmpIndex]:SetBackColor(self.BackColor)
		self.Choices[tmpIndex]:SetPosition(tmpColWidth*tmpCol+1,tmpRowHeight*tmpRow+1)
		tmpCol=tmpCol+1;
		if tmpCol>=self.Cols then
			tmpRow=tmpRow+1;
			tmpCol=0;
		end
	end

	-- just to be safe, we clear the icon on all controls except the selected one (this will also fix the icon path if the user modified it after adding a Choice)
	local tmpIndex;
	for tmpIndex=1,#self.Choices do
		if tmpIndex==self.CurrentSelection then
			self.Choices[tmpIndex].Icon:SetBackground(self.SelectedIcon);
		else
			self.Choices[tmpIndex].Icon:SetBackground(self.UnselectedIcon);
		end
	end
end
