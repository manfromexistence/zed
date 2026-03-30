project "yaml-cpp"
	kind "StaticLib"

	language "C++"
	cppdialect "C++17"

	targetdir ("bin/" .. outputdir .. "/%{prj.name}")
	objdir ("bin-int/" .. outputdir .. "/%{prj.name}")

	files
	{
		"include/**.h",
		"src/**.cpp",
		"src/**.h",
	}

	includedirs
	{
		"include"
	}

	defines
	{
		"YAML_CPP_STATIC_DEFINE"
	}

	filter "system:linux"
		pic "on"
		systemversion "latest"
		staticruntime "on"

	filter "system:windows"
		systemversion "latest"
		staticruntime (staticRuntime)

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
