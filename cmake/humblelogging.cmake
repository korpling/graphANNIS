##################
# Humble logging #
##################

ExternalProject_Add(
  HumbleLogging

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/humblelogging-3.0.1"
  CMAKE_ARGS -DBuildShared=OFF -DBuildExamples=OFF -DCMAKE_POSITION_INDEPENDENT_CODE=True -DCMAKE_INSTALL_PREFIX=${GLOBAL_OUTPUT_PATH}/humblelogging

  TEST_COMMAND ""
)


ExternalProject_Add_Step(
  HumbleLogging CopyToBin
  COMMAND ${CMAKE_COMMAND} -E copy_directory ${GLOBAL_OUTPUT_PATH}/humblelogging/bin ${GLOBAL_OUTPUT_PATH}
  COMMAND ${CMAKE_COMMAND} -E copy_directory ${GLOBAL_OUTPUT_PATH}/humblelogging/lib ${GLOBAL_OUTPUT_PATH}
  DEPENDEES install
)

set(HumbleLogging_INCLUDE_DIRS "${GLOBAL_OUTPUT_PATH}/humblelogging/include")
set(HumbleLogging_LIBRARIES "${CMAKE_SHARED_LIBRARY_PREFIX}humblelogging${CMAKE_SHARED_LIBRARY_SUFFIX}")
include_directories(${HumbleLogging_INCLUDE_DIRS})
