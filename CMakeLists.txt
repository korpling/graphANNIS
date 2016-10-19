project(graphANNIS)
cmake_minimum_required(VERSION 3.5)

set(CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake")
set(GLOBAL_OUTPUT_PATH ${PROJECT_BINARY_DIR}/output)


set(needed_features
  cxx_auto_type
  cxx_override
  cxx_nullptr
  cxx_long_long_type
  cxx_deleted_functions
  cxx_static_assert)

include(ExternalProject)
include(cmake/humblelogging.cmake)
include(cmake/re2.cmake)

link_directories(${GLOBAL_OUTPUT_PATH})



IF(CMAKE_BUILD_TYPE MATCHES Debug)
  option(GRAPHANNIS_BUILD_BENCHMARKS "If ON build the benchmarks" OFF)
ELSE()
  option(GRAPHANNIS_BUILD_BENCHMARKS "If ON build the benchmarks" ON)
ENDIF()


find_package(ICU 4.4 REQUIRED)
find_package(Boost REQUIRED COMPONENTS system filesystem serialization)
find_package (Threads REQUIRED)

# write found packages to property file which is used by maven
file(WRITE "${GLOBAL_OUTPUT_PATH}/libdirs.properties" "Boost_INCLUDE_DIRS=${Boost_INCLUDE_DIRS}\n")
file(APPEND "${GLOBAL_OUTPUT_PATH}/libdirs.properties" "ICU_INCLUDE_DIRS=${ICU_INCLUDE_DIRS}\n")
file(APPEND "${GLOBAL_OUTPUT_PATH}/libdirs.properties" "Boost_LIBRARY_DIRS=${Boost_LIBRARY_DIRS}\n")



include_directories(
  "include"
)

include_directories(
  SYSTEM
  "ext/cpp-btree-1.0.1/"
  ${HumbleLogging_INCLUDE_DIRS}
  ${ICU_INCLUDE_DIRS}
  ${Boost_INCLUDE_DIRS}
  ${RE2_INCLUDE_DIRS}
)

# make sure QtCreator shows the include files
FILE(GLOB_RECURSE IncludeFiles "include/*.h")

file(GLOB SRC_LIST_LIB "src/lib/*.h" "src/lib/*.cpp" "src/lib/*.c"
  "src/lib/util/*.cpp" "src/lib/util/*.c"
  "src/lib/annosearch/*.cpp"
  "src/lib/api/*.cpp"
  "src/lib/graphstorage/*.cpp"
  "src/lib/operators/*.cpp"
  "src/lib/join/*.cpp"
  "src/lib/json/*.cpp"
)
file(GLOB SRC_LIST_RUNNER "src/runner/*.h" "src/runner/*.cpp" "src/runner/*.c" )

add_library(annis ${SRC_LIST_LIB} ${IncludeFiles})
target_compile_features(annis PRIVATE ${needed_features})
set_property(TARGET annis PROPERTY POSITION_INDEPENDENT_CODE TRUE)
# newer Boost serialization versions are otherwise incompatible with BTree
add_definitions(-DBOOST_NO_CXX11_HDR_UNORDERED_MAP -DBOOST_NO_CXX11_HDR_UNORDERED_SET)
if (MSVC)
	# the min/max defines get in our way when using std::min/std::max
    add_definitions(/DNOMINMAX)
elseif ( CMAKE_COMPILER_IS_GNUCC AND CMAKE_CXX_COMPILER_VERSION VERSION_GREATER 5.1.0 )
    set_property( TARGET annis APPEND_STRING PROPERTY COMPILE_FLAGS -Wsuggest-override )
endif (MSVC)

target_link_libraries(annis humblelogging ${Boost_LIBRARIES} ${RE2_LIBRARIES} ${CMAKE_THREAD_LIBS_INIT})
add_dependencies(annis HumbleLogging)
add_dependencies(annis RE2)

add_executable(annis_runner ${SRC_LIST_RUNNER})
target_compile_features(annis_runner PRIVATE ${needed_features})
target_link_libraries(annis_runner humblelogging ${Boost_LIBRARIES} ${RE2_LIBRARIES} annis)

################################
# Testing with Google Test
################################
add_subdirectory(ext/gtest-1.7.0)
enable_testing()
include_directories(SYSTEM ${gtest_SOURCE_DIR}/include ${gtest_SOURCE_DIR} "src/lib/")

file(GLOB TEST_SRC_LIST "tests/*.cpp" "tests/*.h")

set(TEST_EXEC "test_${PROJECT_NAME}")
add_executable(${TEST_EXEC}  ${TEST_SRC_LIST} )
target_compile_features(${TEST_EXEC} PRIVATE ${needed_features})
target_link_libraries(${TEST_EXEC} annis gtest gtest_main)
add_test( ${TEST_EXEC} ${TEST_EXEC} )

IF(GRAPHANNIS_BUILD_BENCHMARKS)

  set(CELERO_RUN_EXAMPLE_ON_BUILD OFF CACHE BOOL "Set to ON to automatically run the example after a successful build.")
  set(CELERO_ENABLE_EXPERIMENTS OFF CACHE BOOL "Set to ON to automatically build all examples.")
  set(CELERO_COMPILE_DYNAMIC_LIBRARIES OFF CACHE BOOL "Set to ON to build Celero for dynamic linking.  Use OFF for static.")
  add_subdirectory(ext/Celero-2.0.2)

  file(GLOB BENCHMARK_SRC_LIST "benchmarks/*.cpp" "benchmarks/*.h")
  set(BENCHMARK_EXEC "benchmark_${PROJECT_NAME}")
  include_directories(SYSTEM "ext/Celero-2.0.2/include/")
  include_directories("src/lib/")
  add_executable(${BENCHMARK_EXEC}  ${BENCHMARK_SRC_LIST} )
  target_compile_features(${BENCHMARK_EXEC} PRIVATE ${needed_features})
  target_link_libraries(${BENCHMARK_EXEC} celero annis  )

ENDIF()
