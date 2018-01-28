ExternalProject_Add(
    graphannis-rs
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release --all
    BINARY_DIR "${CMAKE_SOURCE_DIR}/graphannis"
    INSTALL_COMMAND ""
    LOG_BUILD OFF)

  if( CMAKE_SYSTEM_NAME STREQUAL "Linux" )
    set(GRAPHANNIS_RUST_LIBS "util" "dl" "rt" "pthread" "c" "m" "rt" "pthread" "util")
  elseif( CMAKE_SYSTEM_NAME STREQUAL "Darwin")
    set(GRAPHANNIS_RUST_LIBS "System" "resolv" "c" "m")
  else()
    # TODO: find a good default and add Windows
    set(GRAPHANNIS_RUST_LIBS "")
  endif()

  set(GRAPHANNISRS_LIBRARIES "${CMAKE_SOURCE_DIR}/graphannis/target/release/${CMAKE_STATIC_LIBRARY_PREFIX}graphannis${CMAKE_STATIC_LIBRARY_SUFFIX}" ${GRAPHANNIS_RUST_LIBS})
  set(GRAPHANNISRS_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/graphannis/include/")
