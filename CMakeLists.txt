project(graphANNIS)
cmake_minimum_required(VERSION 3.5)

set(CMAKE_MODULE_PATH "${PROJECT_SOURCE_DIR}/cmake")
set(GLOBAL_OUTPUT_PATH ${PROJECT_BINARY_DIR}/output)

# make sure a build type is selected
if(NOT CMAKE_BUILD_TYPE)
  set(CMAKE_BUILD_TYPE "Release" CACHE STRING "Choose the type of build, options are: Debug Release RelWithDebInfo MinSizeRel."
      FORCE)
endif(NOT CMAKE_BUILD_TYPE)

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
include(cmake/celero.cmake)
include(cmake/googletest.cmake)

link_directories(${GLOBAL_OUTPUT_PATH})



IF(CMAKE_BUILD_TYPE MATCHES Debug)
  option(GRAPHANNIS_BUILD_BENCHMARKS "If ON build the benchmarks" OFF)
ELSE()
  option(GRAPHANNIS_BUILD_BENCHMARKS "If ON build the benchmarks" ON)
ENDIF()


find_package(ICU 4.4 REQUIRED)
find_package(Boost REQUIRED COMPONENTS system filesystem thread)
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
  "ext/cpp-btree-1.0.1"
  ${HumbleLogging_INCLUDE_DIRS}
  ${ICU_INCLUDE_DIRS}
  ${Boost_INCLUDE_DIRS}
  ${RE2_INCLUDE_DIRS}
  "ext/cereal-1.2.1/include"
)

# make sure QtCreator shows the include files
set(IncludeFiles
  src/lib/annis/wrapper.h
  src/lib/annis/edgeannotationstorage.h
  src/lib/annis/stringstorage.h
  src/lib/annis/json/jsonqueryparser.h
  src/lib/annis/json/json.h
  src/lib/annis/json/json-forwards.h
  src/lib/annis/db.h
  src/lib/annis/graphstorage/prepostorderstorage.h
  src/lib/annis/graphstorage/adjacencyliststorage.h
  src/lib/annis/graphstorage/graphstorage.h
  src/lib/annis/graphstorage/linearstorage.h
  src/lib/annis/iterators.h
  src/lib/annis/types.h
  src/lib/annis/graphstorageholder.h
  src/lib/annis/filter.h
  src/lib/annis/operators/precedence.h
  src/lib/annis/operators/operator.h
  src/lib/annis/operators/inclusion.h
  src/lib/annis/operators/dominance.h
  src/lib/annis/operators/overlap.h
  src/lib/annis/operators/pointing.h
  src/lib/annis/operators/identicalcoverage.h
  src/lib/annis/operators/abstractedgeoperator.h
  src/lib/annis/dbcache.h
  src/lib/annis/api/corpusstoragemanager.h
  src/lib/annis/api/admin.h
  src/lib/annis/api/graphupdate.h
  src/lib/annis/serializers.h
  src/lib/annis/graphstorageregistry.h
  src/lib/annis/nodeannostorage.h
  src/lib/annis/util/helper.h
  src/lib/annis/util/plan.h
  src/lib/annis/util/dfs.h
  src/lib/annis/util/size_estimator.h
  src/lib/annis/util/comparefunctions.h
  src/lib/annis/query.h
  src/lib/annis/join/seed.h
  src/lib/annis/join/nestedloop.h
  src/lib/annis/annosearch/exactannovaluesearch.h
  src/lib/annis/annosearch/annotationsearch.h
  src/lib/annis/annosearch/exactannokeysearch.h
  src/lib/annis/annosearch/regexannosearch.h
)

set(SRC_LIST_LIB
  src/lib/annis/filter.cpp
  src/lib/annis/json/jsoncpp.cpp
  src/lib/annis/json/jsonqueryparser.cpp
  src/lib/annis/db.cpp
  src/lib/annis/wrapper.cpp
  src/lib/annis/graphstorage/adjacencyliststorage.cpp
  src/lib/annis/graphstorage/prepostorderstorage.cpp
  src/lib/annis/graphstorage/linearstorage.cpp
  src/lib/annis/graphstorage/graphstorage.cpp
  src/lib/annis/dbcache.cpp
  src/lib/annis/stringstorage.cpp
  src/lib/annis/graphstorageholder.cpp
  src/lib/annis/operators/overlap.cpp
  src/lib/annis/operators/pointing.cpp
  src/lib/annis/operators/precedence.cpp
  src/lib/annis/operators/identicalcoverage.cpp
  src/lib/annis/operators/inclusion.cpp
  src/lib/annis/operators/dominance.cpp
  src/lib/annis/operators/abstractedgeoperator.cpp
  src/lib/annis/api/corpusstoragemanager.cpp
  src/lib/annis/api/admin.cpp
  src/lib/annis/api/graphupdate.cpp
  src/lib/annis/edgeannotationstorage.cpp
  src/lib/annis/query.cpp
  src/lib/annis/util/dfs.cpp
  src/lib/annis/util/plan.cpp
  src/lib/annis/util/getRSS.cpp
  src/lib/annis/nodeannostorage.cpp
  src/lib/annis/graphstorageregistry.cpp
  src/lib/annis/join/nestedloop.cpp
  src/lib/annis/join/seed.cpp
  src/lib/annis/annosearch/exactannokeysearch.cpp
  src/lib/annis/annosearch/exactannovaluesearch.cpp
  src/lib/annis/annosearch/regexannosearch.cpp
  src/lib/annis/annosearch/annotationsearch.cpp
)

set(SRC_LIST_RUNNER
  src/runner/console.h
  src/runner/main.cpp
  src/runner/linenoise.h
  src/runner/linenoise.c
  src/runner/console.cpp
)

include_directories( "src/lib")

add_library(annis ${SRC_LIST_LIB})
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

target_link_libraries(annis ${HumbleLogging_LIBRARIES} ${Boost_LIBRARIES} ${RE2_LIBRARIES} ${CMAKE_THREAD_LIBS_INIT})
add_dependencies(annis HumbleLogging)
add_dependencies(annis RE2)	

add_executable(annis_runner ${SRC_LIST_RUNNER})
target_compile_features(annis_runner PRIVATE ${needed_features})
target_link_libraries(annis_runner ${HumbleLogging_LIBRARIES} ${Boost_LIBRARIES} ${RE2_LIBRARIES} annis)

################################
# Testing with Google Test
################################
enable_testing()
include_directories(SYSTEM ${gtest_SOURCE_DIR}/include ${gtest_SOURCE_DIR})

set(TEST_SRC_LIST
  tests/testlogger.h
  tests/SearchTestPcc2.h
  tests/SearchTestParlament.h
  tests/SearchTestRidges.h
  tests/CorpusStorageManagerTest.h
  tests/SearchTestGUM.h
  tests/SearchTestTueBaDZ.h
  tests/LoadTest.h
  tests/SearchTestTiger.h
  tests/testmain.cpp
)

set(TEST_EXEC "test_${PROJECT_NAME}")

add_executable(${TEST_EXEC}  ${TEST_SRC_LIST} )
add_dependencies(${TEST_EXEC} GoogleTest)

target_compile_features(${TEST_EXEC} PRIVATE ${needed_features})
target_link_libraries(${TEST_EXEC} annis ${GoogleTest_LIBRARIES})
add_test( ${TEST_EXEC} ${TEST_EXEC} )


if(GRAPHANNIS_BUILD_BENCHMARKS)
  file(GLOB BENCHMARK_SRC_LIST "benchmarks/*.cpp" "benchmarks/*.h")

  set(BENCHMARK_EXEC "benchmark_${PROJECT_NAME}")
  add_executable(${BENCHMARK_EXEC}  ${BENCHMARK_SRC_LIST} )
  add_dependencies(${BENCHMARK_EXEC} Celero)
  target_compile_features(${BENCHMARK_EXEC} PRIVATE ${needed_features})
  target_link_libraries(${BENCHMARK_EXEC} ${Celero_LIBRARIES} annis  )

endif()
