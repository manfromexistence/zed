project "wren"
	kind "StaticLib"
	language "C"
	cdialect "C99"

	targetdir ("bin/" .. outputdir .. "/%{prj.name}")
	objdir ("bin-int/" .. outputdir .. "/%{prj.name}")

	files
	{
		"src/**.h",
		"src/**.c"
	}

	includedirs
	{
		"src/include",
		"src/vm",
		"src/optional"
	}

	filter "system:windows"
		systemversion "latest"
		staticruntime (staticRuntime)

	filter "system:linux"
		pic "on"
		systemversion "latest"
		staticruntime "on"
		links { "m" }

		filter "configurations:Debug"
		runtime "Debug"
		symbols "on"

	filter "configurations:DebugOptimized"
		runtime "Debug"
		symbols "on"
		optimize "on"

	filter "configurations:Release"
		runtime "Release"
		optimize "on"

