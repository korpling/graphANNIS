ExternalProject_Add(
    graphannis-rs
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build --release --all
    BINARY_DIR "${CMAKE_SOURCE_DIR}/graphannis-rs"
    INSTALL_COMMAND ""
    LOG_BUILD OFF)

  set(GRAPHANNISRS_LIBRARIES "${CMAKE_SOURCE_DIR}/graphannis-rs/target/release/${CMAKE_STATIC_LIBRARY_PREFIX}graphannis${CMAKE_STATIC_LIBRARY_SUFFIX}"
    "util" "dl" "rt" "pthread" "c" "m" "rt" "pthread" "util")
  set(GRAPHANNISRS_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/graphannis-rs/include/")
