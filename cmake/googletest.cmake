#######################
# google test library #
#######################

ExternalProject_Add(
  GoogleTest

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/gtest-1.7.0"
  CMAKE_ARGS -DBUILD_SHARED_LIBS=OFF

  TEST_COMMAND ""
  INSTALL_COMMAND ""
)


set(GoogleTest_LIBRARIES "${CMAKE_STATIC_LIBRARY_PREFIX}gtest${CMAKE_STATIC_LIBRARY_SUFFIX}"
  "${CMAKE_STATIC_LIBRARY_PREFIX}gtest-main${CMAKE_STATIC_LIBRARY_SUFFIX}"
  )
set(GoogleTest_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/ext/gtest-1.7.0/include/")

ExternalProject_Get_Property(GoogleTest BINARY_DIR)

ExternalProject_Add_Step(
  GoogleTest CopyToBin
  COMMAND ${CMAKE_COMMAND} -E copy_directory ${BINARY_DIR} ${GLOBAL_OUTPUT_PATH}
  DEPENDEES install
)


include_directories(SYSTEM ${GoogleTest_INCLUDE_DIRS})
