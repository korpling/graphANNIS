ExternalProject_Add(
    graphannis-rs
    DOWNLOAD_COMMAND ""
    CONFIGURE_COMMAND ""
    BUILD_COMMAND cargo build COMMAND cargo build --release --all
    BINARY_DIR "${CMAKE_SOURCE_DIR}/graphannis-rs"
    INSTALL_COMMAND ""
    LOG_BUILD ON)

  set(GRAPHANNISRS_LIBRARIES "${CMAKE_SOURCE_DIR}/graphannis-rs/target/release/${CMAKE_STATIC_LIBRARY_PREFIX}graphannis_capi${CMAKE_STATIC_LIBRARY_SUFFIX}" "dl")
  set(GRAPHANNISRS_INCLUDE_DIRS "${CMAKE_SOURCE_DIR}/graphannis-rs/capi/include/")
