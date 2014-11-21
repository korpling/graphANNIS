project(ANNIS4)
cmake_minimum_required(VERSION 2.8)
set(CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake" "${PROJECT_SOURCE_DIR}/ext/humble-logging-library-2.0.0/cmake")

add_subdirectory(ext/humble-logging-library-2.0.0)

find_package(ICU 4.4 REQUIRED)
find_package(Boost REQUIRED COMPONENTS system filesystem serialization)

message(STATUS "HumbleLogging_INCLUDE_DIRS=${HumbleLogging_INCLUDE_DIRS}")
message(STATUS "HumbleLogging_LIBRARY=${HumbleLogging_LIBRARY}")

include_directories(
  SYSTEM
  "include"
  "ext/humble-logging-library-2.0.0/include"
  ${ICU_INCLUDE_DIRS}
  ${Boost_INCLUDE_DIRS}
)

file(GLOB SRC_LIST_LIB "src/lib/*.h" "src/lib/*.cpp"
  "src/lib/edgedb/*.h" "src/lib/edgedb/*.cpp"
  "src/lib/operators/*.h" "src/lib/operators/*.cpp")
file(GLOB SRC_LIST_RUNNER "src/runner/*.h" "src/runner/*.cpp" "src/runner/*.c" )

if(CMAKE_COMPILER_IS_GNUCXX)
  set(CMAKE_CXX_FLAGS "-std=c++11")
endif()

add_library(${PROJECT_NAME} ${SRC_LIST_LIB})
target_link_libraries(${PROJECT_NAME} humblelogging ${Boost_LIBRARIES})


add_executable("a4_runner" ${SRC_LIST_RUNNER})
target_link_libraries("a4_runner" humblelogging ${Boost_LIBRARIES} ${PROJECT_NAME})


file(GLOB BENCHMARK_SRC_LIST "benchmarks/*.cpp" "benchmarks/*.h")
set(BENCHMARK_EXEC "benchmark_${PROJECT_NAME}")
include_directories(SYSTEM "ext/hayai/src/")
include_directories("src/lib/")
add_executable(${BENCHMARK_EXEC}  ${BENCHMARK_SRC_LIST} )
target_link_libraries(${BENCHMARK_EXEC} ${PROJECT_NAME} )

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
