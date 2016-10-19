#####################
# RE2 regex library #
#####################

ExternalProject_Add(
  RE2

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/re2-2016-10-01"
  CMAKE_ARGS -BUILD_SHARED_LIBS=OFF -DCMAKE_POSITION_INDEPENDENT_CODE=True -DCMAKE_INSTALL_PREFIX=${GLOBAL_OUTPUT_PATH}/re2

  TEST_COMMAND ""
)


set(RE2_LIBRARIES "${CMAKE_STATIC_LIBRARY_PREFIX}re2${CMAKE_STATIC_LIBRARY_SUFFIX}")
set(RE2_INCLUDE_DIRS "${GLOBAL_OUTPUT_PATH}/re2/include/")

ExternalProject_Get_Property(RE2 BINARY_DIR)

ExternalProject_Add_Step(
  RE2 CopyToBin
  COMMAND ${CMAKE_COMMAND} -E copy_directory ${GLOBAL_OUTPUT_PATH}/re2/lib ${GLOBAL_OUTPUT_PATH}
  DEPENDEES install
)



include_directories(${RE2_INCLUDE_DIRS})
