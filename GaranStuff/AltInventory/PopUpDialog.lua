local importPath=string.gsub(getfenv(1)._.Name,"%.PopUpDialog","").."."
local resourcePath=string.gsub(importPath,"%.","/").."Resources/"

if plugin~=nil and plugin.GetName~=nil then
	PluginName=plugin:GetName();
end

PopUpDialog = class(Turbine.UI.Lotro.Window);
function PopUpDialog:Constructor(Title,Message,Icon,Button1,Button2,Button3,ShowPin,Callback1,Callback2,Callback3)
	Turbine.UI.Lotro.Window.Constructor( self );
	local charWidth=8.3 -- rough estimate, but it'll have to do
	self:SetText(Title);
	self:SetVisible(true);
	local titleWidth=string.len(Title)*charWidth;
	local messageWidth;
	local messageTotal=string.len(Message)*charWidth;
	maxWidth=math.floor(Turbine.UI.Display:GetWidth()*.75);
	if fontColor==nil then fontColor=Turbine.UI.Color(1,1,1) end
	if backColor==nil then backColor=Turbine.UI.Color(0,0,0) end
	-- try to find width that will provide roughly 8/3 ratio width/height
	messageWidth=math.floor(messageTotal/(((messageTotal*45/8)^0.5)/15))+20; -- we add 20 to accomodate the text field's border
	if messageWidth>maxWidth then messageWidth=maxWidth end;
	if messageWidth<titleWidth+140 then messageWidth=titleWidth+140 end
	if messageWidth<300 then messageWidth=300 end;

-- determine the height via the scrollbar...
--	messageHeight=(math.floor(messageTotal/messageWidth)+1)*15+5;

	-- need to add 15 to the message height for each CR character
--	local newLine;
--	for newLine in string.gmatch(Message, "\n") do
--		messageHeight=messageHeight+15;
--	end
--	messageHeight=messageHeight+35; -- we add 35 to accomodate the text field's border and a spare CR

--	if messageHeight<35 then messageHeight=35 end; -- force minimum height
	
	self.Loading=true;
	self.isPinned=false
	self.IsPinned=function()
		return self.isPinned
	end
	if ShowPin==nil then ShowPin=false end
	self.PinButton=Turbine.UI.Control()
	self.PinButton:SetParent(self)
	self.PinButton:SetSize(16,16)
	self.PinButton:SetPosition(5,21)
	self.PinButton:SetBlendMode(Turbine.UI.BlendMode.Overlay)
	self.PinButton:SetBackground(resourcePath.."tack_out.tga")
	self.PinButton:SetVisible(ShowPin)
	self.PinButton.MouseClick=function()
		self.isPinned=not self.isPinned
		if self.isPinned then
			self.PinButton:SetBackground(resourcePath.."tack_in.tga")
		else
			self.PinButton:SetBackground(resourcePath.."tack_out.tga")
		end
	end
	self.SetPinned=function(sender,args)
		self.isPinned=args
		if self.isPinned then
			self.PinButton:SetBackground(resourcePath.."tack_in.tga")
		else
			self.PinButton:SetBackground(resourcePath.."tack_out.tga")
		end
	end

	self.Message=Turbine.UI.Lotro.TextBox();
	self.Message:SetReadOnly(true);
	self.Message:SetParent(self);
	self.Message:SetFont(Turbine.UI.Lotro.Font.Verdana16);
--self.Message:SetBackColor(Turbine.UI.Color(0,.8,1));
	self.Message:SetForeColor(fontColor);
	self.Message:SetSize(messageWidth,32);
	self.Message:SetSelectable(true);
	self.Message:SetMarkupEnabled(true);
	self.Message:SetTextAlignment( Turbine.UI.ContentAlignment.MiddleCenter);
	self.Message:SetText(Message);
	self.Message.VScroll=Turbine.UI.Lotro.ScrollBar();
	self.Message.VScroll:SetOrientation(Turbine.UI.Orientation.Vertical);
	self.Message.VScroll:SetParent(self);
	self.Message.VScroll:SetBackColor(backColor);
	self.Message.VScroll:SetPosition(self.Message:GetLeft()+self.Message:GetWidth(),self.Message:GetTop())
	self.Message.VScroll:SetWidth(12);
	self.Message:SetVerticalScrollBar(self.Message.VScroll);
	
	-- resize message based on scrollbar and max height
	local height=self.Message:GetHeight();
	local maxHeight=(Turbine.UI.Display:GetHeight()*.75);
	while (self.Message.VScroll:IsVisible() and height<maxHeight) do
		height=height+1;
		self.Message:SetHeight(height);
		self.Message:SetText(Message) -- for some weird reason, the scrollbar visibility doesn't get updated when we are "loading" unless we reset the text. bizarre.
	end
	if Icon==nil then
		self:SetSize(self.Message:GetWidth()+32,self.Message:GetHeight()+75);
		self.Message:SetPosition(10,40);
	else
		self.Icon=Turbine.UI.Control();
		self.Icon:SetParent(self);
		self.Icon:SetSize(32,32);
		self.Icon:SetPosition(10,40);
		self.Icon:SetBlendMode(Turbine.UI.BlendMode.Overlay);
		if Icon==1 then
			self.Icon:SetBackground(resourcePath.."Exclamation.tga");
		elseif Icon==2 then
			self.Icon:SetBackground(resourcePath.."Question.tga");
		elseif Icon==3 then
			self.Icon:SetBackground(resourcePath.."Critical.tga");
		else
			-- default to Information
			self.Icon:SetBackground(resourcePath.."Information.tga");
		end
		self:SetSize(self.Message:GetWidth()+72,self.Message:GetHeight()+75);
		self.Message:SetPosition(50,40);
	end
	self:SetPosition((Turbine.UI.Display:GetWidth() - self:GetWidth()) / 2,(Turbine.UI.Display:GetHeight() - self:GetHeight()) / 2);
	self.Message.VScroll:SetHeight(self.Message:GetHeight());

	self.Callback1=Callback1
	self.Callback2=Callback2
	self.Callback3=Callback3
-- buttons
	local buttonWidth=50;
	self.Buttons={}
	if Button1~=nil then
		local buttonIndex=#self.Buttons+1;
		self.Buttons[buttonIndex]=Turbine.UI.Lotro.Button();
		self.Buttons[buttonIndex]:SetParent(self);
		self.Buttons[buttonIndex]:SetSize(100,20);
		self.Buttons[buttonIndex].Value=1;
		self.Buttons[buttonIndex]:SetTop(self:GetHeight()-30);
		self.Buttons[buttonIndex]:SetText(Button1);
		self.Buttons[buttonIndex].Click=function()
			-- return 1
			if self.Callback1~=nil then
				self.Callback1()
			end
			self:SetVisible(false);
		end
		if string.len(Button1)*charWidth>buttonWidth then buttonWidth=string.len(Button1)*charWidth end
	end
	if Button2~=nil then
		local buttonIndex=#self.Buttons+1;
		self.Buttons[buttonIndex]=Turbine.UI.Lotro.Button();
		self.Buttons[buttonIndex]:SetParent(self);
		self.Buttons[buttonIndex]:SetSize(100,20);
		self.Buttons[buttonIndex].Value=2;
		self.Buttons[buttonIndex]:SetTop(self:GetHeight()-30);
		self.Buttons[buttonIndex]:SetText(Button2);
		self.Buttons[buttonIndex].Click=function()
			-- return 2
			if self.Callback2~=nil then
				self.Callback2()
			end
			self:SetVisible(false);
		end
		if string.len(Button2)*charWidth>buttonWidth then buttonWidth=string.len(Button2)*charWidth end
	end
	if Button3~=nil then
		local buttonIndex=#self.Buttons+1;
		self.Buttons[buttonIndex]=Turbine.UI.Lotro.Button();
		self.Buttons[buttonIndex]:SetParent(self);
		self.Buttons[buttonIndex]:SetSize(100,20);
		self.Buttons[buttonIndex].Value=3;
		self.Buttons[buttonIndex]:SetTop(self:GetHeight()-30);
		self.Buttons[buttonIndex]:SetText(Button3);
		self.Buttons[buttonIndex].Click=function()
			-- return 3
			if self.Callback3~=nil then
				self.Callback3()
			end
			self:SetVisible(false);
		end
		if string.len(Button3)*charWidth>buttonWidth then buttonWidth=string.len(Button3)*charWidth end
	end
	for index=1, #self.Buttons do
		self.Buttons[index]:SetWidth(buttonWidth);
	end
	-- make sure dialog is wide enough for all of its buttons
	if #self.Buttons*buttonWidth+60>self:GetWidth() then
		self:SetWidth(#self.Buttons*(buttonWidth+10)+40)
		if Icon==nil then
			self.Message:SetWidth(self:GetWidth()-20);
		else
			self.Message:SetWidth(self:GetWidth()-60);
		end
	end

	if #self.Buttons==0 then
		-- throw an error and exit. we can't display a dialog with no buttons
	elseif #self.Buttons==1 then
		self.Buttons[1]:SetLeft((self:GetWidth()-self.Buttons[1]:GetWidth())/2);
	elseif #self.Buttons==2 then
		self.Buttons[1]:SetLeft((self:GetWidth()-self.Buttons[1]:GetWidth()*2)/3);
		self.Buttons[2]:SetLeft(self:GetWidth()-self.Buttons[1]:GetWidth()-self.Buttons[1]:GetLeft());
	else
		self.Buttons[1]:SetLeft(25);
		self.Buttons[2]:SetLeft((self:GetWidth()-self.Buttons[1]:GetWidth())/2);
		self.Buttons[3]:SetLeft(self:GetWidth()-self.Buttons[3]:GetWidth()-25);
	end
	self.Update=function()
		if self.Loading then
			self:SetWantsUpdates(false);
			self.Loading=false;
			self:SetZOrder(self:GetZOrder()+1);
		end
	end
	self:SetWantsUpdates(true);
end
