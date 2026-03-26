import "Turbine.UI";
import "Turbine.UI.Lotro";
ColorPicker = class( Turbine.UI.Control );

function ColorPicker:Constructor()
	Turbine.UI.Label.Constructor( self );
	-- we use a Label as the default container for this control so that we can get a border
	self.Border=Turbine.UI.Label();
	self:SetSize(276,20);
	self.Border:SetParent(self);
	self.Border:SetPosition(0,0);
	self.Border:SetSize(self:GetWidth(),self:GetHeight());
	self.BackPanel=Turbine.UI.Label();
	self.BackPanel:SetParent(self.Border);
	self.BackPanel:SetPosition(1,1);
	self.BackPanel:SetSize(self:GetWidth()-2,self:GetHeight()-2);

	self.ColorSwatch=Turbine.UI.Label();
	self.ColorSwatch:SetParent(self);
	self.ColorSwatch:SetSize(18,18);
	self.ColorSwatch:SetPosition(self:GetWidth()-19,1);
	self.ColorSwatch.MouseHover=function(sender,args)
		local color=self.ColorSwatch:GetBackColor();
		local parent=sender:GetParent();
		parent.ColorMenu:GetItems():Get(1):SetText("Red:"..string.format("%x",math.floor(color.R*255)));
		parent.ColorMenu:GetItems():Get(2):SetText("Green:"..string.format("%x",math.floor(color.G*255)));
		parent.ColorMenu:GetItems():Get(3):SetText("Blue:"..string.format("%x",math.floor(color.B*255)));
		parent.ColorMenu:ShowMenuAt(Turbine.UI.Display:GetMouseX(),Turbine.UI.Display:GetMouseY()+20);
	end
	self.ColorSwatch.MouseLeave=function()
		self.ColorMenu:Close();
	end
	self.ColorMenu=Turbine.UI.ContextMenu()
	self.ColorMenu:GetItems():Add(Turbine.UI.MenuItem("Red"));
	self.ColorMenu:GetItems():Add(Turbine.UI.MenuItem("Green"));
	self.ColorMenu:GetItems():Add(Turbine.UI.MenuItem("Blue"));

	self.RedSlider=Turbine.UI.Lotro.ScrollBar();
	self.RedSlider:SetOrientation(Turbine.UI.Orientation.Horizontal);
	self.RedSlider:SetParent(self)
	self.RedSlider:SetPosition(1,1);
	self.RedSlider:SetSize(self:GetWidth()-21,6);
	self.RedSlider:SetBackColor(Turbine.UI.Color(1,0,0));
	self.RedSlider:SetMinimum(0);
	self.RedSlider:SetMaximum(255);
	self.RedSlider.ValueChanged=function()
		local color=self.ColorSwatch:GetBackColor();
		color.R=self.RedSlider:GetValue()/self.RedSlider:GetMaximum()
		self.ColorSwatch:SetBackColor(color);
		self:ColorChanged();
	end

	self.GreenSlider=Turbine.UI.Lotro.ScrollBar();
	self.GreenSlider:SetOrientation(Turbine.UI.Orientation.Horizontal);
	self.GreenSlider:SetParent(self)
	self.GreenSlider:SetPosition(1,7);
	self.GreenSlider:SetSize(self:GetWidth()-21,6);
	self.GreenSlider:SetBackColor(Turbine.UI.Color(0,1,0));
	self.GreenSlider:SetMinimum(0);
	self.GreenSlider:SetMaximum(255);
	self.GreenSlider.ValueChanged=function()
		local color=self.ColorSwatch:GetBackColor();
		color.G=self.GreenSlider:GetValue()/self.GreenSlider:GetMaximum()
		self.ColorSwatch:SetBackColor(color);
		self:ColorChanged();
	end

	self.BlueSlider=Turbine.UI.Lotro.ScrollBar();
	self.BlueSlider:SetOrientation(Turbine.UI.Orientation.Horizontal);
	self.BlueSlider:SetParent(self)
	self.BlueSlider:SetPosition(1,13);
	self.BlueSlider:SetSize(self:GetWidth()-21,6);
	self.BlueSlider:SetBackColor(Turbine.UI.Color(0,0,1));
	self.BlueSlider:SetMinimum(0);
	self.BlueSlider:SetMaximum(255);
	self.BlueSlider.ValueChanged=function()
		local color=self.ColorSwatch:GetBackColor();
		color.B=self.BlueSlider:GetValue()/self.BlueSlider:GetMaximum()
		self.ColorSwatch:SetBackColor(color);
		self:ColorChanged();
	end

	self.SetColor=function(sender,color)
		self.ColorSwatch:SetBackColor(color);
		self.RedSlider:SetValue(color.R*255);
		self.GreenSlider:SetValue(color.G*255);
		self.BlueSlider:SetValue(color.B*255);
		self:ColorChanged();
	end
	self.GetColor=function()
		return(self.ColorSwatch:GetBackColor());
	end
	self.SetBackColor=function(sender,color)
		self.BackPanel:SetBackColor(color);
	end
	self.SetTrimColor=function(sender,color)
		self.Border:SetBackColor(color);
	end
	self.SetWidth=function(sender,width)
		Turbine.UI.Label.SetWidth(self,width);
		self.Border:SetWidth(width);
		self.BackPanel:SetWidth(width-2);
		self.RedSlider:SetWidth(width-21);
		self.GreenSlider:SetWidth(width-21);
		self.BlueSlider:SetWidth(width-21);
		self.ColorSwatch:SetLeft(width-19);
	end
	self.SetHeight=function(sender,height)
		Turbine.UI.Label.SetHeight(self,height);
		self.Border:SetHeight(height);
		self.BackPanel:SetHeight(height-2);
		local sliderHeight=(height-2)/3;
		self.RedSlider:SetHeight(sliderHeight);
		self.GreenSlider:SetTop(sliderHeight+1);
		self.GreenSlider:SetHeight(sliderHeight);
		self.BlueSlider:SetTop(sliderHeight*2+1);
		self.BlueSlider:SetHeight(height-2-sliderHeight*2);
		self.ColorSwatch:SetHeight(height-2);
	end
	self.ColorChanged=function()
	end
end