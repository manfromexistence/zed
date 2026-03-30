add_library(imgui
    "imconfig.h"
	"imgui.h"
	"imgui.cpp"
	"imgui_draw.cpp"
	"imgui_internal.h"
    "imgui_widgets.cpp"
    "imgui_tables.cpp"
	"imstb_rectpack.h"
	"imstb_textedit.h"
	"imstb_truetype.h"
    "imgui_demo.cpp"
)

target_include_directories(imgui PRIVATE
    "${PROJECT_SOURCE_DIR}/OverEngine/vendor/stb"
)
