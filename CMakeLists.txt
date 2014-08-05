project(ANNIS4)
cmake_minimum_required(VERSION 2.8)

include_directories("include")
file(GLOB SRC_LIST_LIB "src/lib/*.h" "src/lib/*.cpp" )
file(GLOB SRC_LIST_RUNNER "src/runner/*.h" "src/runner/*.cpp" )

if(CMAKE_COMPILER_IS_GNUCXX)
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")
endif()

add_library(${PROJECT_NAME} ${SRC_LIST_LIB})
add_executable("a4_runner" ${SRC_LIST_RUNNER})
target_link_libraries("a4_runner" ${PROJECT_NAME})

################################
# Testing with Google Test
################################
add_subdirectory(ext/gtest-1.7.0)
enable_testing()
include_directories(${gtest_SOURCE_DIR}/include ${gtest_SOURCE_DIR} "src/lib/")

file(GLOB TEST_SRC_LIST "tests/*.cpp" "tests/*.h")

set(TEST_EXEC "test_${PROJECT_NAME}")
add_executable(${TEST_EXEC}  ${TEST_SRC_LIST} )
target_link_libraries(${TEST_EXEC} ${PROJECT_NAME} gtest gtest_main)
add_test( ${TEST_EXEC} ${TEST_EXEC} )

