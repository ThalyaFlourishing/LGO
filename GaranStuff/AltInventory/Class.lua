-- Class.lua
-- Compatible with Lua 5.1 (not 5.0).
--
-- Modified to use a metatable to reference base objects instead
-- or performing shallow copies. This means smaller classes that
-- are a bit more dynamic at the expence of some performance.

-- NOTE, this class definition is ONLY defined for the current environment, NOT the _G environment so it is totally safe to use with other plugins, even those that depend on older Turbine samples
class = function( base )
	if ( base ~= nil ) then
		local baseType = type( base );

		if ( baseType ~= 'table' ) then
			error( "Base class is not a table. Base class was a " .. baseType );
		end
	end

	-- The class definition table. Contains all of the actual class
	-- methods, constructor, and an IsA function automatically
	-- provided.
	local c  = { };

	c.base = base;
	-- NOTE: The IsA will only compare positively for classes directly derived from system classes, NOT user defined classes
	-- the version of Class.lua in the Turbine samples works slightly better for user defined classes
	c.IsA = function( self, klass )
		local m = getmetatable( self );
		
		while m do 
			if m == klass then
				return true;
			end
			
			m = m.base;
		end
		
		return false;
	end

	-- The class definition metatable. This is used to hook up
	-- base classes and then to register a call handler that is
	-- used to construct new instances of the class.
	local mt = { };
	mt.__index = base;

	mt.__call = function( class_tbl, ... )
		if ( class_tbl.Abstract ) then
			error "Abstract class cannot be constructed.";
		end

		-- Create the new class instance.
		local instance = { };
		setmetatable( instance, { __index = c } );
		
		-- Invoke the constructor if it exists.
		if ( ( instance.Constructor ~= nil ) and ( type( instance.Constructor ) == 'function' ) ) then
			instance:Constructor( ... );
		end
		
		return instance;
	end

	setmetatable( c, mt );
	return c;
end
