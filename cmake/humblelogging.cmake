##################
# Humble logging #
##################

set(HumbleLogging_PREFIX ${GLOBAL_OUTPUT_PATH}/humblelogging)

ExternalProject_Add(
  HumbleLogging

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/humblelogging-3.0.1"
  CMAKE_ARGS -DBuildShared=OFF -DBuildExamples=OFF -DCMAKE_POSITION_INDEPENDENT_CODE=True -DCMAKE_INSTALL_PREFIX=${HumbleLogging_PREFIX}

  TEST_COMMAND ""
)

set(HumbleLogging_INCLUDE_DIRS "${HumbleLogging_PREFIX}/include")
set(HumbleLogging_LIBRARIES "${HumbleLogging_PREFIX}/lib/${CMAKE_SHARED_LIBRARY_PREFIX}humblelogging${CMAKE_STATIC_LIBRARY_SUFFIX}")
include_directories(SYSTEM ${HumbleLogging_INCLUDE_DIRS})
