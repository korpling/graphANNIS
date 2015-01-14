find_path(RE2_INCLUDE_DIRS NAMES re2/re2.h HINTS /usr/include /usr/local/include)
find_library(RE2_LIBRARIES NAMES libre2.a libre2.lib libre2.dylib HINTS /usr/lib /usr/local/lib)

set(RE2_FOUND FALSE)
if(RE2_INCLUDE_DIRS AND RE2_LIBRARIES)
  set(RE2_FOUND TRUE)
endif()

if(RE2_FOUND)
  message(STATUS "Found RE2: ${RE2_LIBRARIES}")
else(RE2_FOUND)
  message(FATAL_ERROR "Could not find RE2 library.")
endif()
