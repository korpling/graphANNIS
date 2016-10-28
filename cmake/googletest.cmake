#######################
# google test library #
#######################

ExternalProject_Add(
  GoogleTest

  UPDATE_COMMAND ""
  PATCH_COMMAND ""

  SOURCE_DIR "${CMAKE_SOURCE_DIR}/ext/gtest-1.7.0"
  CMAKE_ARGS -DCMAKE_POSITION_INDEPENDENT_CODE=True -DBUILD_SHARED_LIBS=OFF

  TEST_COMMAND ""
  INSTALL_COMMAND ""
)

ExternalProject_Get_Property(GoogleTest BINARY_DIR)

set(GoogleTest_LIBRARIES
  "${BINARY_DIR}/${CMAKE_STATIC_LIBRARY_PREFIX}gtest${CMAKE_STATIC_LIBRARY_SUFFIX}"
  "${BINARY_DIR}/${CMAKE_STATIC_LIBRARY_PREFIX}gtest_main${CMAKE_STATIC_LIBRARY_SUFFIX}"
  )
set(GoogleTest_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/ext/gtest-1.7.0/include/")


include_directories(SYSTEM ${GoogleTest_INCLUDE_DIRS})
