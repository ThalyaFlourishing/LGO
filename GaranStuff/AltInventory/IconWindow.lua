iconWindow=Turbine.UI.Window()
if Settings.useMiniIcon then
	iconWindow.UpImage="GaranStuff/AltInventory/Resources/mini_icon.tga"
	iconWindow.HoverImage="GaranStuff/AltInventory/Resources/mini_hover.tga"
	iconWindow.MoveImage="GaranStuff/AltInventory/Resources/mini_drag.tga"
	iconWindow.FlashImage="GaranStuff/AltInventory/Resources/mini_border.tga"
else
	iconWindow.UpImage="GaranStuff/AltInventory/Resources/bags_up.jpg"
	iconWindow.HoverImage="GaranStuff/AltInventory/Resources/bags_down.jpg"
	iconWindow.MoveImage="GaranStuff/AltInventory/Resources/bags_drag.jpg"
	iconWindow.FlashImage="GaranStuff/AltInventory/Resources/bags_border.tga"
end
iconWindow:SetBackground(iconWindow.UpImage);
iconWindow:SetSize(32,32);
iconWindow:SetOpacity(.3);
iconWindow:SetVisible((Settings.loadMinimized and Settings.showIcon==1) or Settings.showIcon==2);
function initIcon()
	local left=Settings.iconLeft;
	if left==nil then
		left=0;
	else
		left=euroNormalize(left);
	end
	local top=Settings.iconTop;
	if top==nil then
		top=1-iconWindow:GetHeight()/displayHeight;
	else
		top=euroNormalize(top);
	end
	if left<0 then left=0 end
	if left*displayWidth+iconWindow:GetWidth()>displayWidth then
		left=1-iconWindow:GetWidth()/displayWidth
	end
	if top<0 then top=0 end
	if top*displayHeight+iconWindow:GetHeight()>displayHeight then top=1-iconWindow:GetHeight()/displayHeight end

	iconWindow:SetPosition(left*displayWidth, top*displayHeight);
end
initIcon();
iconWindow.MouseClick = function(sender,args)
	iconWindow:SetBackground(iconWindow.UpImage);
	if iconWindow.Moving then
		iconWindow.Moving=false;
		iconWindow.MoveX=-1;
	else
		if Settings.useMinimalHeader then
			if minimalWindow:IsVisible() then
				minimalWindow:SetVisible(false)
			else
				if Settings.showIcon~=2 then
					iconWindow:SetVisible(false)
				end
				minimalWindow:SetVisible(true)
			end
		else
			if inventoryWindow:IsVisible() then
				inventoryWindow:SetVisible(false)
			else
				if Settings.showIcon~=2 then
					iconWindow:SetVisible(false)
				end
				inventoryWindow:SetVisible(true)
			end
		end
	end
end

iconWindow.MoveX=-1;
iconWindow.MoveY=0;
iconWindow.Moving=false;

iconWindow.MouseDown=function(sender,args)
	if iconWindow:IsControlKeyDown() or iconWindow:IsShiftKeyDown() or iconWindow:IsAltKeyDown() then
		iconWindow:SetBackground(iconWindow.HoverImage);
		iconWindow.MoveX=args.X;
		iconWindow.MoveY=args.Y;
	end
end
iconWindow.MouseUp=function(sender,args)
	iconWindow:SetBackground(iconWindow.UpImage);
	iconWindow.MoveX=-1;
	iconWindow.MoveY=0;
end

iconWindow.MouseEnter=function(sender,args)
	iconWindow:SetOpacity(1);
end

iconWindow.MouseLeave=function(sender,args)
	iconWindow:SetOpacity(.3);
end

iconWindow.MouseMove=function(sender,args)
	if iconWindow.MoveX~=-1 and (args.X~=iconWindow.MoveX or args.Y~= iconWindow.MoveY) then
		iconWindow:SetBackground(iconWindow.MoveImage);
		iconWindow.Moving=true;
		local newLeft=iconWindow:GetLeft()-(iconWindow.MoveX-args.X)
		local newTop=iconWindow:GetTop()-(iconWindow.MoveY-args.Y)
		if newLeft<0 then newLeft=0 end;
		if newLeft>(Turbine.UI.Display.GetWidth()-iconWindow:GetWidth()) then newLeft=Turbine.UI.Display.GetWidth()-iconWindow:GetWidth() end;
		if newTop<0 then newTop=0 end;
		if newTop>(Turbine.UI.Display.GetHeight()-iconWindow:GetHeight()) then newTop=Turbine.UI.Display.GetHeight()-iconWindow:GetHeight() end;
		iconWindow:SetPosition(newLeft,newTop);
		if setupWindow~=nil and setupWindow:IsVisible() then
			setupWindow.iconLeft:SetText(newLeft);
			setupWindow.iconTop:SetText(newTop);
		end
	end
end

iconWindow.FlashBackground=Turbine.UI.Control();
iconWindow.FlashBackground:SetParent(iconWindow);
iconWindow.FlashBackground:SetMouseVisible(false);
iconWindow.FlashBackground:SetVisible(false);
iconWindow.FlashBackground:SetSize(32,32);
iconWindow.FlashBackground:SetBlendMode(Turbine.UI.BlendMode.Overlay);
iconWindow.FlashBackground:SetBackground(iconWindow.FlashImage);
iconWindow.FlashBackground:SetBackColorBlendMode(Turbine.UI.BlendMode.Color);
iconWindow.Flashing=false;
iconWindow.FlashCount=0;
iconWindow.Flash=function(sender,color)
	iconWindow:SetWantsUpdates(true);
	iconWindow.Flashing=true;
	iconWindow.FlashCount=0;
	iconWindow.FlashBackground:SetBackColor(color);
	iconWindow.FlashBackground:SetVisible(true);
	iconWindow.FlashState=1;
	iconWindow.FlashTimer=Turbine.Engine:GetGameTime()+.2;
	iconWindow:SetOpacity(1)
	iconWindow:SetVisible(true);
end
iconWindow.Update=function(sender,args)
	if iconWindow.Flashing then
		if Turbine.Engine:GetGameTime()>iconWindow.FlashTimer then
			if iconWindow.FlashState==0 then
				iconWindow.FlashBackground:SetVisible(true);
				iconWindow.FlashState=1;
				iconWindow.FlashTimer=Turbine.Engine:GetGameTime()+.2;
			else
				iconWindow.FlashBackground:SetVisible(false);
				iconWindow.FlashCount=iconWindow.FlashCount+1;
				if iconWindow.FlashCount>3 then
					iconWindow.Flashing=false;
					iconWindow:SetOpacity(.3);
					iconWindow:SetWantsUpdates(false);
					if inventoryWindow:IsVisible() or minimalWindow:IsVisible() then
						iconWindow:SetVisible(false);
					end
				else
					iconWindow.FlashState=0;
					iconWindow.FlashTimer=Turbine.Engine:GetGameTime()+.2;
				end
			end
		end
	else
		iconWindow.SetWantsUpdates(false); -- turn updates off... no need to waste machine cycles on this
	end
end
