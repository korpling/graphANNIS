project(ANNIS4)
cmake_minimum_required(VERSION 2.8)

include_directories("include")
aux_source_directory(src SRC_LIST)

if(CMAKE_COMPILER_IS_GNUCXX)
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -std=c++11")
endif()

add_executable(${PROJECT_NAME} ${SRC_LIST})

