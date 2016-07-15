project(ANNIS4)
cmake_minimum_required(VERSION 2.8)
set(CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake")

set(needed_features
  cxx_auto_type
  cxx_override
  cxx_nullptr
  cxx_long_long_type)

set(BuildShared OFF CACHE BOOL "")
add_subdirectory(ext/humblelogging-3.0.1)

set(CELERO_RUN_EXAMPLE_ON_BUILD OFF CACHE BOOL "Set to ON to automatically run the example after a successful build.")
set(CELERO_ENABLE_EXPERIMENTS OFF CACHE BOOL "Set to ON to automatically build all examples.")
set(CELERO_COMPILE_DYNAMIC_LIBRARIES OFF CACHE BOOL "Set to ON to build Celero for dynamic linking.  Use OFF for static.")
add_subdirectory(ext/Celero-2.0.2)


find_package(ICU 4.4 REQUIRED)
find_package(Boost REQUIRED COMPONENTS system filesystem serialization)
find_package(Re2 REQUIRED)
find_package (Threads REQUIRED)

include_directories(
  SYSTEM
  "include"
  "ext/humblelogging-3.0.1/include"
  ${ICU_INCLUDE_DIRS}
  ${Boost_INCLUDE_DIRS}
  ${RE2_INCLUDE_DIR}
)

# make sure QtCreator shows the include files
FILE(GLOB_RECURSE IncludeFiles "include/*.h")
add_custom_target(headers SOURCES ${IncludeFiles})

file(GLOB SRC_LIST_LIB "src/lib/*.h" "src/lib/*.cpp" "src/lib/*.c"
  "src/lib/util/*.cpp" "src/lib/util/*.c"
  "src/lib/annosearch/*.cpp"
  "src/lib/graphstorage/*.cpp"
  "src/lib/operators/*.cpp"
  "src/lib/join/*.cpp"
  "src/lib/json/*.cpp"
)
file(GLOB SRC_LIST_RUNNER "src/runner/*.h" "src/runner/*.cpp" "src/runner/*.c" )

add_library(${PROJECT_NAME} ${SRC_LIST_LIB})
target_compile_features(${PROJECT_NAME} PRIVATE ${needed_features})
target_link_libraries(${PROJECT_NAME} humblelogging ${Boost_LIBRARIES} ${RE2_LIBRARIES} ${CMAKE_THREAD_LIBS_INIT})


add_executable("a4_runner" ${SRC_LIST_RUNNER})
target_compile_features("a4_runner" PRIVATE ${needed_features})
target_link_libraries("a4_runner" humblelogging ${Boost_LIBRARIES} ${RE2_LIBRARIES} ${PROJECT_NAME})


file(GLOB BENCHMARK_SRC_LIST "benchmarks/*.cpp" "benchmarks/*.h")
set(BENCHMARK_EXEC "benchmark_${PROJECT_NAME}")
include_directories(SYSTEM "ext/Celero-2.0.2/include/")
include_directories("src/lib/")
add_executable(${BENCHMARK_EXEC}  ${BENCHMARK_SRC_LIST} )
target_compile_features(${BENCHMARK_EXEC} PRIVATE ${needed_features})
target_link_libraries(${BENCHMARK_EXEC} celero ${PROJECT_NAME}  )

################################
# Testing with Google Test
################################
add_subdirectory(ext/gtest-1.7.0)
enable_testing()
include_directories(${gtest_SOURCE_DIR}/include ${gtest_SOURCE_DIR} "src/lib/")

file(GLOB TEST_SRC_LIST "tests/*.cpp" "tests/*.h")

set(TEST_EXEC "test_${PROJECT_NAME}")
add_executable(${TEST_EXEC}  ${TEST_SRC_LIST} )
target_compile_features(${TEST_EXEC} PRIVATE ${needed_features})
target_link_libraries(${TEST_EXEC} ${PROJECT_NAME} gtest gtest_main)
add_test( ${TEST_EXEC} ${TEST_EXEC} )
