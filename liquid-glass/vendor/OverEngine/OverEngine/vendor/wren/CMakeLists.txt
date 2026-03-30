file(GLOB_RECURSE wren_vm_source_files "src/vm/*.c")
file(GLOB_RECURSE wren_vm_header_files "src/vm/*.h")

file(GLOB_RECURSE wren_optional_source_files "src/optional/*.c")
file(GLOB_RECURSE wren_optional_header_files "src/optional/*.h")
file(GLOB_RECURSE wren_optional_inc_files "src/optional/*.inc")

add_library(wren
	${wren_vm_source_files}
	${wren_vm_header_files}
	"src/vm/wren_core.wren.inc"

	${wren_optional_source_files}
	${wren_optional_header_files}
	${wren_optional_inc_files}

	"src/include/wren.h"
)

target_include_directories(wren PRIVATE
	"src/include"
	"src/optional"
	"src/vm"
)

